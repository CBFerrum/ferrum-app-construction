# Changelog

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
