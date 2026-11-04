import { describe, expect, it } from "vitest";
import { certified, waiting } from "./Home";

describe("the contracts page", () => {
  it("shows how far a contract is certified", () => {
    expect(certified({ id: "c", contract_sum: "200000", certified_to_date: "50000" })).toBe(25);
    expect(certified({ id: "c", contract_sum: "0", certified_to_date: "0" })).toBe(0);
  });

  it("counts what waits on each contract", () => {
    const m = waiting([{ id: "v1", contract: "c1" }], [
      { id: "p1", contract: "c1" },
      { id: "p2", contract: "c2" },
    ]);
    expect(m.get("c1")).toBe(2);
    expect(m.get("c2")).toBe(1);
  });
});
