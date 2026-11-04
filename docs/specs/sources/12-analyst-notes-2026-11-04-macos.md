---
id: SRC-12
title: Analyst notes — product owner's request 2026-11-04, the app on macOS
kind: informal
origin: analyst — Maciej (also the product owner)
date: 2026-11-04
received_via: conversation, 2026-11-04
supersedes:
status: read-only
---

# [SRC-12] Analyst notes — product owner's request 2026-11-04, the app on macOS

## Summary

The product owner works on a Mac and asked how to install the app so that it works in the system. On learning that the app is built for Linux only, they chose to add macOS support rather than only trying the app as it is or limiting it to local calendars, approved a first stage that installs the app and keeps sign-ins in the Keychain, with the menu bar icon and the rest left for later, and asked for the work to be recorded as requirements and user stories.

## Items

- [SRC-12-01] "Jak zainstalować aplikację żeby działała w systemie?" — the product owner wants to install the app so that it works in the system. — *conversation 2026-11-04*
- [SRC-12-02] "Jestem na macu." — the product owner's computer is a Mac. — *conversation 2026-11-04*
- [SRC-12-03] Of three ways — real macOS support, only trying the app as it is, or local calendars only — the product owner chose real macOS support: the Keychain for sign-ins, the app in `~/Applications`, starting at login, the menu bar icon, notifications. — *conversation 2026-11-04*
- [SRC-12-04] The first stage, approved as a plan: one `cargo install` and one start put "Calendar Buddy.app" in `~/Applications`; sign-ins are kept in the Keychain; "Start at login" works; the menu bar icon and staying in the background, notifications with buttons, sleep and network signals and `webcal:` links come in a second stage. — *conversation 2026-11-04, approved plan*
- [SRC-12-05] "Czy te zmiany nie wymagały zmian w requirements albo nowych US?" — "Tak": the work goes through the spec workflow, with a source, requirements in a new module, user stories and test cases, and the existing requirements brought up to date. — *conversation 2026-11-04*
