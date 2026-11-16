//! A building job (FERRUM-INDUSTRY-007) against a real PostgreSQL
//! (`FERRUM_CONSTRUCTION_TEST_DATABASE_URL`, default `postgres://ferrum:ferrum@localhost:5432/ferrum_construction_test`;
//! reset first): an estimate priced with a markup, sent and accepted into a contract worth its total;
//! variations submitted on site and approved, through the platform's approval, by a contracts manager
//! other than whoever raised them; progress claims measured line by line, submitted, certified the same
//! way less retention and earlier claims, and invoiced through ERP, a change while one waits starting
//! its approval again; certified work not taken back; a draft made stale by another claim recomputed
//! when certified; completing a contract certified in full, releasing its retention; the site diary;
//! and who may do what (the site measures, the contracts manager certifies and completes).

use ferrum_kernel::tenant::NewTenant;
use ferrum_kernel::{Ctx, Db, DbConfig, Doc, Error, HookRegistry, Kernel, KernelSettings, ListQuery};
use ferrum_meta::Catalog;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

fn obj(v: Value) -> Doc {
    v.as_object().cloned().unwrap()
}

fn uid(v: &Value) -> Uuid {
    v.as_str().unwrap_or_else(|| panic!("not an id: {v}")).parse().unwrap()
}

fn dec(v: &Value) -> f64 {
    v.as_str().map(|s| s.parse().unwrap()).or_else(|| v.as_f64()).unwrap_or_else(|| panic!("not a number: {v}"))
}

struct T {
    k: Arc<Kernel>,
    tenant: Uuid,
    admin: Ctx,
}

impl T {
    async fn insert(&self, ctx: &Ctx, entity: &str, v: Value) -> Doc {
        self.k.insert(ctx, entity, obj(v)).await.unwrap_or_else(|e| panic!("insert {entity}: {e:?}"))
    }
    async fn get(&self, entity: &str, id: Uuid) -> Doc {
        self.k.get(&self.admin, entity, id, false).await.unwrap_or_else(|e| panic!("get {entity}: {e:?}"))
    }
    async fn find(&self, entity: &str, filters: Value) -> Vec<Doc> {
        let q = ListQuery { filters: ferrum_kernel::query::parse_filters(&filters).unwrap(), limit: Some(50), ..Default::default() };
        self.k.list(&self.admin, entity, &q).await.unwrap().rows
    }
    async fn find_one(&self, entity: &str, filters: Value) -> Doc {
        let rows = self.find(entity, filters.clone()).await;
        assert_eq!(rows.len(), 1, "{entity} {filters}");
        rows[0].clone()
    }
    async fn user(&self, email: &str, roles: &[&str]) -> Ctx {
        let roles: Vec<String> = roles.iter().map(|r| r.to_string()).collect();
        let id = self.k.create_user(&self.admin, email, email, Some("Correct-Horse-9"), &roles, false).await.expect("user");
        let s = self.k.login("brick", email, "Correct-Horse-9").await.expect("login");
        self.k.load_ctx(self.tenant, id, Some(s.token_version), "it".into()).await.expect("ctx")
    }
    async fn act(&self, ctx: &Ctx, entity: &str, id: Uuid, action: &str) -> Result<Value, Error> {
        self.k.run_action(ctx, entity, id, action, Value::Null).await
    }
    /// Submit It by `by` (the approval asked in the name of whoever made the record), then approved by `decider`: the
    /// record as it then is.
    async fn passed(&self, by: &Ctx, decider: &Ctx, entity: &str, id: Uuid, submit: &str) -> Doc {
        self.act(by, entity, id, submit).await.unwrap_or_else(|e| panic!("{submit}: {e:?}"));
        self.k.decide_approval(decider, entity, id, true, None).await.unwrap_or_else(|e| panic!("decide {entity}: {e:?}"));
        self.get(entity, id).await
    }
    /// The invoice a result points at: (customer, total).
    async fn invoiced(&self, created: &Value) -> (String, f64) {
        let inv = self.get("SalesInvoice", uid(&created["id"])).await;
        (inv["customer"].as_str().unwrap().to_string(), dec(&inv["grand_total"]))
    }
}

fn refused<V: std::fmt::Debug>(r: Result<V, Error>, kind: &str, says: &str) {
    let msg = format!("{r:?}");
    let ok = matches!(
        (&r, kind),
        (Err(Error::Invalid(_) | Error::Validation(_)), "invalid") | (Err(Error::Forbidden(_)), "forbidden") | (Err(Error::NotFound(_)), "not found")
    );
    assert!(ok && msg.contains(says), "expected {kind} saying {says:?}, got {msg}");
}

#[test]
fn a_building_job() {
    let rt = tokio::runtime::Builder::new_multi_thread().enable_all().thread_stack_size(16 << 20).build().unwrap();
    rt.block_on(async { tokio::spawn(scenario()).await.unwrap() });
}

async fn scenario() {
    let url =
        std::env::var("FERRUM_CONSTRUCTION_TEST_DATABASE_URL").unwrap_or_else(|_| "postgres://ferrum:ferrum@localhost:5432/ferrum_construction_test".into());
    let db = Db::connect(DbConfig { url, max_connections: 16, ..Default::default() }).await.expect("connect");
    sqlx::raw_sql("drop schema if exists public cascade; create schema public;").execute(&db.pool).await.expect("reset");
    let mut packages = ferrum_domain::packages().expect("platform packages");
    packages.extend(ferrum_app_erp::packages().expect("ERP's packages"));
    packages.extend(ferrum_app_construction::packages().expect("app packages"));
    let catalog = Catalog::from_packages(packages).expect("catalog");
    let mut hooks = HookRegistry::default();
    ferrum_domain::register(&mut hooks);
    hooks.app("erp", &[], ferrum_app_erp::register);
    ferrum_app_construction::register(&mut hooks);
    for source in ferrum_app_construction::reports() {
        ferrum_domain::reports::add_source(source);
    }
    let k = Kernel::boot(db, catalog, hooks, KernelSettings::default(), None).await.expect("boot");
    ferrum_domain::maintenance(&k).await.expect("partitions");
    ferrum_app_erp::maintenance(&k).await.expect("partitions");
    let tenant = k
        .create_tenant(NewTenant {
            slug: "brick".into(),
            name: "Brick and Beam".into(),
            admin_email: "admin@brick.test".into(),
            admin_password: "Correct-Horse-9".into(),
            admin_name: None,
            packages: vec!["construction".into()],
            settings: Some(json!({"fiscal_year_start_month": 4, "timezone": "Asia/Kolkata"})),
        })
        .await
        .expect("tenant");
    let session = k.login("brick", "admin@brick.test", "Correct-Horse-9").await.expect("login");
    let admin = k.load_ctx(tenant, session.user, Some(session.token_version), "it".into()).await.expect("ctx");
    let t = T { k: k.clone(), tenant, admin };
    let a = t.admin.clone();

    // ───────────── the books ─────────────
    let inr = t.find_one("Currency", json!({"code": "INR"})).await;
    let company = uid(&t.insert(&a, "Company", json!({"company_name": "Brick and Beam", "abbr": "BB", "default_currency": inr["id"]})).await["id"]);
    let services = t.find_one("ItemGroup", json!({"group_name": "Services"})).await;
    let nos = t.find_one("Uom", json!({"uom_name": "Nos"})).await;
    let claim_item = uid(&t
        .insert(
            &a,
            "Item",
            json!({"item_code": "WORKS", "item_name": "Construction works", "item_group": services["id"], "stock_uom": nos["id"], "is_stock_item": false}),
        )
        .await["id"]);
    let acme = uid(&t.insert(&a, "Customer", json!({"customer_name": "Acme Developments", "customer_type": "Company"})).await["id"]);
    let cm = t.user("cm@brick.test", &["Contracts Manager"]).await;
    let sm = t.user("sm@brick.test", &["Site Manager"]).await;
    t.insert(&cm, "ConstructionSettings", json!({"company": company.to_string(), "claim_item": claim_item.to_string()})).await;
    let project =
        uid(&t.insert(&a, "Project", json!({"project_name": "Riverside Offices", "company": company.to_string(), "customer": acme.to_string()})).await["id"]);

    // ───────────── an estimate, sent and accepted ─────────────
    let estimate = |title: &str, lines: Value, markup: &str| json!({"title": title, "customer": acme.to_string(), "project": project.to_string(), "lines": lines, "markup_percent": markup});
    let three = json!([
        {"description": "Groundworks", "qty": "1", "unit": "lump sum", "rate": "40000"},
        {"description": "Steel frame", "qty": "200", "unit": "m²", "rate": "150"},
        {"description": "Roof", "qty": "1", "unit": "lump sum", "rate": "30000"}]);
    refused(t.k.insert(&cm, "Estimate", obj(estimate("Nothing", json!([]), "10"))).await, "invalid", "lines");
    refused(
        t.k.insert(&cm, "Estimate", obj(estimate("Free", json!([{"description": "x", "qty": "0", "rate": "5"}]), "10"))).await,
        "invalid",
        "quantity above zero",
    );
    refused(t.k.insert(&cm, "Estimate", obj(estimate("Cut", three.clone(), "-5"))).await, "invalid", "zero or more");
    let est = t.insert(&cm, "Estimate", estimate("Riverside Offices", three.clone(), "10")).await;
    let est_id = uid(&est["id"]);
    assert_eq!((dec(&est["subtotal"]), dec(&est["total"])), (100000.0, 110000.0));
    assert_eq!(dec(&est["lines"][1]["amount"]), 30000.0, "200 m² at 150");
    t.act(&cm, "Estimate", est_id, "send").await.unwrap();
    refused(t.k.update(&cm, "Estimate", est_id, obj(json!({"markup_percent": "15"})), None).await, "invalid", "keeps its prices");
    t.k.update(&cm, "Estimate", est_id, obj(json!({"notes": "valid for 30 days"})), None).await.expect("notes are not prices");
    let accepted = t.act(&cm, "Estimate", est_id, "accept").await.expect("accepted");
    let contract = uid(&accepted["contract"]["id"]);
    refused(t.act(&cm, "Estimate", est_id, "accept").await, "invalid", "not available");
    let c = t.get("ConstructionContract", contract).await;
    let schedule: Vec<(String, f64)> = c["lines"].as_array().unwrap().iter().map(|l| (l["code"].as_str().unwrap().into(), dec(&l["value"]))).collect();
    assert_eq!(schedule, [("1".into(), 44000.0), ("2".into(), 33000.0), ("3".into(), 33000.0)], "the markup spread across the lines");
    assert_eq!((dec(&c["contract_sum"]), dec(&c["original_sum"]), dec(&c["retention_percent"])), (110000.0, 110000.0, 5.0));
    assert_eq!(t.get("Estimate", est_id).await["contract"], contract.to_string());
    let other = uid(&t.insert(&cm, "Estimate", estimate("Car park", json!([{"description": "Tarmac", "qty": "1", "rate": "9000"}]), "0")).await["id"]);
    t.act(&cm, "Estimate", other, "send").await.unwrap();
    t.act(&cm, "Estimate", other, "decline").await.unwrap();
    assert_eq!(t.get("Estimate", other).await["status"], "Declined");
    // A markup that does not divide evenly: the last line takes the cent, and the contract is worth
    // the estimate's total.
    let ones = json!([{"description": "Sign A", "qty": "1", "rate": "1"}, {"description": "Sign B", "qty": "1", "rate": "1"}, {"description": "Sign C", "qty": "1", "rate": "1"}]);
    let signs = t.insert(&cm, "Estimate", estimate("Signs", ones, "12.5")).await;
    assert_eq!(dec(&signs["total"]), 3.38);
    t.act(&cm, "Estimate", uid(&signs["id"]), "send").await.unwrap();
    let signed = t.act(&cm, "Estimate", uid(&signs["id"]), "accept").await.unwrap();
    let sc = t.get("ConstructionContract", uid(&signed["contract"]["id"])).await;
    let values: Vec<f64> = sc["lines"].as_array().unwrap().iter().map(|l| dec(&l["value"])).collect();
    assert_eq!((values, dec(&sc["contract_sum"])), (vec![1.13, 1.13, 1.12], 3.38));

    // ───────────── variations ─────────────
    let variation = |desc: &str, amount: &str| json!({"contract": contract.to_string(), "description": desc, "amount": amount, "reason": "Site Condition"});
    refused(t.k.insert(&sm, "Variation", obj(variation("Nothing", "0"))).await, "invalid", "adds or takes out");
    let drain = uid(&t.insert(&sm, "Variation", variation("Extra drainage", "5500")).await["id"]);
    let asked = t.act(&sm, "Variation", drain, "submit_variation").await.unwrap();
    assert_eq!(asked["approval"]["steps"][0]["label"], "Contracts Manager", "asked of a Contracts Manager: {asked}");
    refused(t.k.decide_approval(&sm, "Variation", drain, true, None).await, "forbidden", "");
    refused(t.k.update(&sm, "Variation", drain, obj(json!({"amount": "9000"})), None).await, "invalid", "keeps its terms");
    t.k.decide_approval(&cm, "Variation", drain, true, None).await.expect("approved");
    let drain_doc = t.get("Variation", drain).await;
    assert_eq!(drain_doc["status"], "Approved");
    let record = t.k.approval_of(&t.admin, "Variation", drain).await.unwrap();
    assert_eq!(record["decisions"][0]["decider"], json!(cm.user.to_string()), "who approved it is the request's record");
    let vcode = format!("V-{}", drain_doc["number"].as_str().unwrap());
    let c = t.get("ConstructionContract", contract).await;
    assert_eq!((dec(&c["contract_sum"]), dec(&c["original_sum"]), dec(&c["variations_sum"])), (115500.0, 110000.0, 5500.0));
    assert_eq!(c["lines"].as_array().unwrap().last().unwrap()["code"], vcode.as_str());
    // What someone raises, someone else approves (FEO-049): asked in its raiser's name, even when another submits it.
    let mine = uid(&t.insert(&cm, "Variation", variation("Own idea", "100")).await["id"]);
    let cm2 = t.user("cm2@brick.test", &["Contracts Manager"]).await;
    let theirs = t.act(&cm2, "Variation", mine, "submit_variation").await.unwrap();
    assert_eq!(theirs["approval"]["requester"], json!(cm.user.to_string()), "asked in the name of whoever raised it: {theirs}");
    // The platform leaves the requester out of who may decide: a Contracts Manager, but not this one.
    refused(t.k.decide_approval(&cm, "Variation", mine, true, None).await, "forbidden", "not yours to decide");
    t.k.recall_approval(&cm, "Variation", mine).await.expect("whoever raised it takes it back");
    // Asked again directly, before upkeep, by someone else: refused, else its raiser could decide it.
    refused(t.act(&cm2, "Variation", mine, "approve").await, "invalid", "in the name of whoever raised it");
    ferrum_app_construction::maintenance(&k).await.unwrap();
    assert_eq!(t.get("Variation", mine).await["status"], "Draft", "back to its draft");
    let paint = uid(&t.insert(&sm, "Variation", variation("Gold paint", "20000")).await["id"]);
    t.act(&sm, "Variation", paint, "submit_variation").await.unwrap();
    refused(t.k.decide_approval(&cm, "Variation", paint, false, None).await, "invalid", "a reason is needed");
    t.k.decide_approval(&cm, "Variation", paint, false, Some("Not in the budget")).await.unwrap();
    ferrum_app_construction::maintenance(&k).await.unwrap();
    assert_eq!(t.get("Variation", paint).await["status"], "Rejected");
    assert_eq!(dec(&t.get("ConstructionContract", contract).await["contract_sum"]), 115500.0, "a rejected variation changes nothing");

    // ───────────── progress claims ─────────────
    let claim = |to: &str, lines: Value| json!({"contract": contract.to_string(), "period_to": to, "lines": lines});
    let pc1 =
        t.insert(&sm, "ProgressClaim", claim("2026-03-31", json!([{"code": "1", "percent_complete": "100"}, {"code": "2", "percent_complete": "50"}]))).await;
    assert_eq!((dec(&pc1["value_to_date"]), dec(&pc1["retention"]), dec(&pc1["amount_due"])), (60500.0, 3025.0, 57475.0));
    assert_eq!(pc1["lines"].as_array().unwrap().len(), 4, "every line of the contract, the variation's too");
    let pc1 = uid(&pc1["id"]);
    t.act(&sm, "ProgressClaim", pc1, "submit_claim").await.expect("asked");
    refused(t.k.decide_approval(&sm, "ProgressClaim", pc1, true, None).await, "forbidden", "");
    // What someone draws up, someone else certifies; what someone raises, someone else approves (FEO-049).
    let own = uid(&t.insert(&cm, "ProgressClaim", claim("2026-03-31", json!([{"code": "1", "percent_complete": "10"}]))).await["id"]);
    refused(t.act(&sm, "ProgressClaim", own, "certify").await, "invalid", "in the name of whoever drew it up");
    t.act(&cm, "ProgressClaim", own, "submit_claim").await.expect("asked");
    refused(t.k.decide_approval(&cm, "ProgressClaim", own, true, None).await, "forbidden", "not yours to decide");
    t.k.recall_approval(&cm, "ProgressClaim", own).await.expect("taken back");
    t.k.delete(&a, "ProgressClaim", own, None).await.expect("a draft goes");
    t.k.decide_approval(&cm, "ProgressClaim", pc1, true, None).await.expect("certified");
    let certified = t.get("ProgressClaim", pc1).await;
    assert_eq!(certified["status"], "Certified");
    assert_eq!(t.invoiced(&json!({"id": certified["sales_invoice"]})).await, (acme.to_string(), 57475.0));
    let c = t.get("ConstructionContract", contract).await;
    assert_eq!((dec(&c["certified_to_date"]), dec(&c["retention_held"])), (60500.0, 3025.0));
    refused(t.k.update(&cm, "ProgressClaim", pc1, obj(json!({"period_to": "2026-04-01"})), None).await, "invalid", "not changed");
    let mut lines = c["lines"].as_array().unwrap().clone();
    lines[0]["value"] = json!("50000");
    refused(t.k.update(&cm, "ConstructionContract", contract, obj(json!({"lines": lines})), None).await, "invalid", "through variations");
    t.k.update(&cm, "ConstructionContract", contract, obj(json!({"title": "Riverside Offices, phase 1"})), None).await.expect("its title is not its terms");
    // Certified work is not taken back, claims follow one another, and lines are the contract's.
    refused(
        t.k.insert(&cm, "ProgressClaim", obj(claim("2026-03-15", json!([{"code": "2", "percent_complete": "80"}])))).await,
        "invalid",
        "is after 2026-03-31",
    );
    refused(t.k.insert(&cm, "ProgressClaim", obj(claim("2026-04-30", json!([{"code": "2", "percent_complete": "40"}])))).await, "invalid", "certified at 50");
    refused(
        t.k.insert(&cm, "ProgressClaim", obj(claim("2026-04-30", json!([{"code": "9", "percent_complete": "10"}])))).await,
        "invalid",
        "line 9 is not on the contract",
    );
    refused(t.k.insert(&cm, "ProgressClaim", obj(claim("2026-04-30", json!([{"code": "3", "percent_complete": "120"}])))).await, "invalid", "0% to 100%");
    let second = json!([{"code": "2", "percent_complete": "100"}, {"code": "3", "percent_complete": "50"}, {"code": vcode, "percent_complete": "100"}]);
    let pc2 = t.insert(&sm, "ProgressClaim", claim("2026-04-30", second)).await;
    assert_eq!(
        (dec(&pc2["value_to_date"]), dec(&pc2["retention"]), dec(&pc2["previous_certified"]), dec(&pc2["amount_due"])),
        (99000.0, 4950.0, 57475.0, 36575.0),
        "line 1 carried at 100%"
    );
    let pc2 = uid(&pc2["id"]);
    // A draft drawn up before the claim ahead of it is certified, is recomputed when it is.
    let pc3 = t.insert(&sm, "ProgressClaim", claim("2026-05-31", json!([{"code": "3", "percent_complete": "100"}]))).await;
    assert_eq!(dec(&pc3["previous_certified"]), 57475.0, "drafted while the April claim was a draft");
    let pc3 = uid(&pc3["id"]);
    // A change to what a waiting claim measures starts its approval again.
    t.act(&sm, "ProgressClaim", pc2, "submit_claim").await.expect("asked");
    t.k.update(&sm, "ProgressClaim", pc2, obj(json!({"period_to": "2026-04-29"})), None).await.expect("changed while it waits");
    let mut tx = k.db.tenant_tx(tenant).await.unwrap();
    let resets: i64 = sqlx::query_scalar("select count(*) from audit_event where tenant_id = $1 and record_id = $2 and action = 'approval:reset'")
        .bind(tenant)
        .bind(pc2)
        .fetch_one(&mut *tx)
        .await
        .unwrap();
    tx.commit().await.unwrap();
    assert_eq!(resets, 1, "its approval started again");
    t.k.decide_approval(&cm, "ProgressClaim", pc2, true, None).await.expect("certified");
    let c2 = t.get("ProgressClaim", pc2).await;
    assert_eq!(t.invoiced(&json!({"id": c2["sales_invoice"]})).await.1, 36575.0);
    let c3 = t.passed(&sm, &cm, "ProgressClaim", pc3, "submit_claim").await;
    assert_eq!(dec(&c3["amount_due"]), 15675.0, "115500 less 5775 held less the 94050 certified before");
    assert_eq!(t.invoiced(&json!({"id": c3["sales_invoice"]})).await.1, 15675.0);
    assert_eq!(dec(&c3["previous_certified"]), 94050.0, "what is kept is what was certified");
    let pc4 = uid(&t.insert(&sm, "ProgressClaim", claim("2026-06-30", json!([]))).await["id"]);
    refused(t.act(&sm, "ProgressClaim", pc4, "submit_claim").await, "invalid", "adds no work");
    // A variation that would make the contract worth less than what is certified is refused.
    let cut = uid(&t.insert(&sm, "Variation", variation("Smaller roof", "-500")).await["id"]);
    refused(t.act(&sm, "Variation", cut, "submit_variation").await, "invalid", "less than the 115500");

    // ───────────── a subcontract and its claims (FEO-049) ─────────────
    let steelco = uid(&t.insert(&a, "Supplier", json!({"supplier_name": "Steel Co"})).await["id"]);
    let steel = uid(&t
        .insert(&cm, "Subcontract", json!({"title": "Steel frame", "contract": contract.to_string(), "supplier": steelco.to_string(), "value": "30000"}))
        .await["id"]);
    assert_eq!(dec(&t.get("Subcontract", steel).await["retention_percent"]), 5.0, "the settings' retention");
    let sub_claim = |to: &str, value: &str| json!({"subcontract": steel.to_string(), "period_to": to, "value_to_date": value});
    refused(t.k.insert(&sm, "SubcontractClaim", obj(sub_claim("2026-03-31", "31000"))).await, "invalid", "to the subcontract's 30000");
    let sc1 = t.insert(&sm, "SubcontractClaim", sub_claim("2026-03-31", "12000")).await;
    assert_eq!((dec(&sc1["retention"]), dec(&sc1["amount_due"])), (600.0, 11400.0));
    let sc1 = uid(&sc1["id"]);
    t.act(&sm, "SubcontractClaim", sc1, "submit_claim").await.expect("asked");
    refused(t.k.decide_approval(&sm, "SubcontractClaim", sc1, true, None).await, "forbidden", "");
    t.k.decide_approval(&cm, "SubcontractClaim", sc1, true, None).await.expect("certified");
    let billed = t.get("SubcontractClaim", sc1).await;
    let bill = t.get("PurchaseInvoice", uid(&billed["purchase_invoice"])).await;
    assert_eq!((bill["supplier"].as_str(), dec(&bill["net_total"])), (Some(steelco.to_string().as_str()), 11400.0), "billed through ERP, less retention");
    refused(t.k.insert(&sm, "SubcontractClaim", obj(sub_claim("2026-04-30", "10000"))).await, "invalid", "work certified is not taken back");
    let sc2 = uid(&t.insert(&sm, "SubcontractClaim", sub_claim("2026-04-30", "30000")).await["id"]);
    let done_sub = t.passed(&sm, &cm, "SubcontractClaim", sc2, "submit_claim").await;
    assert_eq!(dec(&done_sub["amount_due"]), 17100.0, "30000 less 1500 held less the 11400 before");
    let sub = t.get("Subcontract", steel).await;
    assert_eq!((dec(&sub["certified_to_date"]), dec(&sub["retention_held"])), (30000.0, 1500.0));
    // The margin: certified to the client less certified to the subcontractor; forecast, the sums.
    let margin = ferrum_domain::reports::run(&t.k, &cm, "construction_margin", &HashMap::new()).await.expect("margin");
    let number = t.get("ConstructionContract", contract).await["number"].clone();
    let row = margin["rows"].as_array().unwrap().iter().find(|r| r["contract"] == number).cloned().unwrap();
    assert_eq!(
        (dec(&row["certified"]), dec(&row["subcontract_certified"]), dec(&row["margin"]), dec(&row["forecast_margin"])),
        (115500.0, 30000.0, 85500.0, 85500.0)
    );
    assert!(matches!(ferrum_domain::reports::run(&t.k, &sm, "construction_margin", &HashMap::new()).await, Err(Error::Forbidden(_))), "what subcontracts cost");

    // ───────────── completing ─────────────
    let small = uid(&t
        .insert(
            &cm,
            "ConstructionContract",
            json!({"title": "Fence", "customer": acme.to_string(), "retention_percent": "10", "retention_cap_percent": "4", "lines": [{"code": "1", "description": "Fence", "value": "10000"}]}),
        )
        .await["id"]);
    assert_eq!(dec(&t.get("ConstructionContract", small).await["retention_percent"]), 10.0, "a contract's own retention");
    let half = uid(&t
        .insert(&sm, "ProgressClaim", json!({"contract": small.to_string(), "period_to": "2026-04-30", "lines": [{"code": "1", "percent_complete": "50"}]}))
        .await["id"]);
    assert_eq!(dec(&t.get("ProgressClaim", half).await["retention"]), 400.0, "10% of 5000 is 500, capped at 4% of the 10000");
    t.passed(&sm, &cm, "ProgressClaim", half, "submit_claim").await;
    refused(t.act(&cm, "ConstructionContract", small, "complete").await, "invalid", "certified 5000");
    refused(t.act(&sm, "ConstructionContract", contract, "complete").await, "forbidden", "");
    // Even given the right to change contracts, a Site Manager does not complete one.
    let write: ferrum_meta::PermissionDef = serde_json::from_value(json!({"entity": "ConstructionContract", "read": true, "write": true})).unwrap();
    t.k.set_role_permission(&a, "Site Manager", &write).await.unwrap();
    let sm = t.k.load_ctx(t.tenant, sm.user, None, "it".into()).await.unwrap();
    refused(t.act(&sm, "ConstructionContract", contract, "complete").await, "forbidden", "a Contracts Manager completes");
    refused(t.act(&sm, "ConstructionContract", contract, "practical_completion").await, "forbidden", "a Contracts Manager");
    let practical = t.act(&cm, "ConstructionContract", contract, "practical_completion").await.expect("practically complete");
    assert_eq!(t.invoiced(&practical["invoice"]).await, (acme.to_string(), 2887.5), "half of the 5775 held");
    assert_eq!(dec(&t.get("ConstructionContract", contract).await["retention_held"]), 2887.5);
    refused(t.act(&cm, "ConstructionContract", contract, "practical_completion").await, "invalid", "not available");
    let done = t.act(&cm, "ConstructionContract", contract, "complete").await.expect("complete");
    assert_eq!(t.invoiced(&done["retention_invoice"]).await, (acme.to_string(), 2887.5), "the rest of the retention released");
    assert_eq!(dec(&t.get("ConstructionContract", contract).await["retention_released"]), 5775.0);
    let c = t.get("ConstructionContract", contract).await;
    assert_eq!((c["status"].as_str(), dec(&c["retention_held"])), (Some("Complete"), 0.0));
    refused(t.act(&cm, "ConstructionContract", contract, "complete").await, "invalid", "not available");
    refused(t.k.insert(&cm, "ProgressClaim", obj(claim("2026-07-31", json!([])))).await, "invalid", "takes no more claims");
    refused(t.k.insert(&sm, "Variation", obj(variation("Late extra", "100"))).await, "invalid", "takes no more variations");
    refused(t.k.update(&cm, "ConstructionContract", contract, obj(json!({"title": "Renamed"})), None).await, "invalid", "complete contract");

    // ───────────── the site diary ─────────────
    let diary =
        |day: &str, workers: i64| json!({"project": project.to_string(), "date": day, "weather": "Rain", "workers": workers, "work_done": "Footings poured"});
    let d1 = t.insert(&sm, "SiteDiary", diary("2026-03-02", 14)).await;
    assert_eq!(d1["written_by"], "sm@brick.test");
    refused(t.k.insert(&sm, "SiteDiary", obj(diary("2026-03-02", 3))).await, "invalid", "written already");
    t.insert(&sm, "SiteDiary", diary("2026-03-03", 12)).await;
    t.k.update(&sm, "SiteDiary", uid(&d1["id"]), obj(json!({"delays": "Concrete late by two hours"})), None).await.expect("added to");
    refused(t.k.insert(&sm, "SiteDiary", obj(diary("2026-03-04", -1))).await, "invalid", "from zero");

    // ───────────── who may do what ─────────────
    refused(t.k.insert(&sm, "Estimate", obj(estimate("Mine", three.clone(), "0"))).await, "forbidden", "");
    refused(t.k.insert(&sm, "ConstructionContract", obj(json!({"title": "x", "customer": acme.to_string(), "lines": []}))).await, "forbidden", "");
}
