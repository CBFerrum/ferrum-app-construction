//! Site diaries: one to a project a day, written by whoever kept it.

use async_trait::async_trait;
use ferrum_domain::util::{id, s};
use ferrum_kernel::{Doc, DocHook, Error, HookCtx, Result};
use serde_json::{Value, json};
use uuid::Uuid;

pub struct DiaryHook;

#[async_trait]
impl DocHook for DiaryHook {
    fn name(&self) -> &'static str {
        "construction.diary"
    }

    fn fills(&self) -> &'static [&'static str] {
        &["written_by"]
    }

    async fn before_validate(&self, hc: &mut HookCtx<'_>, doc: &mut Doc) -> Result<()> {
        if doc.get("written_by").is_none_or(Value::is_null) && !hc.ctx.email.is_empty() {
            doc.insert("written_by".into(), json!(hc.ctx.email));
        }
        Ok(())
    }

    async fn validate(&self, hc: &mut HookCtx<'_>, doc: &Doc) -> Result<()> {
        if doc.get("workers").and_then(Value::as_i64).is_some_and(|w| w < 0) || doc.get("incidents").and_then(Value::as_i64).is_some_and(|w| w < 0) {
            return Err(Error::field("workers", "people and incidents are counted from zero"));
        }
        let (Some(project), Some(day)) = (id(doc, "project"), s(doc, "date")) else { return Ok(()) };
        let other: Option<Uuid> =
            sqlx::query_scalar("select id from e_site_diary where tenant_id = $1 and project = $2 and \"date\" = $3::date and id <> $4 limit 1")
                .bind(hc.ctx.tenant)
                .bind(project)
                .bind(day)
                .bind(id(doc, "id").unwrap_or_else(Uuid::nil))
                .fetch_optional(&mut **hc.tx)
                .await?;
        if other.is_some() {
            return Err(Error::field("date", "the project's diary for this day is written already: add to it"));
        }
        Ok(())
    }
}
