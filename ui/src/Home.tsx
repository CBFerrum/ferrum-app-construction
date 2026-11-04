// The contracts: each one's worth, how much of it is certified, the retention held, and what waits
// on someone (variations submitted, claims in draft). Each opens its record.

import { useQuery } from "@tanstack/react-query";
import { Link } from "react-router";
import { api, docPath } from "@ferrum/client";
import { Badge, ErrorState, PageLoading, statusTone, useShellTitle } from "@ferrum/kit";

interface Row {
  id: string;
  [k: string]: unknown;
}

const rows = (entity: string, filters: unknown[], extra: Record<string, string | number> = {}) =>
  api.get<{ rows: Row[] }>(docPath(entity), { filters: JSON.stringify(filters), limit: 500, ...extra }).then((r) => r.rows);

const num = (v: unknown) => Number(v ?? 0) || 0;

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
  useShellTitle("Construction");
  const q = useQuery({
    queryKey: ["construction", "home"],
    queryFn: async () => {
      const [contracts, variations, claims] = await Promise.all([
        rows("ConstructionContract", [], { sort: "created_at desc" }),
        rows("Variation", [["status", "eq", "Submitted"]]),
        rows("ProgressClaim", [["status", "eq", "Draft"]]),
      ]);
      return { contracts, variations, claims };
    },
  });
  if (q.isPending) return <PageLoading />;
  if (q.isError) return <ErrorState error={q.error} onRetry={() => void q.refetch()} />;
  const pending = waiting(q.data.variations, q.data.claims);
  return (
    <div className="np-page">
      <header className="row" style={{ justifyContent: "space-between" }}>
        <h2>Contracts</h2>
        <Link to="/e/Estimate/new">New estimate</Link>
      </header>
      <div className="np-columns">
        {q.data.contracts.map((c) => (
          <Link key={c.id} className="card card-pad stack" to={`/e/ConstructionContract/${c.id}`}>
            <strong>{String(c.title)}</strong>
            <span className="faint">{String(c.number ?? "")}</span>
            <span>
              Worth {String(c.contract_sum ?? "0")} · certified {certified(c)}% · held {String(c.retention_held ?? "0")}
            </span>
            <span className="row">
              <Badge tone={statusTone(String(c.status))}>{String(c.status)}</Badge>
              {(pending.get(c.id) ?? 0) > 0 && <Badge tone="amber">{`${pending.get(c.id)} waiting`}</Badge>}
            </span>
          </Link>
        ))}
      </div>
    </div>
  );
}
