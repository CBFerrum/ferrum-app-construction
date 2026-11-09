//! Construction Margin (FEO-049): each contract's worth and what is certified of it, beside its
//! subcontracts' worth and what is certified of them. The margin is what was certified to the client
//! less what was certified to subcontractors; the forecast, the contract sum less the subcontracts'.
//! Contracts Managers and administrators only: it shows what subcontracts cost.

use crate::decides;
use async_trait::async_trait;
use ferrum_domain::reports::{ReportSource, col, read_tx};
use ferrum_domain::util::money;
use ferrum_kernel::{Ctx, Error, Kernel, Result};
use rust_decimal::Decimal;
use serde_json::{Value, json};
use std::collections::HashMap;

pub struct Margin;

#[async_trait]
impl ReportSource for Margin {
    fn catalog(&self) -> Vec<Value> {
        vec![json!({"name": "construction_margin", "label": "Construction Margin", "module": "construction", "entity": "ConstructionContract", "params": []})]
    }

    async fn run(&self, k: &Kernel, ctx: &Ctx, name: &str, _params: &HashMap<String, String>) -> Result<Value> {
        if name != "construction_margin" {
            return Err(Error::NotFound(format!("report {name}")));
        }
        if !decides(ctx) {
            return Err(Error::Forbidden("construction margin shows what subcontracts cost, which a Contracts Manager sees".into()));
        }
        let mut tx = read_tx(k, ctx).await?;
        #[allow(clippy::type_complexity)]
        let found: Vec<(Option<String>, Option<String>, Option<Decimal>, Option<Decimal>, Option<Decimal>, Decimal, Decimal, Decimal)> = sqlx::query_as(
            "select c.number, c.title, c.contract_sum, c.certified_to_date, c.retention_held, coalesce(sum(s.value), 0), \
             coalesce(sum(s.certified_to_date), 0), coalesce(sum(s.retention_held), 0) \
             from e_construction_contract c left join e_subcontract s on s.tenant_id = c.tenant_id and s.contract = c.id \
             where c.tenant_id = $1 group by c.id, c.number, c.title, c.contract_sum, c.certified_to_date, c.retention_held order by c.number nulls last limit 1000",
        )
        .bind(ctx.tenant)
        .fetch_all(&mut *tx)
        .await?;
        tx.commit().await?;
        let rows: Vec<Value> = found
            .into_iter()
            .map(|(number, title, sum, certified, held, sub_value, sub_certified, sub_held)| {
                let (sum, certified, held) = (sum.unwrap_or_default(), certified.unwrap_or_default(), held.unwrap_or_default());
                json!({"contract": number, "title": title, "contract_sum": money(sum), "certified": money(certified), "retention_held": money(held),
                       "subcontracted": money(sub_value), "subcontract_certified": money(sub_certified), "subcontract_retention": money(sub_held),
                       "margin": money(certified - sub_certified), "forecast_margin": money(sum - sub_value)})
            })
            .collect();
        Ok(json!({
            "columns": [col("contract", "Contract", "data"), col("title", "Title", "data"), col("contract_sum", "Contract Sum", "currency"),
                col("certified", "Certified", "currency"), col("retention_held", "Retention Held", "currency"), col("subcontracted", "Subcontracted", "currency"),
                col("subcontract_certified", "Subcontracts Certified", "currency"), col("subcontract_retention", "Subcontract Retention", "currency"),
                col("margin", "Margin", "currency"), col("forecast_margin", "Forecast Margin", "currency")],
            "rows": rows,
        }))
    }
}
