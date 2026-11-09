//! Subcontracts (FEO-049): work let to a supplier under a contract, at its value. A subcontract's
//! claim says the value of its work done to date. Certified by a Contracts Manager other than
//! whoever drew it up, what is due after the retention held and earlier claims is billed through ERP
//! as a draft purchase invoice from the supplier, numbered as the claim. Certified work is not taken
//! back, and claims follow one another in time.

use crate::{decides, obj, raised_by, status};
use async_trait::async_trait;
use ferrum_domain::flow::created;
use ferrum_domain::util::{d, elevated, id, money, round, s, sys};
use ferrum_kernel::{Doc, DocHook, Error, HookCtx, Result};
use rust_decimal::Decimal;
use serde_json::{Value, json};
use uuid::Uuid;

pub struct SubcontractHook;

#[async_trait]
impl DocHook for SubcontractHook {
    fn name(&self) -> &'static str {
        "construction.subcontract"
    }

    fn fills(&self) -> &'static [&'static str] {
        &["retention_percent"]
    }

    async fn before_validate(&self, hc: &mut HookCtx<'_>, doc: &mut Doc) -> Result<()> {
        // The retention held from its claims: the settings' when it names none.
        if doc.get("retention_percent").is_none_or(Value::is_null) {
            let r: Option<Decimal> = sqlx::query_scalar("select retention_percent from e_construction_settings where tenant_id = $1 limit 1")
                .bind(hc.ctx.tenant)
                .fetch_optional(&mut **hc.tx)
                .await?
                .flatten();
            doc.insert("retention_percent".into(), money(r.unwrap_or_default()));
        }
        Ok(())
    }

    async fn validate(&self, _hc: &mut HookCtx<'_>, doc: &Doc) -> Result<()> {
        let value = d(doc, "value");
        if value <= Decimal::ZERO {
            return Err(Error::field("value", "is above zero"));
        }
        if value < d(doc, "certified_to_date") {
            return Err(Error::field("value", format!("is less than the {} certified", d(doc, "certified_to_date"))));
        }
        let r = d(doc, "retention_percent");
        if r < Decimal::ZERO || r > Decimal::ONE_HUNDRED {
            return Err(Error::field("retention_percent", "is from 0 to 100"));
        }
        Ok(())
    }
}

pub struct SubcontractClaimHook;

/// The claim's subcontract.
async fn subcontract_of(hc: &mut HookCtx<'_>, doc: &Doc) -> Result<Doc> {
    let sub = id(doc, "subcontract").ok_or_else(|| Error::field("subcontract", "is required"))?;
    hc.kernel.get_tx(&sys(hc.ctx), hc.tx, "Subcontract", sub, false).await
}

/// The last certified claim on the subcontract other than `claim`: its value to date and period.
async fn last_certified(hc: &mut HookCtx<'_>, sub: Uuid, claim: Option<Uuid>) -> Result<Option<(Decimal, chrono::NaiveDate)>> {
    Ok(sqlx::query_as(
        "select value_to_date, period_to from e_subcontract_claim where tenant_id = $1 and subcontract = $2 and status = 'Certified' and id <> $3 \
         order by period_to desc limit 1",
    )
    .bind(hc.ctx.tenant)
    .bind(sub)
    .bind(claim.unwrap_or_else(Uuid::nil))
    .fetch_optional(&mut **hc.tx)
    .await?)
}

/// The retention held to date, what earlier claims certified, and what is due now.
async fn compute(hc: &mut HookCtx<'_>, doc: &mut Doc) -> Result<()> {
    let sub = subcontract_of(hc, doc).await?;
    let to_date = d(doc, "value_to_date");
    let retention = round(to_date * d(&sub, "retention_percent") / Decimal::ONE_HUNDRED, 2);
    let previous: Decimal = sqlx::query_scalar(
        "select coalesce(sum(amount_due), 0) from e_subcontract_claim where tenant_id = $1 and subcontract = $2 and status = 'Certified' and id <> $3",
    )
    .bind(hc.ctx.tenant)
    .bind(id(&sub, "id"))
    .bind(id(doc, "id").unwrap_or_else(Uuid::nil))
    .fetch_one(&mut **hc.tx)
    .await?;
    doc.insert("retention".into(), money(retention));
    doc.insert("previous_certified".into(), money(previous));
    doc.insert("amount_due".into(), money(to_date - retention - previous));
    Ok(())
}

#[async_trait]
impl DocHook for SubcontractClaimHook {
    fn name(&self) -> &'static str {
        "construction.subcontract_claim"
    }

    fn fills(&self) -> &'static [&'static str] {
        &["retention", "previous_certified", "amount_due"]
    }

    async fn before_validate(&self, hc: &mut HookCtx<'_>, doc: &mut Doc) -> Result<()> {
        if status(doc) == "Certified" {
            return Ok(());
        }
        compute(hc, doc).await
    }

    async fn validate(&self, hc: &mut HookCtx<'_>, doc: &Doc) -> Result<()> {
        if status(doc) == "Certified" {
            return Ok(());
        }
        let sub = subcontract_of(hc, doc).await?;
        let to_date = d(doc, "value_to_date");
        if to_date < Decimal::ZERO || to_date > d(&sub, "value") {
            return Err(Error::field("value_to_date", format!("is from 0 to the subcontract's {}", d(&sub, "value"))));
        }
        if let Some((was, on)) = last_certified(hc, id(&sub, "id").unwrap_or_default(), id(doc, "id")).await? {
            if to_date < was {
                return Err(Error::field("value_to_date", format!("{was} was certified: work certified is not taken back")));
            }
            if s(doc, "period_to").and_then(|p| p.parse::<chrono::NaiveDate>().ok()).is_some_and(|p| p <= on) {
                return Err(Error::field("period_to", format!("is after {on}, the last certified claim's")));
            }
        }
        Ok(())
    }

    async fn validate_change(&self, hc: &mut HookCtx<'_>, old: Option<&Doc>, _doc: &Doc) -> Result<()> {
        if !hc.ctx.is_system && old.is_some_and(|o| status(o) == "Certified") {
            return Err(Error::invalid("a certified claim is not changed: the next claim carries on from it"));
        }
        Ok(())
    }

    async fn action(&self, hc: &mut HookCtx<'_>, name: &str, doc: &Doc, _params: &Value) -> Result<Option<Value>> {
        if name != "certify" {
            return Ok(None);
        }
        if !decides(hc.ctx) {
            return Err(Error::Forbidden("a Contracts Manager certifies a subcontract claim".into()));
        }
        if raised_by(doc, hc.ctx) {
            return Err(Error::Forbidden("someone other than whoever drew up a claim certifies it".into()));
        }
        let claim = id(doc, "id").ok_or_else(|| Error::internal("a claim without an id"))?;
        // Against the claims certified as they stand now.
        let mut now = doc.clone();
        compute(hc, &mut now).await?;
        let due = d(&now, "amount_due");
        if due <= Decimal::ZERO {
            return Err(Error::invalid("it adds no work: nothing is due"));
        }
        let sub = subcontract_of(hc, doc).await?;
        let system = elevated(hc.ctx);
        let (company, item, _) = crate::settings(hc.tx, hc.ctx.tenant).await?;
        let number = s(doc, "number").unwrap_or_default().to_string();
        let what = format!("Subcontract claim {number}, {}", s(&sub, "title").unwrap_or_default());
        let line = json!({"item": item.to_string(), "qty": "1", "price_list_rate": money(due), "discount_percent": "0"});
        let input = obj(json!({"company": company.to_string(), "supplier": s(&sub, "supplier"), "bill_no": number, "items": [line], "remarks": what}));
        let bill = hc.kernel.insert_tx(&system, hc.tx, "PurchaseInvoice", input).await?;
        let values = obj(json!({"status": "Certified", "certified_by": hc.ctx.email, "retention": now.get("retention"),
                                "previous_certified": now.get("previous_certified"), "amount_due": money(due), "purchase_invoice": bill.get("id")}));
        hc.kernel.set_values_tx(&system, hc.tx, "SubcontractClaim", claim, values).await?;
        let sub_id = id(&sub, "id").ok_or_else(|| Error::internal("a subcontract without an id"))?;
        let held = obj(json!({"certified_to_date": doc.get("value_to_date"), "retention_held": now.get("retention")}));
        hc.kernel.set_values_tx(&system, hc.tx, "Subcontract", sub_id, held).await?;
        Ok(Some(json!({"status": "Certified", "amount_due": money(due), "purchase_invoice": created("PurchaseInvoice", &bill)})))
    }
}
