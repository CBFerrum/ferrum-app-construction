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
//! A variation's approval and a claim's certification are the platform's approval policies (package.yaml,
//! `approvals`): Submit It asks, in the name of whoever made the record ([`ask`]), and the hidden action does the
//! work once a Contracts Manager other than them approves.
//!
//! This app's own design; no code was copied (docs/PROVENANCE.md). Hooks are registered per entity,
//! so a workspace without the `construction` package never runs them.

mod claims;
mod contracts;
mod diary;
mod estimates;
mod reports;
mod subcontracts;

use ferrum_domain::reports::ReportSource;
use ferrum_domain::util::{d, s};
use ferrum_kernel::{Ctx, Doc, Error, HookCtx, HookRegistry, Kernel, Result, Tx};
use rust_decimal::Decimal;
use serde_json::{Value, json};
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
        .register("Subcontract", Arc::new(subcontracts::SubcontractHook))
        .register("SubcontractClaim", Arc::new(subcontracts::SubcontractClaimHook))
        .register("SiteDiary", Arc::new(diary::DiaryHook));
}

/// This app's reports: none of its own yet; what is billed is ERP's receivables.
pub fn reports() -> Vec<Arc<dyn ReportSource>> {
    vec![Arc::new(reports::Margin)]
}

/// Upkeep: variations whose approval ended without approving them moved on.
pub async fn maintenance(k: &Kernel, tenant: Uuid) -> Result<()> {
    let ctx = Ctx::system(tenant, k.registry.schema(tenant).await?);
    let mut tx = k.db.tenant_tx(tenant).await?;
    contracts::follow_approvals(k, &ctx, &mut tx).await?;
    tx.commit().await?;
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

/// Asks for the approval `action` of record `id` (`doc`) carries, in the name of whoever made the record: the policy
/// never lets whoever asked decide, so whoever raised or drew it up does not (FEO-049). The action's condition admits
/// no one else as the asker, so asking it directly changes nothing. The answer: the record's status while it waits
/// (`waiting`) and the request, or the action's own when the workspace's policy asked no one.
pub(crate) async fn ask(hc: &mut HookCtx<'_>, entity: &str, id: Uuid, doc: &Doc, action: &str, waiting: &str) -> Result<Value> {
    let maker = doc.get("created_by").and_then(Value::as_str).and_then(|u| u.parse::<Uuid>().ok()).filter(|u| *u != hc.ctx.user);
    let who = match maker {
        Some(u) => hc
            .kernel
            .load_ctx(hc.ctx.tenant, u, None, hc.ctx.request_id.clone())
            .await
            .map_err(|_| Error::invalid("whoever made it is no longer here to ask for its approval: make it again"))?,
        None => hc.ctx.clone(),
    };
    let out = hc.kernel.run_action_tx(&who, hc.tx, entity, id, action, &json!({})).await?;
    Ok(match out.get("_approval") {
        Some(asked) => json!({"status": waiting, "approval": asked}),
        None => out,
    })
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
