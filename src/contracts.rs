//! Contracts and variations. A contract is worth the sum of its lines. Once work on it is certified
//! or varied, its lines change only through approved variations, each added as a line of its own;
//! one that would make it worth less than what is certified is refused. Completing a contract
//! certified in full invoices the retention held.

use crate::{decides, invoice, obj, status, sum};
use async_trait::async_trait;
use chrono::Utc;
use ferrum_domain::flow::created;
use ferrum_domain::util::{d, elevated, id, money, rows, s};
use ferrum_kernel::{Doc, DocHook, Error, HookCtx, Result};
use rust_decimal::Decimal;
use serde_json::{Value, json};
use std::collections::HashSet;

pub struct ContractHook;

/// What a contract's lines say: each one's code, description and value.
fn schedule(doc: &Doc) -> Vec<(Option<String>, Option<String>, Decimal)> {
    rows(doc, "lines").iter().map(|l| (s(l, "code").map(str::to_string), s(l, "description").map(str::to_string), d(l, "value"))).collect()
}

#[async_trait]
impl DocHook for ContractHook {
    fn name(&self) -> &'static str {
        "construction.contract"
    }

    fn fills(&self) -> &'static [&'static str] {
        &["original_sum", "contract_sum", "retention_percent"]
    }

    async fn before_validate(&self, hc: &mut HookCtx<'_>, doc: &mut Doc) -> Result<()> {
        // A contract holds the retention agreed for it: the settings' when it names none.
        if doc.get("retention_percent").is_none_or(Value::is_null) {
            let r: Option<Decimal> = sqlx::query_scalar("select retention_percent from e_construction_settings where tenant_id = $1 limit 1")
                .bind(hc.ctx.tenant)
                .fetch_optional(&mut **hc.tx)
                .await?
                .flatten();
            doc.insert("retention_percent".into(), money(r.unwrap_or_default()));
        }
        let total = sum(&rows(doc, "lines"), "value");
        doc.insert("contract_sum".into(), money(total));
        doc.insert("original_sum".into(), money(total - d(doc, "variations_sum")));
        Ok(())
    }

    async fn validate(&self, _hc: &mut HookCtx<'_>, doc: &Doc) -> Result<()> {
        let lines = rows(doc, "lines");
        let mut seen = HashSet::new();
        if lines.iter().any(|l| !seen.insert(s(l, "code").unwrap_or_default().to_string())) {
            return Err(Error::field("lines", "each line has a code of its own"));
        }
        let r = d(doc, "retention_percent");
        if r < Decimal::ZERO || r > Decimal::ONE_HUNDRED {
            return Err(Error::field("retention_percent", "is from 0 to 100"));
        }
        Ok(())
    }

    async fn validate_change(&self, hc: &mut HookCtx<'_>, old: Option<&Doc>, doc: &Doc) -> Result<()> {
        if hc.ctx.is_system {
            return Ok(());
        }
        let Some(o) = old else { return Ok(()) };
        if status(o) == "Complete" {
            return Err(Error::invalid("a complete contract is not changed"));
        }
        let started = d(o, "certified_to_date") != Decimal::ZERO || d(o, "variations_sum") != Decimal::ZERO;
        if started && (schedule(o) != schedule(doc) || s(o, "customer") != s(doc, "customer") || d(o, "retention_percent") != d(doc, "retention_percent")) {
            return Err(Error::invalid("work on this contract is certified or varied: its lines change through variations"));
        }
        Ok(())
    }

    async fn action(&self, hc: &mut HookCtx<'_>, name: &str, doc: &Doc, _params: &Value) -> Result<Option<Value>> {
        if name != "complete" {
            return Ok(None);
        }
        let contract = id(doc, "id").ok_or_else(|| Error::internal("a contract without an id"))?;
        if !decides(hc.ctx) {
            return Err(Error::Forbidden("a Contracts Manager completes a contract".into()));
        }
        let (certified, total) = (d(doc, "certified_to_date"), d(doc, "contract_sum"));
        if certified != total {
            return Err(Error::invalid(format!("certified {certified} of {total}: certify the whole contract before completing it")));
        }
        let system = elevated(hc.ctx);
        let held = d(doc, "retention_held");
        let mut values = obj(json!({"status": "Complete", "completed_on": Utc::now().date_naive().to_string(), "retention_held": money(Decimal::ZERO)}));
        let mut out = json!({"status": "Complete"});
        if held > Decimal::ZERO {
            let customer = s(doc, "customer").ok_or_else(|| Error::field("customer", "is required"))?;
            let what = format!("Retention released, {}", s(doc, "number").unwrap_or_default());
            let inv = invoice(hc.kernel, &system, hc.tx, customer, held, what).await?;
            values.insert("retention_invoice".into(), inv.get("id").cloned().unwrap_or(Value::Null));
            out["retention_invoice"] = created("SalesInvoice", &inv);
        }
        hc.kernel.set_values_tx(&system, hc.tx, "ConstructionContract", contract, values).await?;
        Ok(Some(out))
    }
}

pub struct VariationHook;

#[async_trait]
impl DocHook for VariationHook {
    fn name(&self) -> &'static str {
        "construction.variation"
    }

    async fn validate(&self, hc: &mut HookCtx<'_>, doc: &Doc) -> Result<()> {
        if d(doc, "amount").is_zero() {
            return Err(Error::field("amount", "a variation adds or takes out some value"));
        }
        let contract = id(doc, "contract").ok_or_else(|| Error::field("contract", "is required"))?;
        let st: Option<String> = sqlx::query_scalar("select status from e_construction_contract where tenant_id = $1 and id = $2")
            .bind(hc.ctx.tenant)
            .bind(contract)
            .fetch_optional(&mut **hc.tx)
            .await?
            .flatten();
        if st.as_deref() == Some("Complete") && status(doc) == "Draft" {
            return Err(Error::field("contract", "is complete: it takes no more variations"));
        }
        Ok(())
    }

    async fn validate_change(&self, hc: &mut HookCtx<'_>, old: Option<&Doc>, doc: &Doc) -> Result<()> {
        if hc.ctx.is_system {
            return Ok(());
        }
        if let Some(o) = old
            && status(o) != "Draft"
            && (s(o, "contract") != s(doc, "contract") || s(o, "description") != s(doc, "description") || d(o, "amount") != d(doc, "amount"))
        {
            return Err(Error::invalid(format!("a variation that is {} keeps its terms", status(o).to_lowercase())));
        }
        Ok(())
    }

    async fn action(&self, hc: &mut HookCtx<'_>, name: &str, doc: &Doc, _params: &Value) -> Result<Option<Value>> {
        let variation = id(doc, "id").ok_or_else(|| Error::internal("a variation without an id"))?;
        let system = elevated(hc.ctx);
        let by = hc.ctx.email.clone();
        let decided = |to: &str| obj(json!({"status": to, "decided_on": Utc::now().date_naive().to_string(), "decided_by": by}));
        match name {
            "submit_variation" => {
                hc.kernel.set_values_tx(&system, hc.tx, "Variation", variation, obj(json!({"status": "Submitted"}))).await?;
                Ok(Some(json!({"status": "Submitted"})))
            }
            "reject" => {
                if !decides(hc.ctx) {
                    return Err(Error::Forbidden("a Contracts Manager decides a variation".into()));
                }
                hc.kernel.set_values_tx(&system, hc.tx, "Variation", variation, decided("Rejected")).await?;
                Ok(Some(json!({"status": "Rejected"})))
            }
            "approve" => {
                if !decides(hc.ctx) {
                    return Err(Error::Forbidden("a Contracts Manager decides a variation".into()));
                }
                let contract_id = id(doc, "contract").ok_or_else(|| Error::field("contract", "is required"))?;
                let contract = hc.kernel.get_tx(&system, hc.tx, "ConstructionContract", contract_id, true).await?;
                if status(&contract) == "Complete" {
                    return Err(Error::invalid("the contract is complete: it takes no more variations"));
                }
                let amount = d(doc, "amount");
                let worth = d(&contract, "contract_sum") + amount;
                if worth < d(&contract, "certified_to_date") {
                    return Err(Error::invalid(format!("the contract would be worth {worth}, less than the {} certified", d(&contract, "certified_to_date"))));
                }
                let mut lines: Vec<Value> = rows(&contract, "lines").into_iter().map(Value::Object).collect();
                lines
                    .push(json!({"code": format!("V-{}", s(doc, "number").unwrap_or_default()), "description": s(doc, "description"), "value": money(amount)}));
                let varied = d(&contract, "variations_sum") + amount;
                hc.kernel
                    .update_tx(&system, hc.tx, "ConstructionContract", contract_id, obj(json!({"lines": lines, "variations_sum": money(varied)})), None)
                    .await?;
                hc.kernel.set_values_tx(&system, hc.tx, "Variation", variation, decided("Approved")).await?;
                Ok(Some(json!({"status": "Approved", "contract_sum": money(worth)})))
            }
            _ => Ok(None),
        }
    }
}
