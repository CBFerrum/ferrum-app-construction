//! Construction and professional services (FERRUM-INDUSTRY-007): estimates, contracts with
//! variations, progress claims with retention invoiced through ERP, and site diaries.
//!
//! * `estimates`: priced lines with a markup; accepted, an estimate becomes a contract whose
//!   schedule of values is its lines.
//! * `contracts`: a contract's lines change through approved variations once work is certified or
//!   varied; completing it releases the retention held.
//! * `claims`: how far each line has got; certified, what is due after retention and earlier
//!   claims is invoiced to the customer.
//! * `diary`: a day on site, one to a project.
//!
//! This app's own design; no code was copied (docs/PROVENANCE.md). Hooks are registered per entity,
//! so a workspace without the `construction` package never runs them.

mod claims;
mod contracts;
mod diary;
mod estimates;

use ferrum_domain::reports::ReportSource;
use ferrum_domain::util::{d, s};
use ferrum_kernel::{Ctx, Doc, Error, HookRegistry, Kernel, Result, Tx};
use rust_decimal::Decimal;
use serde_json::Value;
use std::sync::Arc;
use uuid::Uuid;

include!(concat!(env!("OUT_DIR"), "/packages.rs"));
include!(concat!(env!("OUT_DIR"), "/ui.rs"));

/// This app's manifest (`app.yaml`).
pub const MANIFEST: &str = include_str!("../app.yaml");

/// This app's metadata packages, embedded at build time.
pub fn packages() -> std::result::Result<Vec<ferrum_meta::Package>, ferrum_meta::MetaError> {
    ferrum_meta::catalog::load_embedded(EMBEDDED)
}

/// Registers this app's document hooks.
pub fn register(hooks: &mut HookRegistry) {
    hooks
        .register("Estimate", Arc::new(estimates::EstimateHook))
        .register("ConstructionContract", Arc::new(contracts::ContractHook))
        .register("Variation", Arc::new(contracts::VariationHook))
        .register("ProgressClaim", Arc::new(claims::ClaimHook))
        .register("SiteDiary", Arc::new(diary::DiaryHook));
}

/// This app's reports: none of its own yet; what is billed is ERP's receivables.
pub fn reports() -> Vec<Arc<dyn ReportSource>> {
    vec![]
}

/// Upkeep: nothing runs by itself.
pub async fn maintenance(_k: &Kernel) -> Result<()> {
    Ok(())
}

/// A document from JSON.
pub(crate) fn obj(v: Value) -> Doc {
    v.as_object().cloned().unwrap_or_default()
}

/// Where claims are invoiced from: the company, the item claims are invoiced as, and the
/// retention held when a contract names none.
pub(crate) async fn settings(tx: &mut Tx, tenant: Uuid) -> Result<(Uuid, Uuid, Decimal)> {
    let row: Option<(Option<Uuid>, Option<Uuid>, Option<Decimal>)> =
        sqlx::query_as("select company, claim_item, retention_percent from e_construction_settings where tenant_id = $1 limit 1")
            .bind(tenant)
            .fetch_optional(&mut **tx)
            .await?;
    match row {
        Some((Some(company), Some(item), retention)) => Ok((company, item, retention.unwrap_or_default())),
        _ => Err(Error::invalid("set the company and the claim item in Construction Settings first")),
    }
}

/// A draft Sales Invoice of one line to `customer`.
pub(crate) async fn invoice(k: &Kernel, ctx: &Ctx, tx: &mut Tx, customer: &str, amount: Decimal, what: String) -> Result<Doc> {
    let (company, item, _) = settings(tx, ctx.tenant).await?;
    let currency: Option<Uuid> = sqlx::query_scalar("select default_currency from e_company where tenant_id = $1 and id = $2")
        .bind(ctx.tenant)
        .bind(company)
        .fetch_optional(&mut **tx)
        .await?
        .flatten();
    let line = serde_json::json!({"item": item.to_string(), "qty": "1", "price_list_rate": ferrum_domain::util::money(amount), "discount_percent": "0", "description": what});
    k.insert_tx(
        ctx,
        tx,
        "SalesInvoice",
        obj(serde_json::json!({"company": company.to_string(), "customer": customer, "currency": currency.map(|c| c.to_string()), "items": [line], "remarks": what})),
    )
    .await
}

/// Whether `ctx` decides for the contracts: a Contracts Manager, or an administrator.
pub(crate) fn decides(ctx: &Ctx) -> bool {
    ctx.is_admin || ctx.roles.iter().any(|r| r == "Contracts Manager")
}

/// A status, for messages.
pub(crate) fn status(doc: &Doc) -> &str {
    s(doc, "status").unwrap_or("Draft")
}

/// An amount, for sums of child rows.
pub(crate) fn sum(lines: &[Doc], k: &str) -> Decimal {
    lines.iter().map(|l| d(l, k)).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_interface_is_a_release_the_platform_takes() {
        let m = ferrum_apps::AppManifest::parse(MANIFEST).expect("app.yaml");
        assert_eq!(m.routes(), vec![("construction".to_string(), vec![])]);
        let files = UI.iter().map(|(p, b)| (p.to_string(), b.to_vec())).collect();
        let ui = ferrum_apps::ui::prepare("construction", files).expect("ui/dist is an interface release");
        assert!(UI.iter().any(|(p, _)| *p == "index.html") && !UI.iter().any(|(p, _)| p.starts_with('.')), "{}", ui.digest);
    }
}
