# Handoff: Security hardening of the client data and logins

**For:** Claude Code. This brief is the source of truth; confirm scope before deviating.

## The task
Harden the security of the café bookkeeping client data and the logins used to access it.

## Why it matters
The project turns hourly café bookkeeping into a fixed ~£150/month package, starting with three existing clients. The confirmed outcome is that café owners know their monthly cost up front and send monthly records instead of a year-end 'shoebox' of receipts. Reconstructing 12 months of hours per café (the recommended first action) and running a flat-fee trial both depend on holding client financial records, so that data and the access to it need to be protected.

## Scope for this request
Because the material names no specific system, confirm the targets first (see open questions), then cover:
- The store(s) holding client bookkeeping data and reconstructed hours/records.
- The login(s) and credentials used to reach that data.
- Access controls, authentication strength, and protection of the data at rest and in transit.

## What the project already has on this
- Experiment [Planned] E0: Reconstruct 12 months of actual hours per café — will generate client data to protect.
- A "Project brief" page.
- No security tooling, system, repository, app, database, or login mechanism is described anywhere in the material.

## Constraints
- Stay inside this task; do not widen it into pricing, client conversion, or outreach work.
- Do not invent requirements or systems — if a target isn't named, list it as an open question rather than assuming one.
- Treat assumptions as assumptions, not evidence.
- Do-not-do-yet: the material lists no explicit do-not list; flag anything you're unsure is in scope before acting.
- Client financial data is involved — avoid any change that could expose or lose records.

## Open questions (not stated in the material)
- What system(s) actually hold the client data (spreadsheets, accounting software, cloud drive, local files)?
- What logins exist, and who has access?
- Is there a specific threat, incident, or compliance driver (e.g. UK data protection for client financial records) behind this request?
- What platform/tools are available to implement hardening?
- Any existing passwords, 2FA, or backup arrangements?

## Done when
- [ ] The data store(s) and login(s) in scope are identified and confirmed.
- [ ] Security hardening of that client data and those logins is implemented, with how you verified it shown.
- [ ] Open questions above are answered or carried forward.
- [ ] Anything you couldn't do, or decided differently, is listed in your report.

(written by claude-code in 18.8s)
