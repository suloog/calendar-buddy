<!-- Copy to docs/specs/NFR.md. Content in the project language;
     headings fixed. Delete groups that have no requirements. -->

# Non-functional requirements

> Project-wide. A requirement that constrains one module only lives in that module's file as
> `MOD-NN-NFR-nn`, not here.
>
> Every requirement here is priority **must** unless marked otherwise. Only list constraints that
> actually shape the design — an NFR nobody can fail is noise.

## Performance

- [NFR-01] The interface stays responsive while the app synchronizes with Google or waits for the network. (`SRC-01-87`)
- [NFR-02] The app uses little CPU when idle and modest memory, starts fast and switches views fast, also with many events. (`SRC-01-10`, `SRC-01-88`, `SRC-01-89`)
- [NFR-03] Thresholds for start-up, switching period or view, and scrolling the week view are set after the first measurements, on a profile of up to 5 accounts, 50 calendars and about 20,000 cached events. (`SRC-01-96`) — *source: `DEC-14`*

## Security

- [NFR-04] Tokens are kept only in the platform's secret store, never in plaintext files; when no secret store is running, sign-in stops with an explanation of what to install or start. (`SRC-01-21`) — *source: `DEC-47`*
- [NFR-05] The app asks Google only for the minimum permissions its features need. (`SRC-01-19`, `SRC-01-92`)
- [NFR-06] The app's only network traffic is HTTPS to Google's APIs. — *source: `DEC-16`*
- [NFR-07] Logs are local, with a level the user sets, and never contain tokens or event contents; there is no telemetry and no remote crash reporting. (`SRC-01-93`) — *source: `DEC-13`*
- [NFR-08] The local cache is not encrypted by the app (the user's disk encryption covers it); removing an account deletes its tokens, cached data and local settings. — *source: `DEC-15`, `DEC-20`*

## Accessibility

- [NFR-09] Every feature can be operated from the keyboard. (`SRC-01-46`) — *source: `DEC-11`*
- [NFR-10] Text and controls have readable contrast in the light and dark themes and scale correctly on HiDPI screens; screen-reader support is not required. (`SRC-01-48`) — *source: `DEC-11`*

## Compatibility

- [NFR-11] The app installs from a .deb package and works on Ubuntu 26.10 with KDE Plasma, on Wayland and on X11. (`SRC-01-06`, `SRC-01-95`, `SRC-01-108`) — *source: `DEC-12`*
- [NFR-12] The app behaves as a native desktop application: system notifications, the system theme, window resizing. (`SRC-01-09`)

## Maintainability

- [NFR-13] Platform-specific functionality sits behind interfaces, so macOS and Windows remain possible from the same codebase. (`SRC-01-05`, `SRC-01-07`)
- [NFR-14] The calendar-provider integration is separate from the app's own model, and provider-specific features are optional. (`SRC-01-13`, `SRC-01-86`)
- [NFR-15] A new version is delivered as a new .deb package installed by hand; there is no maintenance contract. — *source: `DEC-07`, `DEC-46`*

## Availability and scalability

- [NFR-16] The app recovers on its own from network failures, Google API quotas, and expired or revoked access. (`SRC-01-32`, `SRC-01-90`)
- [NFR-17] There are no backups: Google is the system of record, and the cache is rebuilt by resynchronization. — *source: `DEC-15`*

## Compliance and audit

- [NFR-18] Personal data, including guests' e-mail addresses, stays on the user's device and in Google; nothing is processed by us. — *source: `DEC-16`*
