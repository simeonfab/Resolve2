# Intake run: You are a freelance bookkeeper packaging your existing per-café bookkeeping work

Provider: claude-code||
Scenario: Service business
Stage: proposal

## Opening

I'm a freelance bookkeeper and I want to turn what I do into a fixed-price monthly package for small cafés, instead of charging by the hour.

## Questions

1. **What's the main thing about charging by the hour that isn't working for you or your café clients today?** _(The problem or opportunity)_
   - Why: This tests the assumption that hourly billing is the real problem (rather than something like unpredictable workload or client demand), which decides whether a fixed-price package would actually fix it.
   - Answer: Hourly billing makes clients nervous about costs and makes my income lumpy.
   - Source: claude-code / default
2. **Can you tell me about a recent time a café client got nervous about a bill from you, and what they said or did?** _(Evidence they have)_
   - Why: This tests the assumption that hourly billing is what makes clients nervous, and not something else like the total cost or how clear your invoices are. A real example shows whether a fixed monthly price would fix it.
   - Answer: Three existing café clients; two said they'd prefer a fixed fee. I spend roughly 5 hours a month per café.
   - Source: claude-code / default
3. **Across those three cafés, does the work take about the same 5 hours every month, or does it swing a lot from month to month?** _(Their assumptions)_
   - Why: A fixed monthly fee assumes the monthly workload is steady enough to price in advance, so this tests that assumption and whether the 5-hour figure can be relied on.
   - Answer: That cafés will pay about £150 a month; that the work per café is predictable.
   - Source: claude-code / default
4. **If the fixed monthly package works well a year from now, what would be different for you and for your café clients, and how would you know?** _(The outcome and how they'd know)_
   - Why: This tests whether a fixed fee actually makes you and the cafés better off, and what you would count as success, which sets the bar for judging the £150 and 'predictable work' assumptions.
   - Answer: (skipped)
   - Source: claude-code / default

## What Resolve heard

- (assumption) Hourly billing makes my clients nervous about costs
- (evidence) Hourly billing makes my income lumpy
- (evidence) Two of my three existing café clients said they'd prefer a fixed fee
- (evidence) I spend roughly 5 hours a month per café
- (assumption) Cafés will pay about £150 a month
- (assumption) The work per café is predictable

## Proposal: Fixed-fee café bookkeeping

Package your existing per-café bookkeeping into a fixed ~£150/month fee, starting with your three current café clients.

**Objective:** Replace hourly billing with a predictable fixed monthly package so your income is steadier and your café clients stop worrying about unpredictable bills, proven first on the clients you already have.

**Outcome:** 

**H0:** Packaging café bookkeeping as fixed fee / NH0: A fixed monthly package fails because the work isn't predictable or priced right, leaving income less steady, not more

### Hypotheses

- [assumption] Per-café work is predictable (NHx: Monthly hours swing enough that a flat fee loses money in heavy months)
- [assumption] Cafés accept £150 fixed fee (NHx: At £150 cafés hesitate, negotiate down, or your two keen clients balk once it's real)
- [evidence] Fixed fee steadies your income (NHx: Fixed pricing smooths billing but drops your effective rate below hourly, so income steadies lower)
- [assumption] Fixed fee calms client cost anxiety (NHx: Clients were anxious about total spend, not billing shape, so a flat fee doesn't settle them)

### Uncertainties

- [uncertain] Does each café really take ~5 hours every month, or does it swing?
- [uncertain] What covers a bad month (VAT quarter, year-end, messy receipts) under one flat fee?
- [uncertain] Does £150 beat your current average hourly take per café?
- [uncertain] Will the two keen clients sign once a real price and terms are in front of them?
- [uncertain] What's included versus extra, so scope creep doesn't eat the margin?

### Experiments

- [uncertain] E0: Log three months of actual hours per café
- [uncertain] E1: Put a priced package to the two keen clients

### Roadmap

- [uncertain] (Now) Check workload predictability
- [assumption] (Now) Price the monthly package
- [uncertain] (Next) Define scope boundaries
- [evidence] (Next) Propose to willing clients
- [assumption] (Later) Convert the third café
- [uncertain] (Later) Standardise for new cafés

### Tasks

- [evidence] (Now) Gather last 12 months of hours
- [uncertain] (Now) Calculate hour variance per café
- [assumption] (Now) Set a floor price from hours
- [uncertain] (Next) List what's included and excluded
- [assumption] (Next) Draft a one-page package offer
- [evidence] (Next) Book talks with the two willing clients

### Pages

- [assumption] Project brief


## AI calls (11)

- intake:next-question · claude-code / (default) · ok
- intake:classify · claude-code / (default) · ok
- intake:next-question · claude-code / (default) · ok
- intake:classify · claude-code / (default) · ok
- intake:next-question · claude-code / (default) · ok
- intake:classify · claude-code / (default) · ok
- intake:next-question · claude-code / (default) · ok
- intake:understand · claude-code / (default) · ok
- intake:proposal:plan · claude-code / (default) · ok
- intake:proposal:validation · claude-code / (default) · ok
- intake:proposal:validation · claude-code / (default) · ok
