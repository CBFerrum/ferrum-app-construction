//! Estimates: each line's amount is its quantity at its rate, and the total adds the markup. Sent,
//! an estimate keeps its prices; accepted, it becomes a contract whose schedule of values is its
//! lines, the markup spread across them so the contract is worth the estimate's total.

use crate::{obj, status, sum};
use async_trait::async_trait;
use ferrum_domain::flow::created;
use ferrum_domain::util::{d, elevated, id, money, round, rows, s};
use ferrum_kernel::{Doc, DocHook, Error, HookCtx, Result};
use rust_decimal::Decimal;
use serde_json::{Value, json};

pub struct EstimateHook;

/// What an estimate's lines price: each one's description, quantity and rate.
fn priced(doc: &Doc) -> Vec<(Option<String>, Decimal, Decimal)> {
    rows(doc, "lines").iter().map(|l| (s(l, "description").map(str::to_string), d(l, "qty"), d(l, "rate"))).collect()
}

/// The lines' values once the markup is spread across them, in cents, the last line taking what
/// rounding leaves so they add up to `total`.
pub(crate) fn spread(amounts: &[Decimal], total: Decimal) -> Vec<Decimal> {
    let subtotal: Decimal = amounts.iter().sum();
    if amounts.is_empty() || subtotal.is_zero() {
        return amounts.to_vec();
    }
    let mut out: Vec<Decimal> = amounts.iter().map(|a| round(*a * total / subtotal, 2)).collect();
    let left = total - out.iter().sum::<Decimal>();
    if let Some(last) = out.last_mut() {
        *last += left;
    }
    out
}

#[async_trait]
impl DocHook for EstimateHook {
    fn name(&self) -> &'static str {
        "construction.estimate"
    }

    fn fills(&self) -> &'static [&'static str] {
        &["lines", "subtotal", "total"]
    }

    async fn before_validate(&self, _hc: &mut HookCtx<'_>, doc: &mut Doc) -> Result<()> {
        let mut lines = rows(doc, "lines");
        for l in &mut lines {
            let amount = round(d(l, "qty") * d(l, "rate"), 2);
            l.insert("amount".into(), money(amount));
        }
        let subtotal = sum(&lines, "amount");
        let total = round(subtotal * (Decimal::ONE + d(doc, "markup_percent") / Decimal::ONE_HUNDRED), 2);
        doc.insert("lines".into(), Value::Array(lines.into_iter().map(Value::Object).collect()));
        doc.insert("subtotal".into(), money(subtotal));
        doc.insert("total".into(), money(total));
        Ok(())
    }

    async fn validate(&self, _hc: &mut HookCtx<'_>, doc: &Doc) -> Result<()> {
        let lines = rows(doc, "lines");
        if lines.iter().any(|l| d(l, "qty") <= Decimal::ZERO || d(l, "rate") < Decimal::ZERO) {
            return Err(Error::field("lines", "each line has a quantity above zero and a rate of zero or more"));
        }
        if d(doc, "markup_percent") < Decimal::ZERO {
            return Err(Error::field("markup_percent", "is zero or more"));
        }
        Ok(())
    }

    async fn validate_change(&self, hc: &mut HookCtx<'_>, old: Option<&Doc>, doc: &Doc) -> Result<()> {
        if hc.ctx.is_system {
            return Ok(());
        }
        if let Some(o) = old
            && status(o) != "Draft"
            && (s(o, "customer") != s(doc, "customer") || d(o, "markup_percent") != d(doc, "markup_percent") || priced(o) != priced(doc))
        {
            return Err(Error::invalid(format!("an estimate that is {} keeps its prices: copy it to price again", status(o).to_lowercase())));
        }
        Ok(())
    }

    async fn action(&self, hc: &mut HookCtx<'_>, name: &str, doc: &Doc, _params: &Value) -> Result<Option<Value>> {
        let estimate = id(doc, "id").ok_or_else(|| Error::internal("an estimate without an id"))?;
        let system = elevated(hc.ctx);
        let to = match name {
            "send" => "Sent",
            "decline" => "Declined",
            "accept" => {
                let lines = rows(doc, "lines");
                let values = spread(&lines.iter().map(|l| d(l, "amount")).collect::<Vec<_>>(), d(doc, "total"));
                let schedule: Vec<Value> = lines
                    .iter()
                    .zip(values)
                    .enumerate()
                    .map(|(i, (l, v))| json!({"code": (i + 1).to_string(), "description": s(l, "description"), "value": money(v)}))
                    .collect();
                let contract = hc
                    .kernel
                    .insert_tx(
                        &system,
                        hc.tx,
                        "ConstructionContract",
                        obj(json!({"title": s(doc, "title"), "customer": s(doc, "customer"), "project": s(doc, "project"), "estimate": estimate.to_string(), "lines": schedule})),
                    )
                    .await?;
                hc.kernel.set_values_tx(&system, hc.tx, "Estimate", estimate, obj(json!({"status": "Accepted", "contract": contract.get("id")}))).await?;
                return Ok(Some(json!({"status": "Accepted", "contract": created("ConstructionContract", &contract)})));
            }
            _ => return Ok(None),
        };
        hc.kernel.set_values_tx(&system, hc.tx, "Estimate", estimate, obj(json!({"status": to}))).await?;
        Ok(Some(json!({"status": to})))
    }
}

#[cfg(test)]
mod tests {
    use super::spread;
    use rust_decimal::Decimal;

    #[test]
    fn the_markup_is_spread_so_the_lines_add_up_to_the_total() {
        let v = |s: &str| s.parse::<Decimal>().unwrap();
        let out = spread(&[v("100"), v("100"), v("100")], v("310"));
        assert_eq!(out, vec![v("103.33"), v("103.33"), v("103.34")]);
        assert_eq!(out.iter().sum::<Decimal>(), v("310"));
        assert_eq!(spread(&[v("0")], v("0")), vec![v("0")]);
    }
}
