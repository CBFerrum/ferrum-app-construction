<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/assets/banner-dark.svg">
  <img alt="Ferrum Construction: estimates, contracts, variations and claims in the same books" src="docs/assets/banner-light.svg" width="100%">
</picture>

<br>

[![Version](https://img.shields.io/badge/version-0.2.2-a16207?style=flat-square)](CHANGELOG.md)
[![Licence: AGPL-3.0](https://img.shields.io/badge/licence-AGPL--3.0-blue?style=flat-square)](LICENSE)
[![Ferrum platform](https://img.shields.io/badge/runs%20on-Ferrum-2f5bea?style=flat-square)](https://github.com/CBFerrum/Ferrum)
![Rust](https://img.shields.io/badge/Rust-server-b7410e?style=flat-square&logo=rust)
![PostgreSQL](https://img.shields.io/badge/PostgreSQL-data-336791?style=flat-square&logo=postgresql&logoColor=white)
![React](https://img.shields.io/badge/React%20%2B%20TypeScript-interface-087ea4?style=flat-square&logo=react&logoColor=white)
![Tests](https://img.shields.io/badge/tests-3%20Rust%20%2B%20UI%20suite-16a34a?style=flat-square)

### From the first estimate to the last retention release, in the books you already keep.

Ferrum Construction is for contractors and professional-services firms that price work, sign contracts, change them
on site and bill by progress. It replaces the spreadsheet that tracks a schedule of values, the folder of variation
emails and the hand-made progress-claim invoice. Every invoice it raises is an invoice in Ferrum ERP.

[💡 Why](#-why-construction) · [📸 Screenshots](#-screenshots) · [✨ Features](#-features) · [📊 Compare](#-how-it-compares) · [📍 Roadmap](#-roadmap) · [🚀 Install](#-install) · [🔧 Develop](#-develop)

</div>

## 💡 Why Construction

- 🧾 **One set of books.** Progress claims and released retention become draft Sales Invoices in ERP. Subcontract
  claims become draft Purchase Invoices. There is no export, no sync and no second ledger to reconcile.
- 🧮 **The arithmetic is done for you.** A markup is spread across estimate lines to the cent. A claim is worth
  its lines' percentages, less retention, less what earlier claims already certified. A draft drawn up before
  another claim was certified is recomputed, so nothing is paid twice.
- 🔒 **Certified work stays certified.** A line is never claimed at less than it was. A contract cannot be varied to
  be worth less than what is already certified. Claims follow one another in time.
- 👥 **Two people sign.** A variation is approved, and a claim or subcontract claim is certified, by someone other
  than whoever raised it. Only a Contracts Manager decides.
- 🔓 **Open source and self-hosted.** AGPL-3.0, on your own PostgreSQL. No licence per user and no licence per
  volume of work.
- 🌍 **Reads in three languages.** The interface and the names of every record are available in English, Arabic
  and Hindi, and pages mirror for right-to-left reading.
- 🧩 **Extendable in Studio.** Add fields, forms and rules to a record on the Ferrum builder without forking the app.

## 🎯 At a glance

| | |
|---|---|
| **Version** | 0.2.2 |
| **Kind** | Its own interface at `/construction/`: a contracts home, with every record in Ferrum's lists and forms |
| **What it adds** | 11 record types (7 documents, 3 line tables, 1 settings record), 2 roles, 1 report, 1 menu |
| **Depends on** | Ferrum platform `>=0.3.2, <0.4`; ERP `>=0.1` (installing Construction installs ERP) |
| **Works with** | ERP (customers, suppliers, items, companies, sales and purchase invoices), project records |
| **Languages** | English, Arabic, Hindi (words and record names); right-to-left layout |
| **Tests** | 3 Rust tests (one walks a whole building job on PostgreSQL) and a UI suite |
| **Licence** | AGPL-3.0-only |

## 📸 Screenshots

<div align="center">
<picture><source media="(prefers-color-scheme: dark)" srcset="docs/assets/screens/construction-home-dark.webp"><img alt="Contracts at a glance" src="docs/assets/screens/construction-home-light.webp" width="100%"></picture>
<p><b>🏗️ Contracts at a glance</b><br><sub>Each contract with what it is worth, how much is certified, the retention held, and what waits on someone.</sub></p>
</div>

<table>
<tr>
<td width="50%" valign="top">
<picture><source media="(prefers-color-scheme: dark)" srcset="docs/assets/screens/construction-construction-contract-list-dark.webp"><img alt="Contracts" src="docs/assets/screens/construction-construction-contract-list-light.webp"></picture>
<p><b>📑 Contracts</b><br><sub>Browse, search and filter contracts by customer, status and sum.</sub></p>
</td>
<td width="50%" valign="top">
<picture><source media="(prefers-color-scheme: dark)" srcset="docs/assets/screens/construction-estimate-list-dark.webp"><img alt="Estimates" src="docs/assets/screens/construction-estimate-list-light.webp"></picture>
<p><b>🧮 Estimates</b><br><sub>Price work for a customer and see which estimates are sent, accepted or declined.</sub></p>
</td>
</tr>
<tr>
<td width="50%" valign="top">
<picture><source media="(prefers-color-scheme: dark)" srcset="docs/assets/screens/construction-subcontract-list-dark.webp"><img alt="Subcontracts" src="docs/assets/screens/construction-subcontract-list-light.webp"></picture>
<p><b>🤝 Subcontracts</b><br><sub>Work let to suppliers under a contract, with what is certified and the retention held.</sub></p>
</td>
<td width="50%" valign="top">
<picture><source media="(prefers-color-scheme: dark)" srcset="docs/assets/screens/construction-construction-contract-detail-dark.webp"><img alt="One contract" src="docs/assets/screens/construction-construction-contract-detail-light.webp"></picture>
<p><b>📐 One contract</b><br><sub>Read one contract: its schedule of values, sums and retention.</sub></p>
</td>
</tr>
<tr>
<td width="50%" valign="top">
<picture><source media="(prefers-color-scheme: dark)" srcset="docs/assets/screens/construction-progress-claims-dark.webp"><img alt="Progress claims" src="docs/assets/screens/construction-progress-claims-light.webp"></picture>
<p><b>📈 Progress claims</b><br><sub>What has been claimed, certified and invoiced on each contract.</sub></p>
</td>
<td width="50%" valign="top">
<picture><source media="(prefers-color-scheme: dark)" srcset="docs/assets/screens/construction-variation-list-dark.webp"><img alt="Variations" src="docs/assets/screens/construction-variation-list-light.webp"></picture>
<p><b>🔀 Variations</b><br><sub>Changes to the agreed scope, their value and who decided them.</sub></p>
</td>
</tr>
<tr>
<td width="50%" valign="top">
<picture><source media="(prefers-color-scheme: dark)" srcset="docs/assets/screens/construction-estimate-detail-dark.webp"><img alt="One estimate" src="docs/assets/screens/construction-estimate-detail-light.webp"></picture>
<p><b>📝 One estimate</b><br><sub>Read an estimate&#x27;s priced lines, then send it or accept it.</sub></p>
</td>
<td width="50%"></td>
</tr>
</table>

### 📱 On a phone

<table>
<tr>
<td align="center" valign="top">
<picture><source media="(prefers-color-scheme: dark)" srcset="docs/assets/screens/construction-home-phone-dark.webp"><img alt="Contracts on a phone" src="docs/assets/screens/construction-home-phone-light.webp" width="260"></picture>
<p><b>🏗️ Contracts on a phone</b><br><sub>The contracts page on a narrow screen.</sub></p>
</td>
<td align="center" valign="top">
<picture><source media="(prefers-color-scheme: dark)" srcset="docs/assets/screens/construction-construction-contract-list-phone-dark.webp"><img alt="Contract list on a phone" src="docs/assets/screens/construction-construction-contract-list-phone-light.webp" width="260"></picture>
<p><b>📑 Contract list on a phone</b><br><sub>Find a contract from the site.</sub></p>
</td>
</tr>
</table>


## ✨ Features

### 🧮 Estimates

- **Priced line by line.** Each estimate line has a description, a quantity, a unit (m², m³, hours, lump sum), a rate
  and an amount worked out for you. The subtotal is the sum of the lines.
- **A markup on top.** One percentage for overheads and profit gives the estimate's total.
- **Checks on the lines.** A quantity must be above zero and a rate zero or more. The markup cannot be negative.
- **A life of four states.** An estimate is *Draft*, then *Sent*, then *Accepted* or *Declined*. Mark It Sent, Accept
  It and Decline It are buttons that show only in the right state.
- **Prices are kept.** Once sent, an estimate's prices do not change. To price again, copy it.
- **Numbered and dated.** `EST-2026-00001`, with a "valid until" date, a customer, an optional project and notes.
- **Accepted, it becomes a contract.** The contract's schedule of values is the estimate's lines, numbered 1, 2, 3.
  The markup is spread across the lines to the cent, so the contract is worth exactly the estimate's total. The
  estimate links to its contract, and the contract to its estimate.
- **Waiting estimates are on the home page.** Estimates sent and not yet answered are listed there, newest first.

### 📑 Contracts and the schedule of values

- **A contract is its lines.** Each line has a code, a description and a value; the contract sum is the sum of the
  lines. Codes are unique on a contract.
- **Original, varied and current sums.** The contract keeps the original sum, the sum of approved variations, and
  the contract sum today.
- **What is certified, what is held.** *Certified to date* is the value of certified work before retention.
  *Retention held* is what is kept back so far.
- **Retention as agreed.** A contract holds its own retention percentage and its own cap. When it names none, the
  company's Construction Settings supply them.
- **Locks once work starts.** When work is certified or varied, the lines, the customer and the retention percentage
  change only through variations. A complete contract is not changed at all.
- **Practical Completion.** A Contracts Manager releases half of the retention held, after a confirmation. Ferrum
  invoices that half through ERP, records the date and keeps the first half's invoice on the contract.
- **Complete It.** A contract can be completed only when it is certified in full. Completing it invoices the rest of
  the retention, sets the completion date and clears the retention held. *Retention released* keeps the total.
- **Numbered** `CON-2026-00001`, with a customer, an optional project and the estimate it came from.

### 🔀 Variations

- **Raised on site.** A variation names a contract, a description, a reason (*Client Request*, *Design Change*,
  *Site Condition*, *Omission*), an amount and details. The amount may be below zero for work taken out.
- **Four states.** *Draft*, *Submitted*, *Approved* or *Rejected*. Submit It, Approve It and Reject It are buttons
  that show only when they apply.
- **Decided by a Contracts Manager.** The date and the person who decided are recorded on the variation.
- **Not by the person who raised it.** Someone other than whoever raised the variation approves it.
- **Approved, it joins the contract.** An approved variation becomes a line of its own on the contract, and the
  variations sum and contract sum move with it.
- **Guarded.** A variation must add or take out some value. It cannot be raised on a complete contract. Approving one
  that would make the contract worth less than what is certified is refused. Once submitted or decided, a
  variation keeps its terms.
- **Numbered** `VAR-2026-00001`.

### 📈 Progress claims

- **Measured by line.** A claim has one row per contract line, found by its code, with a percentage done to date. The
  row's value to date is the line's value times that percentage.
- **Filled from the contract.** The claim's rows come from the contract's lines as they stand. A line the claim does
  not mention stays where the last certified claim had it.
- **The money.** *Value to date*, less *retention* held to date, less what earlier claims certified, is *amount due*.
  The retention is the contract's percentage, held up to the contract's cap.
- **Checks.** Each line is between 0% and 100%. A line is never claimed at less than the last certified claim had it
  ("work certified is not taken back"). A claim's period end must be after the last certified claim's.
- **Certify It.** A Contracts Manager certifies; a Site Manager may measure. It cannot be the person who drew the
  claim up. Certifying recomputes the claim against the contract and the claims certified at that moment, with the
  contract locked while it runs, so two claims are never certified against the same past.
- **Invoiced through ERP.** The amount due becomes a draft Sales Invoice to the contract's customer, one line,
  invoiced as the item named in the settings, with the claim's number and period in its remarks.
- **Nothing due, nothing certified.** A claim that adds no work is refused. A certified claim is not changed: the next
  claim carries on from it. A complete contract takes no more claims.
- **Numbered** `PC-2026-00001`.

### 🤝 Subcontracts and their claims

- **Work let to a supplier.** A subcontract names its contract, a supplier, a value and a retention percentage. The
  settings' percentage applies when it names none. It keeps what is certified to date and what is held.
- **Claims by value to date.** A subcontract claim says the value of the work done to date, all of it, from 0 to the
  subcontract's value. Work certified is not taken back and periods move forward.
- **Certified by someone else.** Only a Contracts Manager certifies a subcontract claim, and not the person who
  drew it up.
- **Billed through ERP.** What is due after retention and earlier claims becomes a draft Purchase Invoice from the
  supplier, numbered as the claim.
- **A Site Manager** may see subcontracts and draw up their claims.
- **Numbered** `SUB-2026-00001` and `SCC-2026-00001`. A subcontract cannot be worth less than what is certified.

### 📓 The site diary

- **One a day for each project.** Date, weather (*Clear*, *Rain*, *Wind*, *Heat*, *Snow*), number of workers, work
  done, delays, visitors and the number of safety incidents.
- **Signed by whoever kept it.** The writer's email is filled in for you.
- **No second diary for a day.** The project's diary for a day that has one is added to, not repeated.
- **Counted from zero.** People and incidents cannot be negative.

### 📊 Construction Margin and the home page

- **Construction Margin report.** Each contract's sum and what is certified of it, beside its subcontracts' value and
  what is certified of them. The margin is what was certified to the client less what was certified to
  subcontractors; the forecast margin is the contract sum less the subcontracts' value. Contracts Managers only,
  because it shows what subcontracts cost.
- **The contracts page.** One tile per contract: its number, what it is worth, the percentage certified, the retention
  held, its status, and a badge counting variations submitted and claims in draft that wait on someone. Each tile
  opens its record.
- **Estimates waiting.** A queue of sent estimates, with a link to all of them.
- **First-run help.** Before the books are named, the page says claims and retention are invoiced from Construction
  Settings and links there. With nothing yet, it says an accepted estimate becomes a contract.
- **Told-apart states.** Loading, empty, no matches, denied, offline and failed look different.

### 🔐 Roles and permissions

| Who | Can |
|---|---|
| 🏗️ Contracts Manager | Everything in Construction: settings, estimates, contracts, variations, claims, subcontracts, diaries, the margin report. Reads projects, items and companies; creates customers and invoices through ERP |
| 🧑‍🔧 Site Manager | Sees contracts and subcontracts; raises variations and draws up claims; draws up subcontract claims; keeps diaries |

### 🌐 Interface

- **Its own pages.** The home is rebuilt on Ferrum's work-home pattern; every record uses the shared list, form and
  report pages, with search, filters, export and history.
- **Languages.** English, Arabic and Hindi words for the interface, and Arabic and Hindi names for every record and
  field. A workspace can add its own words in Admin.
- **Right to left.** Pages mirror as they read.
- **Assistant.** Ferrum's assistant knows the states and where the figures come from, and suggests questions such
  as "Which progress claims wait to be certified?" and "How much retention is held on our contracts?".
- **Phone friendly.** The pages fit a narrow screen, as the screenshots show.

## 🧭 How it works

```mermaid
flowchart LR
  E["Estimate (Draft)"] --> S["Sent"]
  S -->|accepted| C["Contract with schedule of values"]
  S -->|declined| D["Declined"]
  C --> V["Variation, approved"]
  V --> C
  C --> P["Progress claim, certified"]
  P --> I["Sales invoice in ERP"]
  C --> PC["Practical completion: half of retention invoiced"]
  C --> X["Complete: rest of retention invoiced"]
```

An accepted estimate becomes a contract. Approved variations add lines to it. Each certified claim becomes an
invoice, and the retention held is invoiced in two halves.

```mermaid
flowchart TD
  C["Contract"] --> SC["Subcontract (supplier)"]
  SC --> SCC["Subcontract claim, certified"]
  SCC --> PI["Purchase invoice in ERP"]
  C --> R["Construction Margin report"]
  SC --> R
```

Subcontracts hang off a contract. Their certified claims become purchase invoices, and the margin report sets
what was certified to the client against what was certified to subcontractors.

## 🧱 Data model

| Entity | What it is | Key fields or states |
|---|---|---|
| Estimate | Priced work for a customer | Customer, project, valid until, lines, markup, subtotal, total; *Draft, Sent, Accepted, Declined* |
| Estimate Line | A priced row of an estimate | Description, quantity, unit, rate, amount |
| Contract (`ConstructionContract`) | The work agreed, line by line | Customer, project, estimate, retention and cap, original, variations and contract sum, certified to date, retention held; *Active, Complete* |
| Contract Line | A row of the schedule of values | Code, description, value |
| Variation | A priced change to a contract | Contract, reason, amount, decided on and by; *Draft, Submitted, Approved, Rejected* |
| Progress Claim | How far each contract line has got | Contract, period to, lines, value to date, retention, previous certified, amount due, sales invoice; *Draft, Certified* |
| Claim Line | A row of a claim | Code, description, value, percent complete, value to date |
| Subcontract | Work let to a supplier | Contract, supplier, value, retention percentage, certified to date, retention held |
| Subcontract Claim | A subcontract's work done to date | Subcontract, period to, value to date, retention, amount due, certified by, purchase invoice; *Draft, Certified* |
| Site Diary | A day on site | Project, date, weather, workers, work done, delays, visitors, incidents, written by |
| Construction Settings | Where claims are invoiced from | Company, claim item, retention percentage, retention cap |

**Roles:** Contracts Manager and Site Manager, as above.

**Reports:** Construction Margin.

**Print formats and scheduled jobs:** none. Nothing runs by itself.

## 🔌 Integrations

| With | How | Needs |
|---|---|---|
| **Ferrum ERP** | Customers, suppliers, items and companies are ERP's. Certified claims, practical completion and completion make draft **Sales Invoices**; certified subcontract claims make draft **Purchase Invoices** | ERP installed (it is installed with the app); a company and a claim item in Construction Settings |
| **Project records** | A contract, estimate and diary can name a project | Project records package of the platform |
| **Ferrum assistant** | Knows the records' states and suggests questions | Nothing |
| **Ferrum REST API and Studio** | Every record is reachable through the platform's API and can be extended in Studio | Platform permissions |

No outside standards or services are built in. Drawings, schedules, e-signature and payment gateways are not part of
this app yet (see the roadmap).

## 📊 How it compares

Construction is measured here against **Procore**, **Autodesk Construction Cloud**, **Buildertrend** and **Sage
Intacct Construction**. It covers the commercial spine of a job (estimate, contract, change, claim, retention,
subcontract) inside an ERP, and leaves the field and document side to others for now.

| Capability | Ferrum Construction | Procore | Autodesk Construction Cloud | Buildertrend | Sage Intacct Construction |
|---|:-:|:-:|:-:|:-:|:-:|
| Estimates priced by line with a markup, accepted into a contract | ✅ | ✅ | 🟡 | ✅ | 🟡 |
| Contract with a schedule of values | ✅ | ✅ | ✅ | 🟡 | ✅ |
| Variations or change orders that change the contract sum | ✅ | ✅ | ✅ | ✅ | ✅ |
| Progress claims by percentage per line | ✅ | ✅ | ✅ | 🟡 | ✅ |
| Retention held, capped and released in stages | ✅ | ✅ | ✅ | 🟡 | ✅ |
| Approval by someone other than the person who raised it | ✅ | ✅ | ✅ | 🟡 | ✅ |
| Subcontracts with their own claims and retention | ✅ | ✅ | ✅ | 🟡 | ✅ |
| Invoices land in the same ledger, payables and receivables | ✅ | 🟡 | 🟡 | 🟡 | ✅ |
| Daily site diary with weather, workers and incidents | 🟡 | ✅ | ✅ | ✅ | 🟡 |
| Contract margin from certified work | 🟡 | ✅ | ✅ | ✅ | ✅ |
| Job costing by cost code: budget, commitments, forecast to complete | ❌ | ✅ | ✅ | ✅ | ✅ |
| Change chain: change events, owner sign-off, subcontract changes | 🟡 | ✅ | ✅ | ✅ | 🟡 |
| Claims by quantity and unit price, stored materials, AIA-style forms | ❌ | ✅ | ✅ | 🟡 | ✅ |
| RFIs with due dates and ball in court | ❌ | ✅ | ✅ | 🟡 | ❌ |
| Submittals register and review cycles | ❌ | ✅ | ✅ | 🟡 | ❌ |
| Punch lists and quality or safety inspections | ❌ | ✅ | ✅ | ✅ | ❌ |
| Drawings and documents with revisions and markup | ❌ | ✅ | ✅ | 🟡 | 🟡 |
| Schedule with tasks and Gantt | ❌ | ✅ | 🟡 | ✅ | ❌ |
| Takeoff from plans | ❌ | ✅ | ✅ | ✅ | 🟡 |
| Bid management and award to a subcontract | ❌ | ✅ | ✅ | ✅ | ❌ |
| Time and equipment hours charged to jobs | ❌ | ✅ | 🟡 | ✅ | ✅ |
| Subcontractor portal where subs submit their own claims | ❌ | ✅ | ✅ | ✅ | 🟡 |
| Insurance, licence and lien-waiver compliance | ❌ | ✅ | 🟡 | 🟡 | 🟡 |
| Client portal with progress, photos and approvals | ❌ | 🟡 | 🟡 | ✅ | ❌ |
| Meetings, minutes and action items | ❌ | ✅ | ✅ | 🟡 | ❌ |
| Native mobile app that works offline on site | ❌ | ✅ | ✅ | ✅ | 🟡 |
| Open API for the records | ✅ | ✅ | ✅ | 🟡 | ✅ |
| Words in Arabic and Hindi, right-to-left layout | ✅ | 🟡 | 🟡 | ❌ | 🟡 |
| Self-hosted on your own servers | ✅ | ❌ | ❌ | ❌ | ❌ |
| Open source | ✅ | ❌ | ❌ | ❌ | ❌ |
| No licence fee per user or per volume of work | ✅ | ❌ | ❌ | ❌ | ❌ |

Notes on the 🟡 cells. **Ferrum:** the diary has one record a day with counts and free text, no lines by trade, no
photos; the margin is one report, with no cost codes or budget; variations are decided by internal roles only.
**Leaders:** 🟡 means a lighter feature, a separate product, or a feature the leader reaches through an integration
(for example accounting for Procore and Autodesk, or drawings for Buildertrend and Sage). Oracle Aconex is the usual
choice for formal document transmittals between firms; Construction does not do that either.

### 🏆 Where Construction stands out

- 🧾 **One ledger.** A claim becomes an invoice in the same books as ERP's selling, buying and accounting. The
  leaders sync to an accounting package or are one.
- 🔒 **Hard rules on money.** Certified work cannot be taken back, a contract cannot be varied below what is
  certified, and stale drafts are recomputed before they are invoiced.
- 👥 **Separation of duties by default.** Raising and approving are different people.
- 🔓 **Open and owned.** Self-hosted, AGPL, no per-seat or per-volume pricing, and a data set you can query.
- 🧩 **Extendable on the Ferrum builder.** Add fields, forms and rules in Studio.

### 🚧 Where the leaders are still ahead

- 🏗️ **The field side.** RFIs, submittals, punch lists, inspections, drawings with markup, a schedule, daily logs by
  trade with photos and meeting minutes are all missing here.
- 💰 **Job costing.** Budgets by cost code, commitments, forecast to complete and WIP reporting.
- 📱 **A native mobile app,** with offline use on site.
- 🌐 **Portals.** Client and subcontractor portals, bid management and compliance tracking.
- 📐 **Billing formats.** Claims by quantity, stored materials, and AIA-style forms.

> [!NOTE]
> Leader columns are from public product information as of November 2026 and may change. Product names belong to
> their owners.

## 📍 Roadmap

### ✅ Recently shipped

- [x] **0.2.2** Tuned design system: better menu icons, a lighter first page, a visible scroll cue, cleaner print.
- [x] **0.2.1** The redesigned interface: shaped loading states, told-apart empty, denied and failed states.
- [x] **0.2.0** Subcontracts and their claims; retention caps and release in halves; approvals by someone else;
  the Construction Margin report.
- [x] **0.1.15** Names of models and fields in Arabic and Hindi.
- [x] **0.1.14** The assistant knows the records' states and suggests questions.
- [x] **0.1.7** Amounts written as money; sent estimates listed on the home page; first-run help for the books.

### 🔜 Next up

**Money**
- [ ] Job costing: a budget by cost code, commitments and actuals, with forecast to complete. This is what most
  contractors check first.
- [ ] Claims by measured quantity and unit price, not only by percentage.
- [ ] Claims for materials stored on site.
- [ ] Change chain: change events that become variations, and a client's approval of a variation.

**On site**
- [ ] A richer daily log: manpower by trade, equipment, deliveries and photos.
- [ ] RFIs with a due date and who must answer.
- [ ] Submittals with review cycles.
- [ ] Punch lists and closeout.

**People outside the firm**
- [ ] A client portal to approve variations and claims.
- [ ] A subcontractor portal where subcontractors submit their own claims.

### 🔭 Later

**Documents and planning**
- [ ] Drawings and documents with revisions and markup.
- [ ] A schedule with tasks, a Gantt view and look-ahead.
- [ ] Meetings, minutes and action items.

**Estimating and bidding**
- [ ] A cost catalogue, assemblies and a markup on each line.
- [ ] Bid packages, invitations and levelling, awarded into a subcontract.
- [ ] Takeoff from plans.

**Compliance and cost**
- [ ] Subcontractor insurance and licence expiry, prequalification and lien waivers.
- [ ] Timesheets and equipment hours charged to jobs.
- [ ] Quality and safety inspections and observations.
- [ ] Print formats for estimates, claims and certificates.

## 🚀 Install

Construction is a native Ferrum app: the Ferrum server includes it (`runtime: native`) and a workspace turns it
on from **App Manager**. It needs platform `>=0.3.2, <0.4`. Installing it installs **ERP**, because claims are
invoiced through ERP's selling and buying, and it uses project records.

After installing:

1. Give people the roles **Contracts Manager** and **Site Manager**.
2. Open **Construction Settings** and set the **company**, the **claim item** that claims and released retention are
   invoiced as, the default **retention percentage**, and an optional **retention cap**.
3. Create an **Estimate**, send it, and accept it. The contract appears with its schedule of values.
4. Raise a variation or draw up a progress claim from the contract.

## 🔧 Develop

```bash
cargo fmt -- --check
cargo clippy --all-targets -- -D warnings
FERRUM_CONSTRUCTION_TEST_DATABASE_URL=postgres://ferrum:ferrum@localhost:5432/ferrum_construction_test cargo test
```

The database must exist, be owned by a role that is not a superuser (superusers bypass row-level security) and be
reset before the run. `tests/construction_it.rs` walks a building job: an estimate sent and accepted, variations
raised and decided, three claims (one of them stale), completing and releasing retention, the diary, and who may do
what. Rust dependencies come from the Ferrum platform and Ferrum ERP repositories on `main`.

The interface lives in `ui/` and is built with Ferrum-Frontend. Check the Frontend out beside the repository as
`../ferrum/web` and install its packages, then:

```bash
cd ui
npm run build       # builds into ui/dist, which the server embeds
npm run check       # interface lint
npm run typecheck
npm test
```

CI runs both jobs (`.github/workflows/ci.yml`): Rust on PostgreSQL 17, and the interface checks.

## 📂 Project layout

```text
app.yaml                  manifest: id, roles, shell entries, assistant notes
packages/construction/    the package: entities, roles and the menu as YAML
src/estimates.rs          pricing, markup spreading, accepting into a contract
src/contracts.rs          schedule of values, variations, retention, completion
src/claims.rs             progress claims and certifying
src/subcontracts.rs       subcontracts and their claims
src/diary.rs              the site diary
src/reports.rs            Construction Margin
src/lib.rs                registration, settings, invoicing helpers
ui/                       the interface (home page, words in Arabic and Hindi)
tests/construction_it.rs  a building job on PostgreSQL
docs/PROVENANCE.md        where the design comes from
```

## 📜 Changelog

Every release is described in [CHANGELOG.md](CHANGELOG.md).

## 🤝 Contributing

Issues and pull requests are welcome at [github.com/CBFerrum/ferrum-app-construction](https://github.com/CBFerrum/ferrum-app-construction/issues).
Keep the checks above green. The design and code of this app are its own; see [docs/PROVENANCE.md](docs/PROVENANCE.md).

## 📄 License

AGPL-3.0-only. See [LICENSE](LICENSE).

<div align="center">

<sub>Part of <a href="https://github.com/CBFerrum/Ferrum">Ferrum</a>, the open business platform · Made with 🦀 Rust and 🐘 PostgreSQL</sub>

</div>
