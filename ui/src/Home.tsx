// The contracts: each one's worth, how much of it is certified, the retention held, and what waits
// on someone (variations submitted, claims in draft), with the estimates sent and not yet answered.
// Each opens its record. Before claims can be invoiced, it says what to set up.

import { useQuery } from "@tanstack/react-query";
import { Link } from "react-router";
import { api, docPath, formatDecimal, t, isolate } from "@ferrum/client";
import { HardHat, Settings } from "lucide-react";
import { Alert, Badge, ButtonLink, HomeSkeleton, LoadFailed, PageHead, Queue, QueueGrid, QueueItem, StatePanel, statusTone, Tile, TileGrid, useShellTitle } from "@ferrum/kit";

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
  const head = (
    <PageHead
      title={t("Contracts")}
      actions={
        <ButtonLink to="/e/Estimate/new" variant="primary" icon="plus">
          {t("New estimate")}
        </ButtonLink>
      }
    />
  );
  if (q.isPending) return (
    <>
      <PageHead title={t("Contracts")} />
      <HomeSkeleton />
    </>
  );
  if (q.isError) return <>{head}<LoadFailed error={q.error} onRetry={() => void q.refetch()} /></>;
  const pending = waiting(q.data.variations, q.data.claims);
  const { contracts, estimates, named } = q.data;
  const settings = <Link to="/e/ConstructionSettings">{t("Construction Settings")}</Link>;
  const setup = t("Progress claims and released retention are invoiced through ERP: set the company, the item they are invoiced as and the retention held in");
  if (contracts.length === 0 && estimates.length === 0)
    return (
      <>
        {head}
        {named ? (
          <StatePanel kind="empty" icon={HardHat} title={t("No contracts yet")} action={<ButtonLink to="/e/Estimate/new" variant="primary" icon="plus">{t("New estimate")}</ButtonLink>}>
            {t("An estimate the customer accepts becomes one.")}
          </StatePanel>
        ) : (
          <StatePanel kind="empty" icon={Settings} title={t("Set up the books first")} action={<ButtonLink to="/e/ConstructionSettings" variant="primary">{t("Construction Settings")}</ButtonLink>}>
            {setup}{isolate(" ")}{settings}.
          </StatePanel>
        )}
      </>
    );
  return (
    <>
      {head}
      <div className="grid gap-4">
      {!named && (
        <Alert tone="warning" action={<ButtonLink to="/e/ConstructionSettings">{t("Construction Settings")}</ButtonLink>}>
          <strong className="block">{t("Set up the books first")}</strong>{setup}{isolate(" ")}{settings}.
        </Alert>
      )}
      {contracts.length > 0 && (
        <TileGrid label={t("Contracts")} className="cn-contracts">
          {contracts.map((c) => (
            <Tile
              key={c.id}
              to={`/e/ConstructionContract/${c.id}`}
              title={isolate(String(c.title))}
              meta={
                <>
                  <span className="text-xs text-muted-foreground">{isolate(String(c.number ?? ""))}</span>
                  <span className="mt-2 grid w-full grid-cols-[1fr_auto] gap-x-4 gap-y-0.5 text-sm tabular-nums">
                    <span className="text-muted-foreground">{t("Worth")}</span>
                    <span className="text-end">{isolate(money(c.contract_sum))}</span>
                    <span className="text-muted-foreground">{t("Certified")}</span>
                    <span className="text-end">{certified(c)}%</span>
                    <span className="text-muted-foreground">{t("Held")}</span>
                    <span className="text-end">{isolate(money(c.retention_held))}</span>
                  </span>
                </>
              }
              status={
                <span className="flex flex-wrap gap-1">
                  <Badge tone={statusTone(String(c.status), CONTRACT_TONES)}>{isolate(String(c.status))}</Badge>
                  {(pending.get(c.id) ?? 0) > 0 && <Badge tone="amber">{t("{pending} waiting", { pending: String(pending.get(c.id)) })}</Badge>}
                </span>
              }
            />
          ))}
        </TileGrid>
      )}
      {estimates.length > 0 && (
        <QueueGrid>
          <Queue title={t("Estimates waiting")} count={estimates.length} more={{ to: "/e/Estimate", count: estimates.length - 5 }}>
            {estimates.slice(0, 5).map((e) => (
              <QueueItem key={e.id} to={`/e/Estimate/${e.id}`} title={isolate(String(e.title))} meta={`${isolate(String(e.number ?? ""))} · ${isolate(money(e.total))}`} />
            ))}
          </Queue>
        </QueueGrid>
      )}
      </div>
    </>
  );
}
