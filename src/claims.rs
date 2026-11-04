//! Progress claims. A claim measures each line of its contract by its code, as a percentage done
//! to date; its value to date is those lines' values so far, less the retention held, less what
//! earlier claims certified. Work certified is not taken back: a line is never claimed at less than
//! the last certified claim had it, and claims follow one another in time. Certifying recomputes
//! the claim against the contract and the claims certified as they stand, then invoices what is due
//! through ERP.

use crate::{decides, invoice, obj, status};
use async_trait::async_trait;
use ferrum_domain::flow::created;
use ferrum_domain::util::{d, elevated, id, money, round, rows, s};
use ferrum_kernel::{Ctx, Doc, DocHook, Error, Filter, HookCtx, Kernel, Result, Tx};
use rust_decimal::Decimal;
use serde_json::{Value, json};
use std::collections::HashMap;
use uuid::Uuid;

pub struct ClaimHook;

/// The last certified claim on `contract` other than `claim`, if any.
async fn last_certified(k: &Kernel, ctx: &Ctx, tx: &mut Tx, contract: Uuid, claim: Option<Uuid>) -> Result<Option<Doc>> {
    let found = k
        .find_tx(ctx, tx, "ProgressClaim", &[Filter::eq("contract", contract.to_string()), Filter::eq("status", "Certified")], Some("period_to desc"), 2)
        .await?;
    // Read whole: a list's rows come without their lines.
    match found.into_iter().find_map(|c| id(&c, "id").filter(|i| Some(*i) != claim)) {
        Some(last) => Ok(Some(k.get_tx(ctx, tx, "ProgressClaim", last, false).await?)),
        None => Ok(None),
    }
}

/// Each line's percentage done, by code.
fn percents(doc: &Doc) -> HashMap<String, Decimal> {
    rows(doc, "lines").iter().filter_map(|l| s(l, "code").map(|c| (c.to_string(), d(l, "percent_complete")))).collect()
}

/// The claim's lines and amounts, from its contract's lines as they stand.
async fn compute(hc: &mut HookCtx<'_>, doc: &mut Doc) -> Result<()> {
    let contract_id = id(doc, "contract").ok_or_else(|| Error::field("contract", "is required"))?;
    let system = ferrum_domain::util::sys(hc.ctx);
    let contract = hc.kernel.get_tx(&system, hc.tx, "ConstructionContract", contract_id, false).await?;
    let me = id(doc, "id");
    let before = last_certified(hc.kernel, &system, hc.tx, contract_id, me).await?;
    let given = percents(doc);
    // Rows keep their ids, so a claim's lines stay the same rows as it is redrawn.
    let ids: HashMap<String, Value> =
        rows(doc, "lines").iter().filter_map(|l| Some((s(l, "code")?.to_string(), l.get("id").filter(|v| v.is_string())?.clone()))).collect();
    let schedule = rows(&contract, "lines");
    if let Some(code) = given.keys().find(|c| !schedule.iter().any(|l| s(l, "code") == Some(c.as_str()))) {
        return Err(Error::field("lines", format!("line {code} is not on the contract")));
    }
    // A line the claim does not mention stays where the last certified claim had it. Brought up to
    // date when it is certified (as the system), a draft drawn up before the claim ahead of it was
    // certified has no line below what that claim certified.
    let earlier = before.as_ref().map(percents).unwrap_or_default();
    let floor = hc.ctx.is_system;
    let mut lines = vec![];
    let mut to_date = Decimal::ZERO;
    for l in &schedule {
        let code = s(l, "code").unwrap_or_default();
        let mut pct = given.get(code).or_else(|| earlier.get(code)).copied().unwrap_or_default();
        if floor && let Some(was) = earlier.get(code) {
            pct = pct.max(*was);
        }
        let value = round(d(l, "value") * pct / Decimal::ONE_HUNDRED, 2);
        to_date += value;
        let row = ids.get(code).cloned().unwrap_or_else(|| json!(Uuid::now_v7().to_string()));
        lines.push(json!({"id": row, "idx": lines.len(), "code": code, "description": s(l, "description"), "value": money(d(l, "value")),
                          "percent_complete": money(pct), "value_to_date": money(value)}));
    }
    let retention = round(to_date * d(&contract, "retention_percent") / Decimal::ONE_HUNDRED, 2);
    let previous: Decimal = sqlx::query_scalar(
        "select coalesce(sum(amount_due), 0) from e_progress_claim where tenant_id = $1 and contract = $2 and status = 'Certified' and id <> $3",
    )
    .bind(hc.ctx.tenant)
    .bind(contract_id)
    .bind(me.unwrap_or_else(Uuid::nil))
    .fetch_one(&mut **hc.tx)
    .await?;
    doc.insert("lines".into(), Value::Array(lines));
    doc.insert("value_to_date".into(), money(to_date));
    doc.insert("retention".into(), money(retention));
    doc.insert("previous_certified".into(), money(previous));
    doc.insert("amount_due".into(), money(to_date - retention - previous));
    Ok(())
}

/// Refused when the claim takes back certified work or comes before the last certified claim.
async fn check_progress(hc: &mut HookCtx<'_>, doc: &Doc) -> Result<()> {
    let contract_id = id(doc, "contract").ok_or_else(|| Error::field("contract", "is required"))?;
    let system = ferrum_domain::util::sys(hc.ctx);
    let Some(before) = last_certified(hc.kernel, &system, hc.tx, contract_id, id(doc, "id")).await? else { return Ok(()) };
    if let (Some(prev), Some(now)) = (s(&before, "period_to"), s(doc, "period_to"))
        && now <= prev
    {
        return Err(Error::field("period_to", format!("is after {prev}, the last certified claim's")));
    }
    let earlier = percents(&before);
    for l in rows(doc, "lines") {
        let code = s(&l, "code").unwrap_or_default();
        if let Some(was) = earlier.get(code)
            && d(&l, "percent_complete") < *was
        {
            return Err(Error::field("lines", format!("line {code} was certified at {was}%: work certified is not taken back")));
        }
    }
    Ok(())
}

#[async_trait]
impl DocHook for ClaimHook {
    fn name(&self) -> &'static str {
        "construction.claim"
    }

    fn fills(&self) -> &'static [&'static str] {
        &["lines", "value_to_date", "retention", "previous_certified", "amount_due"]
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
        if rows(doc, "lines").iter().any(|l| d(l, "percent_complete") < Decimal::ZERO || d(l, "percent_complete") > Decimal::ONE_HUNDRED) {
            return Err(Error::field("lines", "each line is from 0% to 100% done"));
        }
        let contract = id(doc, "contract").ok_or_else(|| Error::field("contract", "is required"))?;
        let st: Option<String> = sqlx::query_scalar("select status from e_construction_contract where tenant_id = $1 and id = $2")
            .bind(hc.ctx.tenant)
            .bind(contract)
            .fetch_optional(&mut **hc.tx)
            .await?
            .flatten();
        if st.as_deref() == Some("Complete") {
            return Err(Error::field("contract", "is complete: it takes no more claims"));
        }
        check_progress(hc, doc).await
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
            return Err(Error::Forbidden("a Contracts Manager certifies a claim".into()));
        }
        let claim = id(doc, "id").ok_or_else(|| Error::internal("a claim without an id"))?;
        let contract_id = id(doc, "contract").ok_or_else(|| Error::field("contract", "is required"))?;
        let system = elevated(hc.ctx);
        // The contract is held while its claim is certified, so two claims are never certified
        // against the same past.
        let contract = hc.kernel.get_tx(&system, hc.tx, "ConstructionContract", contract_id, true).await?;
        if status(&contract) == "Complete" {
            return Err(Error::invalid("the contract is complete: it takes no more claims"));
        }
        let fresh = hc.kernel.update_tx(&system, hc.tx, "ProgressClaim", claim, Doc::new(), None).await?;
        check_progress(hc, &fresh).await?;
        let due = d(&fresh, "amount_due");
        if due <= Decimal::ZERO {
            return Err(Error::invalid("this claim adds no work to what is certified"));
        }
        let customer = s(&contract, "customer").ok_or_else(|| Error::field("customer", "is required"))?;
        let what = format!("Progress claim {}, work to {}", s(&fresh, "number").unwrap_or_default(), s(&fresh, "period_to").unwrap_or_default());
        let inv = invoice(hc.kernel, &system, hc.tx, customer, due, what).await?;
        hc.kernel.set_values_tx(&system, hc.tx, "ProgressClaim", claim, obj(json!({"status": "Certified", "sales_invoice": inv.get("id")}))).await?;
        hc.kernel
            .set_values_tx(
                &system,
                hc.tx,
                "ConstructionContract",
                contract_id,
                obj(json!({"certified_to_date": money(d(&fresh, "value_to_date")), "retention_held": money(d(&fresh, "retention"))})),
            )
            .await?;
        Ok(Some(json!({"status": "Certified", "amount_due": money(due), "sales_invoice": created("SalesInvoice", &inv)})))
    }
}
