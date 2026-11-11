// The contracts: each one's worth, how much of it is certified, the retention held, and what waits
// on someone (variations submitted, claims in draft), with the estimates sent and not yet answered.
// Each opens its record. Before claims can be invoiced, it says what to set up.

import { useQuery } from "@tanstack/react-query";
import { Link } from "react-router";
import { api, docPath, formatDecimal, t, isolate } from "@ferrum/client";
import { Badge, EmptyState, ErrorState, ListSkeleton, statusTone, useShellTitle } from "@ferrum/kit";

// ConstructionContract.status as stored (packages/construction/entities).
const CONTRACT_TONES = { Active: "green", Complete: "neutral" } as const;

interface Row {
  id: string;
  [k: string]: unknown;
}

const rows = (entity: string, filters: unknown[], extra: Record<string, string | number> = {}) =>
  api.get<{ rows: Row[] }>(docPath(entity), { filters: JSON.stringify(filters), limit: 500, ...extra }).then((r) => r.rows);

const num = (v: unknown) => Number(v ?? 0) || 0;

/** An amount as the books write it: grouped, to the cent ("22,180.00"). */
export const money = (v: unknown) => formatDecimal(String(v ?? "0") || "0", 2);

/** How far a contract is certified, as a whole percentage of its worth. */
export function certified(contract: Row): number {
  const worth = num(contract.contract_sum);
  return worth > 0 ? Math.round((num(contract.certified_to_date) / worth) * 100) : 0;
}

/** Records waiting on someone, by contract id. */
export function waiting(variations: Row[], claims: Row[]): Map<string, number> {
  const out = new Map<string, number>();
  for (const r of [...variations, ...claims]) {
    const c = String(r.contract ?? "");
    out.set(c, (out.get(c) ?? 0) + 1);
  }
  return out;
}

export function Home() {
  useShellTitle(t("Construction"));
  const q = useQuery({
    queryKey: ["construction", "home"],
    queryFn: async () => {
      const [contracts, variations, claims, estimates, settings] = await Promise.all([
        rows("ConstructionContract", [], { sort: "created_at desc" }),
        rows("Variation", [["status", "eq", "Submitted"]]),
        rows("ProgressClaim", [["status", "eq", "Draft"]]),
        rows("Estimate", [["status", "eq", "Sent"]], { sort: "created_at desc" }),
        rows("ConstructionSettings", [], { fields: "id", limit: 1 }),
      ]);
      return { contracts, variations, claims, estimates, named: settings.length > 0 };
    },
  });
  if (q.isPending) return <ListSkeleton className="np-page" />;
  if (q.isError) return <ErrorState error={q.error} onRetry={() => void q.refetch()} />;
  const pending = waiting(q.data.variations, q.data.claims);
  return (
    <div className="np-page">
      <header className="row" style={{ justifyContent: "space-between" }}>
        <h2>{t("Contracts")}</h2>
        <Link to="/e/Estimate/new">{t("New estimate")}</Link>
      </header>
      {!q.data.named && (
        <EmptyState title={t("Set up the books first")} icon="settings">
          <p className="muted">
            {t("Progress claims and released retention are invoiced through ERP: set the company, the item they are invoiced as and the retention held in")}{isolate(" ")}
            <Link to="/e/ConstructionSettings">{t("Construction Settings")}</Link>.
          </p>
        </EmptyState>
      )}
      {q.data.estimates.length > 0 && (
        <section className="card card-pad stack" aria-label={t("Estimates waiting")}>
          <h3>{t("Estimates sent, waiting for an answer ({length})", { length: q.data.estimates.length })}</h3>
          <ul className="stack">
            {q.data.estimates.map((e) => (
              <li key={e.id}>
                <Link to={`/e/Estimate/${e.id}`}>
                  {isolate(String(e.number ?? ""))} · {isolate(String(e.title))} · {isolate(money(e.total))}
                </Link>
              </li>
            ))}
          </ul>
        </section>
      )}
      {q.data.named && q.data.contracts.length === 0 && q.data.estimates.length === 0 && (
        <p className="muted">{t("No contracts yet. An estimate the customer accepts becomes one.")}</p>
      )}
      <div className="np-columns">
        {q.data.contracts.map((c) => (
          <Link key={c.id} className="card card-pad stack" to={`/e/ConstructionContract/${c.id}`}>
            <strong>{isolate(String(c.title))}</strong>
            <span className="faint">{isolate(String(c.number ?? ""))}</span>
            <span>
              {t("Worth")} {isolate(money(c.contract_sum))} {t("· certified")} {certified(c)}{t("% · held")} {isolate(money(c.retention_held))}
            </span>
            <span className="row">
              <Badge tone={statusTone(String(c.status), CONTRACT_TONES)}>{isolate(String(c.status))}</Badge>
              {(pending.get(c.id) ?? 0) > 0 && <Badge tone="amber">{t("{pending} waiting", { pending: String(pending.get(c.id)) })}</Badge>}
            </span>
          </Link>
        ))}
      </div>
    </div>
  );
}
