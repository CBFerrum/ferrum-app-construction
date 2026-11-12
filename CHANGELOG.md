# Changelog

## 0.2.2

Rebuilt with Ferrum-Frontend 45dad7f, the design system as tuned after the landed build was reviewed. The left menu's icons are chosen so that none repeats on more than three rows and each fits its row; their drawings load beside the first page instead of inside it, so the page starts lighter; a row that scrolls sideways fades at the edge where there is more, in every browser and with reduced motion; a printed record leaves out empty fields and keeps its totals with the table; menus and popovers sit inside the page's landmarks for assistive technology; a selection bar folds what does not fit into More on a narrow screen. Nothing else changes.

## 0.2.1

The interface follows the upgraded design system, rebuilt with Ferrum-Frontend 0a47157. Text uses the kit's type roles (nothing under 12px) and controls its height tokens; loading shows skeletons shaped like what is coming; empty, no matches, denied, offline and failed are told apart; status colours come from stored state ids. The interface check now runs the UI lint against an empty baseline. The home is rebuilt on the kit's work-home pattern: one page title, the main action as a button, queues with their counts, status at its natural width; its other own pages follow the same anatomy.

## 0.2.0

Subcontracts and their claims, retention capped and released in halves, approvals by someone else, and the margin between them (FEO-049).

- **Subcontracts.** Work let to a supplier under a contract, at its value, holding the settings' retention unless it says otherwise. A subcontract claim says the value of the work done to date. A Contracts Manager other than whoever drew it up certifies it, and what is due after retention and earlier claims is billed through ERP as a draft purchase invoice from the supplier, numbered as the claim. Certified work is not taken back.
- **Independent approval.** A variation is approved by someone other than whoever raised it, and a progress claim is certified by someone other than whoever drew it up.
- **Retention.** A Retention Cap (on a contract, or in the settings) stops what is held growing past that share of the contract sum. Practical Completion releases and invoices half of what is held; completing the contract releases the rest; Retention Released keeps the total.
- **Construction Margin.** Each contract's sum and what is certified of it, beside its subcontracts' worth and what is certified of them. The margin is what was certified to the client less what was certified to subcontractors; the forecast is the contract sum less the subcontracts'. It is for Contracts Managers only.

The package now needs ERP's buying, for suppliers and purchase invoices. Its names are in Arabic and Hindi.

## 0.1.15

Rebuilt with Ferrum-Frontend c59c9a2: the names of its models, fields and manifest in Arabic and Hindi (119 each in ui/src/locales/names, which the platform serves to every page of the workspace, so a record's labels, the launcher and the bar read in the person's language). Translated by the model, to be reviewed by native speakers.

## 0.1.14

The assistant knows what its records mean in its pages (contributes.shell.assistant): their states, what counts and where figures come from, with questions it suggests asking there.

## 0.1.13

Rebuilt with Ferrum-Frontend 5b22e7b: its own words carry built-in Arabic and Hindi (14 words each, ui/src/locales, translated by the model and to be reviewed by native speakers), fetched for the page's language only. The labels it keeps in objects, its dialogs, its page titles and the words it keeps in variables go through t() as well, and the names and sizes it shows as they are keep their order right to left. Its pages are drawn once their words are in, and its entry carries only the parts of the kit it uses.

## 0.1.12

Rebuilt with Ferrum-Frontend 6537d65: an action's link, code or token shown once is kept in a dialog to copy, and on a page that reads right to left, English left untranslated keeps its own order. Its own words go through the kit's t() (13 places), so a workspace's own words for its language reach them, and Admin, Interface words lists them under it (dist/words.json).

## 0.1.11

Rebuilt with Ferrum-Frontend d5d98f2: a workspace's own words for its interface (Admin, Interface words) reach its pages as they load; an attachment field that holds a picture shows it; and a value another record already has is said so, where the form said the record had been changed by someone else.

## 0.1.10

Rebuilt with Ferrum-Frontend 63280d3: right to left, its pages mirror as they read (icons that point along the line, margins and alignment by the line's start and end, arrow keys along it), and the records pages' words are in Arabic and Hindi.

## 0.1.9

Rebuilt with Ferrum-Frontend a64416a: a new record's box whose default is 1 shows ticked, as it is saved.

## 0.1.8

Rebuilt with Ferrum-Frontend 922fcfb: an action's dialog says every refusal, the errors on what the action made (another record's rows) among them, and an action with parameters and a confirmation asks once.

## 0.1.7

What walking a contract through the pages found:
- The contracts page wrote amounts as the database keeps them ("Worth 22180 · held 572"). They are now money, grouped and to the cent.
- An estimate sent and not yet answered was on no page but its list. The contracts page now lists them, waiting for an answer.
- Before the books are named, the page says claims and retention are invoiced from Construction Settings and asks for them first. A page with nothing yet says an accepted estimate becomes a contract.

## 0.1.6

Rebuilt with Ferrum-Frontend a76295e: its pages show times on a person's own clock when they chose one on their Account page, while today and due dates stay on the workspace's day.

## 0.1.5

Rebuilt with Ferrum-Frontend 24c3951: its pages show times on the workspace's clock, naming the zone when it is not the browser's, a draft numbered on submit is headed as a draft, and a record of an app that is not installed says so.

## 0.1.4

Rebuilt with Ferrum-Frontend bb60115: a record page reads an empty number or check as the server does, so an action whose condition needs one shows, and a workspace's settings open as their one record.

## 0.1.3

Rebuilt with Ferrum-Frontend 00dea07: in a workflow state that names what may change, the other fields show read-only, and the fields the state needs show required.

## 0.1.2

Rebuilt with Ferrum-Frontend daeadda: after an action runs, a record page shows what the action said it did rather than only that it is done.

## 0.1.1

Rebuilt with Ferrum-Frontend 80ff179: a field's label names its control for assistive technology, and a record whose title is a link shows that record's title, not its id.

## 0.1.0

Construction and professional services (FERRUM-INDUSTRY-007).

- Estimates priced line by line with a markup, sent, accepted into contracts or declined.
- Contracts holding their schedule of values and retention; lines changed through variations once
  work is certified or varied.
- Variations raised on site and decided by a Contracts Manager.
- Progress claims by line, certified less retention and earlier claims, invoiced through ERP;
  recomputed when certified.
- Completing a contract releases its retention.
- Site diaries, one to a project a day.
- Its own interface at `/construction/`: the contracts with what is certified, held and waiting.
