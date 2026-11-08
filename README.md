# Construction

Construction and professional services for [Ferrum](https://github.com/CBFerrum/Ferrum):
- estimates priced line by line with a markup;
- contracts with their schedule of values, changed by approved variations;
- progress claims certified less retention and invoiced through ERP;
- site diaries.

| | |
|---|---|
| 🏗️ App | `construction`, native, in the server |
| 🧩 Package | `construction` (needs ERP's selling and accounting, and project records) |
| 🖥️ Interface | its own, at `/construction/`: each contract with what is certified, held and waiting, and the records |
| 🔐 Roles | Contracts Manager, Site Manager |
| 📜 Licence | AGPL-3.0-only |

## Highlights

- **Estimates.** Each line is its quantity at its rate, and the total adds the markup. Once sent, an
  estimate keeps its prices. Accepted, it becomes a contract whose lines are the estimate's, the
  markup spread across them to the cent so the contract is worth the estimate's total.
- **Contracts and variations.** A contract is worth the sum of its lines, and holds the retention
  agreed for it (the settings' when it names none). Once work is certified or varied, its lines
  change only through variations. A Site Manager raises and submits a variation, and a Contracts
  Manager approves or rejects it. An approved one joins the contract as a line of its own. One that
  would make the contract worth less than what is certified is refused.
- **Progress claims.** A claim measures each line by its code, as a percentage done to date. A line
  it does not mention stays where the last certified claim had it. Its value to date, less the
  retention held, less what earlier claims certified, is what is due. Certified work is not taken
  back, and claims follow one another in time.
- **Certifying.** A Site Manager may measure a claim; a Contracts Manager certifies it. It is recomputed against the contract and
  the claims certified as they stand, so a draft drawn up before another claim was certified is not
  paid twice. Then what is due is invoiced through ERP to the customer.
- **Completing.** A contract certified in full is completed, and the retention held is invoiced.
- **The site diary.** One to a project a day: weather, workers, the work done, delays, visitors and
  incidents, signed by whoever kept it.

## Develop

```bash
cargo test                       # needs FERRUM_CONSTRUCTION_TEST_DATABASE_URL's database
cd ui && npm run build && npm run check
```

`tests/construction_it.rs` walks a building job:
- an estimate, sent and accepted;
- variations raised on site and decided;
- three claims, one of them drawn up stale;
- completing, and the retention released;
- the diary;
- who may do what.

## Not yet

- Retention released in halves (at practical completion and after defects), or capped.
- Claims for materials on site, and subcontractors' claims and payments.
- Measured quantities (claims by quantity rather than percentage), and cost against budget.
- A client's portal for approving variations and claims.

## Changelog

| Version | Highlights |
|---|---|
| **0.1.15** | Its names in Arabic and Hindi. |
| **0.1.14** | The assistant knows its records. |
| **0.1.13** | Rebuilt with Ferrum-Frontend 5b22e7b: its own words in Arabic and Hindi. |
| **0.1.12** | Rebuilt with Ferrum-Frontend 6537d65: once-shown answers kept; English isolated right to left; its own words translatable by a workspace. |
| **0.1.11** | Rebuilt with Ferrum-Frontend d5d98f2: a workspace's own interface words; pictures in attachment fields; a duplicate value said so. |
| **0.1.10** | Rebuilt with Ferrum-Frontend 63280d3: right to left, pages mirror; the records pages' words in Arabic and Hindi. |
| **0.1.9** | Rebuilt with Ferrum-Frontend a64416a: a box whose default is 1 shows ticked. |
| **0.1.8** | Rebuilt with Ferrum-Frontend 922fcfb: action dialogs say every refusal and ask once. |
| **0.1.7** | What walking a contract found: amounts written as money; estimates waiting for an answer; the books named before the first claim. |
| **0.1.6** | Rebuilt with Ferrum-Frontend a76295e: times on a person's own clock when they chose one. |
| **0.1.5** | Rebuilt with Ferrum-Frontend 24c3951: times on the workspace's clock; drafts headed as drafts. |
| **0.1.4** | Rebuilt with Ferrum-Frontend bb60115: empty numbers read as the server reads them; settings open as their record. |
| **0.1.3** | Rebuilt with Ferrum-Frontend 00dea07: a workflow state's kept fields are read-only, its needed ones required. |
| **0.1.2** | Rebuilt with Ferrum-Frontend daeadda: actions say what they did. |
| **0.1.1** | Rebuilt with Ferrum-Frontend 80ff179: labels name their fields, and linked titles show their titles. |
| **0.1.0** | Estimates, contracts with variations, progress claims with retention invoiced through ERP, and site diaries (FERRUM-INDUSTRY-007). |

## License

AGPL-3.0-only. See [LICENSE](LICENSE).
