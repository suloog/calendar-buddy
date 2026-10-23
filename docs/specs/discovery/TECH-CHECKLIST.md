
# Technical checklist

> The company's standing list of technical questions every project answers. The canon (what exactly
> to establish, who answers, where the answer lands) lives in the plugin:
> `skills/discovery/references/tech-checklist.md`. This file holds **this project's answers**.
>
> `/og:discovery close` will not run while any item has no status, or while its reference points at
> an identifier that does not exist. There is no flag that bypasses it.
>
> **Statuses** (always the English values): `answered` — settled, points at a `DSC-NN-nn` ·
> `assumed` — we took a default, points at a `Qnn [A]` · `open` — only the client can say, points
> at a `Qnn [W]` · `decided` — settled by a ruling, points at a `DEC-nn` · `n/a` — deliberately out
> of scope, points at a `SRC-NN-nn` or `DEC-nn` and **requires a note giving the reason**.

## A. Security

| TQ | Question | Status | Reference | Note |
|------|----------|--------|-----------|------|
| TQ01 | Encryption at rest | decided | `DEC-15` | Tokens in the OS keyring; cache not encrypted by the app (disk encryption covers it). |
| TQ02 | Encryption in transit | answered | `DSC-02-13` | Only HTTPS to Google APIs. |
| TQ03 | Multi-factor authentication | n/a | `DEC-18` | Sign-in is Google's; MFA follows the Google account / Workspace policy. |
| TQ04 | Security audits | n/a | `DEC-18` | Internal tool; no audit planned. |

## B. Performance

| TQ | Question | Status | Reference | Note |
|------|----------|--------|-----------|------|
| TQ05 | Response time | answered | `DSC-02-12` | Three flows measured; thresholds set after measurements (DSC-02-03). |
| TQ06 | Throughput | n/a | `DEC-18` | Single user, no server; scale is data volume (DSC-02-12). |
| TQ07 | Vertical and horizontal scalability | n/a | `DEC-18` | No server to scale. |

## C. Availability

| TQ | Question | Status | Reference | Note |
|------|----------|--------|-----------|------|
| TQ08 | SLA | n/a | `DEC-18` | No service of ours; Google availability is outside our control. |
| TQ09 | Redundancy | n/a | `DEC-18` | No server; nothing to make redundant. |
| TQ10 | Disaster recovery (RTO/RPO, backups) | decided | `DEC-15` | No backups; Google is the system of record, cache rebuilt by resync. |

## D. Compliance

| TQ | Question | Status | Reference | Note |
|------|----------|--------|-----------|------|
| TQ11 | Applicable regulations (GDPR, PCI DSS, HIPAA, local) | decided | `DEC-16` | No backend, no processing by us; secure tokens and clean logs only. |
| TQ12 | Personal data — roles, processing agreement, region, retention | decided | `DEC-16` | Data stays on the device and in Google; cache retention in DSC-04 (Q22). |

## E. Portability

| TQ | Question | Status | Reference | Note |
|------|----------|--------|-----------|------|
| TQ13 | Delivery form and offline operation | answered | `DSC-01-10` | Native Linux desktop app installed locally; offline browsing required. |

## F. Maintainability

| TQ | Question | Status | Reference | Note |
|------|----------|--------|-----------|------|
| TQ14 | Updates and maintenance | decided | `DEC-07` | No maintenance contract; updates are a new .deb installed by hand (auto-update: Q27). |
| TQ15 | Monitoring and diagnostics | decided | `DEC-13` | No monitoring; local logs only. |

## G. Testability

| TQ | Question | Status | Reference | Note |
|------|----------|--------|-----------|------|
| TQ16 | Required test levels | decided | `DEC-17` | Unit + integration for sync, recurrence, editing, RSVP; manual acceptance; no load tests. Lands in the constitution. |

## H. Deployment environment

| TQ | Question | Status | Reference | Note |
|------|----------|--------|-----------|------|
| TQ17 | Infrastructure ownership | answered | `DSC-01-16` | No servers; the only environment is the GCP project in the organization's Workspace, owned and paid for by the organization. |
| TQ18 | Owner of permissions and access (named) | decided | `DEC-09` | Maciej on both sides (GCP project owner and Workspace admin). |

## I. Test targets

| TQ | Question | Status | Reference | Note |
|------|----------|--------|-----------|------|
| TQ19 | Web — browsers and minimum versions | n/a | `DEC-18` | Not a web application. |
| TQ20 | Web — resolutions and breakpoints | n/a | `DEC-18` | Not a web application; window sizes and DPI are covered by SRC-01-48 / DSC-02-08. |
| TQ21 | iOS — scope, minimum version, devices | n/a | `DEC-02` | Linux only. |
| TQ22 | Android — scope, minimum version, devices | n/a | `DEC-02` | Linux only. |
| TQ23 | Tablet — scope, orientations, devices | n/a | `DEC-02` | Linux only. |
| TQ24 | E-reader — scope, engine, how it is tested | n/a | `DEC-02` | Linux only. |
