# Phase 3 speed fix (D5): the validation half on Claude Code's default model and on Sonnet

10 Nov 2026. Branch `intake/p3-redesign`. **The setting is off by default**: Settings → AI → "Faster proposal: write the validation half on Sonnet (Claude Code and the Claude API only)". You decide from this.

## How it was run

- **Every Test Intake scenario** (5), on Claude Code, with `resolve2 --testlab --speed-compare "<scenario>" --max-q 3`.
- For each scenario, the questions (3 answered from the scenario) and the understanding ran **once**. Then the proposal's **validation half** (H0, hypotheses, experiments, uncertainties, pages) was written **twice from that same understanding**: first on Claude Code's own default model, then on Sonnet. So the model is the only difference between the two.
- On this PC Claude Code's default model is an Opus model. Its own answer named it `claude-opus-4-8`.
- These are real Claude Code answers; nothing below was written or edited by me. The files are `shots/p3/speed/scenario1.md` to `scenario5.md` (shots/ isn't committed; the text is copied in full below).
- **Caveats:**
  - **One run per model per scenario**, so the timings are single samples.
  - Scenarios 1 to 4 ran two at a time; scenario 5 ran alone. Its first try hit this PC's **real Claude Code weekly limit**, and I re-ran it after the limit reset.
  - The plan half isn't part of this comparison; it stays on the default model either way.
  - In the app, `sanitise` then shortens long titles, keeping the full text as the detail, and removes em dashes. The text below is the raw answer, before that.

## Timings

| Scenario | Default model | Sonnet | Sonnet against default |
|---|---|---|---|
| New consumer app idea | 41.4 s | 61.1 s | 19.7 s slower |
| Rescuing a stalled project | 125.6 s | 51.2 s | 74.4 s faster |
| Service business | 51.1 s | 72.2 s | 21.1 s slower |
| Internal work project | 39.8 s | 56.0 s | 16.2 s slower |
| Doom Clock corporate events | 86.5 s | 51.9 s | 34.6 s faster |
| **Total** | **344.4 s** | **292.4 s** | |
| **Median** | **51.1 s** | **56.0 s** | |

Sonnet was faster in 2 of 5 scenarios and slower in 3. The default model's times swung far more, from 39.8 to 125.6 s; Sonnet's ran from 51.2 to 72.2 s. With one run each, I can't say from this whether either is faster in general (**unverified**).

For comparison, the first measurement (8 Nov, Service business, both halves side by side) had the validation half at 47.0 s.

## What each model wrote (counts)

| Scenario | Default: hypotheses / experiments / uncertainties / pages | Sonnet: hypotheses / experiments / uncertainties / pages |
|---|---|---|
| New consumer app idea | 4 / 2 / 5 / 1 | 4 / 5 / 7 / 3 |
| Rescuing a stalled project | 3 / 2 / 4 / 1 | 4 / 5 / 7 / 3 |
| Service business | 3 / 2 / 5 / 1 | 4 / 5 / 7 / 3 |
| Internal work project | 4 / 2 / 5 / 1 | 4 / 5 / 7 / 3 |
| Doom Clock corporate events | 4 / 2 / 5 / 1 | 4 / 4 / 7 / 2 |

What differs, as written (I'm describing, not judging):

- **Length.** Sonnet writes more items every time, and longer ones: full-sentence titles, negations of two or three sentences. That is most of its time.
- **Titles.** The default model writes short titles ("Workload is month-to-month stable"). Sonnet writes sentences ("The work per café is stable enough month to month that about 5 hours covers it."), which the app would shorten for display.
- **Experiments.** The default model gives an E0 and E1 with one threshold each. Sonnet gives E0 to E4, each with a numeric "Supports it if", often with a sample size.
- **Skipped questions.** Sonnet names the questions you skipped as open uncertainties ("You skipped this question, so…"); the default model does so less often.
- **Em dashes.** The default model's text has em dashes in 3 of 5 scenarios. The app now removes them in every proposal field (`sanitise`, with a self-check).

## The two validations, side by side, per scenario

Each scenario's understanding is first, then the default model's validation, then Sonnet's, from the same understanding.

## New consumer app idea

**Project, as understood:** You want to build a phone app that helps renters split shared household bills fairly with their flatmates, replacing the spreadsheet your flat currently uses.

**Outcome:** 

### Default model (Opus, Claude Code's default): 41.4 s

- **H0:** Bill-split app stops late payments (uncertain). Against it: A phone app does not reduce late payments or arguments, because the real cause of late payments is not something clarity or reminders can fix.
- **Hypothesis:** Late payment cause unknown (evidence). The cause is already known and is a single, specific thing (forgetting, disputed amounts, or payment friction), not a mix.
- **Hypothesis:** Spreadsheet is the bottleneck (evidence). The spreadsheet is fine; people would still pay late with a perfectly up-to-date app, because the real friction is nobody wanting to chase or the payment step itself.
- **Hypothesis:** One person sets up, others follow (assumption). Flatmates won't adopt a tool one person sets up for them, or won't enter/link bills, so the app sits unused like the spreadsheet.
- **Hypothesis:** Three flats share one problem (evidence). The three flats have different problems, so a single app can't serve them, and the sample is too small and too close to you to generalise.
- **Experiment:** E0: Log the real reason. Supports it if No single reason accounts for more than 50% of the 10 cases.
- **Experiment:** E0b: Try an existing app. Supports it if Every flatmate opens it at least once and on-time payment improves versus the spreadsheet baseline.
- **Uncertainty:** Is the core problem reminders, disputed amounts, or paying itself?
- **Uncertainty:** Would flatmates actually enter bills or link a bank, or does that fail like the spreadsheet?
- **Uncertainty:** Why does the spreadsheet go stale — nobody updates it, or nobody reads it?
- **Uncertainty:** Would a free existing app (Splitwise, a group chat reminder) already solve this?
- **Uncertainty:** What would 'it worked' look like — fewer late payments, fewer arguments, or less chasing?
- **Page:** Project brief

### Faster model (Sonnet): 61.1 s

- **H0:** A phone app that keeps what each flatmate owes clear and current will cut late payments and bill arguments in shared rented flats. (uncertain). Against it: The late payments and arguments come from something an app would not change, such as disputes over amounts, people being unable or unwilling to pay, or a flat dynamic that no tool fixes. Or a spreadsheet plus a manual nudge already does the job well enough.
- **Hypothesis:** Late payment is mainly forgetting plus nobody wanting to chase, so timely reminders to each person are the core need. (evidence). Late payment is mainly caused by disputed amounts, disagreement over fairness, or inability to pay, so reminders would not change it.
- **Hypothesis:** An out-of-date spreadsheet leaves flatmates unsure what they owe, and that uncertainty delays payment and fuels arguments. (evidence). Flatmates already know roughly what they owe, so a stale spreadsheet is a side issue and lateness has another cause.
- **Hypothesis:** One person per flat will set the app up for everyone, and bills will get into it by hand or by bank link without that person becoming the chaser again. (assumption). Setup and bill entry will stall on one person, or the other flatmates will not engage, so the app goes stale like the spreadsheet did.
- **Hypothesis:** Flats like yours feel this pain strongly enough to adopt a new tool, and no existing option already solves it well enough. (uncertain). The pain is a mild annoyance, or existing apps, reminders or a better-run spreadsheet would already solve it for most flats.
- **Experiment:** E0: Classify the cause of the last late payments across. Supports it if At least 8 incidents are collected from at least 2 flats and at least 60% are coded forgot, didn't know amount, or chasing avoided.
- **Experiment:** E1: One flat, manual reminder with a clear per-person amount. Supports it if At least 75% of individual payments across the 4 bills arrive on or before the due date, against a baseline at least 25 percentage points lower, or median days late drops by at least half.
- **Experiment:** E2: Can flatmates say what they owe without checking the. Supports it if At least half of the answers are off by more than 10% of the true amount, or the spreadsheet disagrees with the reconciled balance for at least half the people, with at least 6 answers collected.
- **Experiment:** E3: Setup and adoption trial using your sketched screens. Supports it if At least 2 of 3 flats complete setup and enter bills within 7 days and at least 75% of other flatmates view or acknowledge their amount.
- **Experiment:** E4: What have flats already tried. Supports it if At least 8 households interviewed; at least half report having tried or considered an alternative and dropped it for reasons tied to this problem, and the median pain score is 6 or higher.
- **Uncertainty:** What is the real cause of late payment: forgetting, disputed amounts, or inability to pay? You have only reported symptoms so far.
- **Uncertainty:** Have you or your flatmates tried any other tool, and what happened? You did not confirm any alternative had been tried.
- **Uncertainty:** What would tell you it worked? You skipped this question, so success is not yet defined.
- **Uncertainty:** Do the two friends' flats have the same cause as yours, or only the same complaint?
- **Uncertainty:** Will the person who sets the app up end up as the chaser again, and would they accept that role?
- **Uncertainty:** Will flatmates enter bills by hand, link a bank, or do neither? Both are assumptions, and bank linking carries trust and effort costs.
- **Uncertainty:** Are the arguments about money fairness, such as unequal usage or who pays what share, or about the process of chasing?
- **Page:** Project brief
- **Page:** Evidence and assumptions ledger
- **Page:** Open questions

## Rescuing a stalled project

**Project, as understood:** You are redesigning your charity's volunteer sign-up process. A new sign-up form is half-built, with scattered documentation across Google Docs, an outdated Trello board, and the work stalled when the project lead left.

**Outcome:** New volunteers are registered within two days of applying, and office staff stop spending mornings on sign-up calls.

### Default model (Opus, Claude Code's default): 125.6 s

- **H0:** Form redesign solves it (uncertain). Against it: Finishing and launching the redesigned form does not register volunteers in two days or free staff mornings, because the real bottleneck lies elsewhere.
- **Hypothesis:** Form friction drives drop-off (uncertain). The 40% who never finished sign-up abandoned for reasons the form cannot fix (timing, eligibility, follow-up delay, loss of interest), so a better form barely moves completion.
- **Hypothesis:** Sign-up work fills call hours (uncertain). The 6 weekly call-hours cover duties unrelated to sign-up, so automating the form leaves staff mornings largely unchanged.
- **Hypothesis:** Half-built form is salvageable (uncertain). The stalled form is too incomplete or ill-fitting to finish, so restarting from scratch is faster than reviving it.
- **Experiment:** E0: Trace where enquirers drop off. Supports it if A majority of reconstructed drop-off and of reached non-completers points to a step or barrier inside the sign-up form.
- **Experiment:** E0: Log a week of sign-up calls. Supports it if At least half of logged call-time is sign-up tasks a self-serve form could absorb.
- **Uncertainty:** Which of the 6 weekly call-hours is sign-up work a form could absorb
- **Uncertainty:** How much of the half-built form is usable versus redo
- **Uncertainty:** Who owns and decides on the project now the lead has gone
- **Uncertainty:** What 'registered' means and what gates the two-day target
- **Page:** Project brief

### Faster model (Sonnet): 51.2 s

- **H0:** Finishing the redesigned sign-up form will get volunteers registered within two days and take sign-up calls off office staff. (uncertain). Against it: Finishing the form will not achieve this, because the form is not the main cause of the drop-off, the delays or the call load. Something else is, such as response time, staff handling, unclear expectations or volunteer intent. Or the half-built form is not a sound base.
- **Hypothesis:** The 40% drop-off is mainly caused by friction in the current form or sign-up steps, so a better form will recover many of those people. (uncertain). The drop-off is mainly caused by things a form will not fix. Examples are slow replies after enquiry, people changing their minds, unclear expectations of the role, or enquiries that were never serious.
- **Hypothesis:** Most of the roughly 6 hours a week on calls is caused by the sign-up process, such as help with the form and status chasing. A clearer process would remove most of it. (assumption). Most of the call time is not process-driven. It is relationship, screening or placement conversations that staff would still need to have, so a better form removes little of it.
- **Hypothesis:** Slow registration comes mainly from manual handling by the two office staff, so a redesign that reduces manual steps can reach registration within two days. (assumption). Slow registration comes mainly from elsewhere. Examples are waiting on volunteers to respond, checks or references outside the office's control, or the two staff's limited capacity. A form redesign alone will not reach two days.
- **Hypothesis:** Enough of the half-finished form and the Google Docs notes is sound to build on, so finishing is cheaper than restarting. (uncertain). The form and notes are too incomplete, outdated or undocumented to trust, so finishing costs about as much as restarting. The old Trello board is also likely out of date.
- **Experiment:** E0: Is the sign-up form the main constraint at all?. Supports it if At least 50% of non-completers stopped at or within one step of the form, and at least 50% of call time is form- or process-related.
- **Experiment:** E1: Ask non-completers why they stopped. Supports it if At least 10 people respond, and at least half of the coded reasons are form/process-related.
- **Experiment:** E2: Classify a fortnight of sign-up calls. Supports it if At least 60% of logged call time is tagged form help, status chase or missing information.
- **Experiment:** E3: Measure time from application to registration. Supports it if At least 60% of elapsed time on average sits in staff handling or queue wait, and the median is over two days.
- **Experiment:** E4: Audit the half-built form and notes. Supports it if At least 70% of checklist items are done or partial, and no blocking decision is undocumented or contradictory.
- **Uncertainty:** Why did 40% of enquirers never complete sign-up? You skipped this question, so the cause is unknown.
- **Uncertainty:** What does the 40% figure measure, and how reliable is it? You haven't said where it comes from or how 'enquired' and 'completed' were defined.
- **Uncertainty:** What is today's actual time from application to registration? The two-day target has no baseline.
- **Uncertainty:** What share of the 6 weekly call hours is sign-up-related, and what share would remain after a redesign?
- **Uncertainty:** How much of the half-built form is usable, and what was it designed to fix?
- **Uncertainty:** Who owns this now that the project lead has left, and who has to agree before a change goes live? You named only volunteers and the two office staff.
- **Uncertainty:** Did the original project reach any conclusions about why people drop off that are buried in the Google Docs?
- **Page:** Project brief
- **Page:** Evidence and assumptions
- **Page:** Decision log

## Service business

**Project, as understood:** You are a freelance bookkeeper packaging your existing café bookkeeping work into a fixed monthly price (you estimate around £150/café) instead of billing by the hour, starting with your three current café clients.

**Outcome:** Your café clients pay a known fixed fee each month and stop worrying about unpredictable bookkeeping costs, and your own income becomes steady rather than lumpy.

### Default model (Opus, Claude Code's default): 51.1 s

- **H0:** Fixed £150 café package (uncertain). Against it: Packaging your café bookkeeping into a fixed ~£150/month fee will not deliver steady income or real client cost-certainty — because the per-café workload swings, the price is wrong, or the anxiety sits elsewhere.
- **Hypothesis:** Workload is month-to-month stable (assumption). NH1: Hours per café swing widely across the year, so a fixed fee leaves you working well below your effective rate in busy months (VAT quarters, year-end) while clients feel the saving.
- **Hypothesis:** Cafés accept £150 and it covers the work (assumption). NH2: At £150 either cafés hesitate, or once peak-month hours are included the fee earns you less per hour than you make billing hourly today.
- **Hypothesis:** Fixed fee cures the real pain (evidence). NH3: Fixed pricing doesn't remove client cost-nerves (they worry about value, not variability) or smooth your income (lumpiness comes from client count and churn, not billing method).
- **Experiment:** E0: Reconstruct 6–12 months of hours per café. Supports it if Every café's busiest month sits within ±25% of its quietest month.
- **Experiment:** E1: Offer £150 fixed to the two willing clients. Supports it if Both accept near £150 and the peak-month effective rate stays at or above your current hourly rate.
- **Uncertainty:** How far do monthly hours per café actually swing between your busiest and quietest month?
- **Uncertainty:** What drives the peaks — VAT quarters, year-end, seasonal café trade — and are they predictable per client?
- **Uncertainty:** Does £150 cover a peak month, not just an average 5-hour month?
- **Uncertainty:** Would the third client (who hasn't asked for fixed pricing) accept it, or does fixed pricing only suit some cafés?
- **Uncertainty:** What happens when a café's workload genuinely grows — how does the fee adjust without reopening the anxiety you're trying to remove?
- **Page:** Project brief

### Faster model (Sonnet): 72.2 s

- **H0:** Moving your cafés from hourly billing to a fixed monthly fee of about £150 will give them cost certainty and give you steady income. (uncertain). Against it: A fixed fee of about £150 will not deliver both. Either clients still worry about cost, or your hours swing enough that the fee under-pays you in busy months, or your income stays lumpy for reasons billing doesn't touch.
- **Hypothesis:** Hourly billing is what makes your clients nervous about costs and makes your income lumpy. (evidence). The nervousness and lumpiness come from something else, such as variable hours, invoice or payment timing, or having only three clients. Changing the billing basis would not fix them.
- **Hypothesis:** Your café clients will commit to a fixed monthly fee of about £150. (assumption). Fewer than two of your three clients will commit at about £150. Their stated preference for a fixed fee may not hold once there is a real price, scope and start date.
- **Hypothesis:** The work per café is stable enough month to month that about 5 hours covers it. (assumption). Hours swing widely between months, for example around year-end, VAT, staff changes or catch-up work. A flat fee would then under-pay you in the busy months.
- **Hypothesis:** A fixed fee across your current cafés will make your own income noticeably steadier. (uncertain). Your income stays lumpy under fixed fees. Lumpiness comes from other things, such as how many clients you have, one-off jobs or payment timing. Or the fee is set too low to be worth smoothing.
- **Experiment:** E0: Replay the last 6 months at a flat £150. Supports it if In at least 2 of 3 cafés the flat-fee 6-month total is at least 95% of the hourly-billed total, no single month falls below your pre-set minimum effective hourly rate, and total monthly income varies less under flat billing.
- **Experiment:** E1: Track true hours per café, including hidden work. Supports it if For at least 2 of 3 cafés, logged hours stay between 4 and 6.5 each month, and the busiest recalled or logged month is no more than 1.4 times the quietest.
- **Experiment:** E2: Put a concrete £150 offer to all three cafés. Supports it if At least 2 of 3 cafés agree to a start date at £150 or more per month, without asking you to include significantly more work.
- **Experiment:** E3: Find where your lumpy income actually comes from. Supports it if At least half of the month-to-month swing in income is explained by hours varying within the three cafés.
- **Experiment:** E4: Run a 3-month fixed-fee pilot. Supports it if Monthly income from the pilot cafés stays within 10% of the expected total, and every café's effective rate stays at or above your pre-set minimum in at least 2 of the 3 months.
- **Uncertainty:** How much do the monthly hours per café vary? You skipped the busiest-versus-quietest question, and it decides whether a flat fee protects you.
- **Uncertainty:** Does £150 beat what you earn now? £150 for 5 hours is about £30 an hour, but your current hourly rate hasn't been stated, so you can't yet tell whether that is a pay cut.
- **Uncertainty:** Is the 5 hours a month a complete figure? It may leave out unbilled time such as emails, chasing receipts and queries.
- **Uncertainty:** What did the two clients mean by preferring a fixed fee, and why didn't the third? Neither has been tested against a real price.
- **Uncertainty:** What would the package include and exclude, such as year-end work, VAT, payroll or catch-up on messy records? Without this the fee can't be priced or defended.
- **Uncertainty:** Is your lumpy income driven by hours per café, payment timing or the small number of clients? Only the first is fixed by a flat fee.
- **Uncertainty:** Are three existing cafés representative of the new small cafés you would want to sell to?
- **Page:** Project brief
- **Page:** Evidence and assumptions ledger
- **Page:** Experiment pre-registration log

## Internal work project

**Project, as understood:** You have been tasked at work with reducing the time your customer support team takes to give a first reply to emails. Email currently arrives across three different tools, and no one owns triage.

**Outcome:** Customers get a first reply within 4 working hours, and the team stops missing emails — visible in the help desk's response-time report.

### Default model (Opus, Claude Code's default): 39.8 s

- **H0:** Unowned triage across three inboxes (uncertain). Against it: First-reply delay is not mainly caused by unowned triage across three inboxes; it persists even where triage is clear, driven by volume, staffing, or one specific tool.
- **Hypothesis:** Delay concentrates in one tool (uncertain). The 19-hour median is spread roughly evenly across all three tools, so no single inbox explains the delay.
- **Hypothesis:** No triage owner causes misses (assumption). Emails are missed for reasons other than missing ownership (capacity, no SLA, tool gaps), so naming an owner alone would not stop misses.
- **Hypothesis:** Four-hour reply satisfies customers (assumption). A 4-working-hour first reply does not materially change customer satisfaction; the complaints are about resolution time or reply quality, not first-reply speed.
- **Hypothesis:** Report measures first-reply reliably (evidence). The help desk report does not capture first-reply time consistently across all three tools, so the 19-hour median understates or overstates the real wait.
- **Experiment:** E0: Break first-reply time down by tool. Supports it if One tool's median first-reply time is at least 2x the lowest tool's median, OR one tool holds 50%+ of all emails that breached a 4-working-hour reply.
- **Experiment:** E1: Audit missed-email causes on a sample. Supports it if 60%+ of sampled breaches classify as 'never triaged / no owner'.
- **Uncertainty:** Is the 19h median driven by one tool or spread evenly across the three?
- **Uncertainty:** Does the help desk report cover all three tools, or only the one with a report?
- **Uncertainty:** Are the 120 complaints about first-reply speed, or about resolution time and quality?
- **Uncertainty:** What share of emails are missed entirely versus merely slow?
- **Uncertainty:** Is slowness a triage-ownership problem or an inbound-volume-vs-staffing problem?
- **Page:** Project brief

### Faster model (Sonnet): 56.0 s

- **H0:** A 4-working-hour first reply fixes slow replies, and unowned triage across three tools is the main cause (uncertain). Against it: The 4-hour target or the cause is wrong. The delay comes mainly from something else (capacity, one specific tool, email complexity, how the report measures time), or customers are unhappy about something other than first-reply time.
- **Hypothesis:** The delay is concentrated in one or two of the three tools, not spread evenly (uncertain). First-reply times are about the same in all three tools, so the problem is the overall setup and not a particular inbox.
- **Hypothesis:** Waiting for someone to pick up an email (no triage owner) accounts for most of the 19 hours (assumption). Most of the delay happens after an email is picked up (workload, hard questions, handoffs), so adding a triage owner would change little.
- **Hypothesis:** The help desk report measures all three tools in a way comparable to working hours (uncertain). The report misses some emails, for example those in tools that don't feed the help desk. It may also count clock hours, so the true gap to 4 working hours, and the number of missed emails, differ from what you see.
- **Hypothesis:** The 120 survey comments about slow replies are about first-reply time, and a 4-working-hour first reply would address them (assumption). The comments are mostly about other things (time to resolve, follow-ups, other channels), so a faster first reply would not reduce them.
- **Experiment:** E0: How far is today from the 4-working-hour target?. Supports it if Working-hours median is above 8 hours AND fewer than 40% of emails are answered within 4 working hours.
- **Experiment:** E1: Is the delay concentrated in one tool?. Supports it if The slowest tool's median is at least 2x the fastest tool's, and the same ordering holds in both 4-week halves.
- **Experiment:** E2: How much of the wait is before anyone picks. Supports it if In the median email, the pre-pickup wait is at least 60% of total first-reply time.
- **Experiment:** E3: Does the report see every email from all three. Supports it if At least 95% of sampled emails from every tool appear in the report AND the time measure matches working hours.
- **Experiment:** E4: What are the 120 slow-reply comments actually about?. Supports it if At least 60% of comments are coded 'no or late first reply' and the second coder agrees on at least 80% of the sample.
- **Uncertainty:** Does the 19-hour median come mainly from one tool, or is it spread across all three? (Your skipped question.)
- **Uncertainty:** Does the help desk report include emails from all three tools, or are some never counted?
- **Uncertainty:** Is the 19 hours measured in working hours or clock hours, and what share of emails already get a reply within 4 working hours?
- **Uncertainty:** How much of the wait is before anyone picks the email up, and how much is after?
- **Uncertainty:** What would count as 'missed' emails, and is that visible in any report today?
- **Uncertainty:** Do the 120 survey comments mention first reply, or other kinds of slowness?
- **Uncertainty:** Is 4 working hours the right target for customers, or an agreed figure that hasn't been checked?
- **Page:** Project brief
- **Page:** Evidence and assumptions
- **Page:** Measurement definitions

## Doom Clock corporate events

**Project, as understood:** You have designed a tabletop game called Doom Clock, a race-against-time co-op game, and you want to sell it to companies as an on-site team-building event that you run in person.

**Outcome:** 

### Default model (Opus, Claude Code's default): 86.5 s

- **H0:** Doom Clock sells as paid corporate team-building (uncertain). Against it: Companies will not book and pay for Doom Clock as an on-site event
- **Hypothesis:** Quote interest becomes paid bookings (evidence). Quote requests stall before anyone pays and schedules a run event
- **Hypothesis:** Buyers find current team-building forced (assumption). Buyers are satisfied with their current team-building and feel no pull toward Doom Clock
- **Hypothesis:** Games-night praise predicts corporate value (evidence). Casual players enjoying it says nothing about whether HR judges it worth paying for
- **Hypothesis:** 20-200 staff firms are the buyer (assumption). The real buyer and budget sit elsewhere, not HR/team leads at 20-200 staff firms
- **Experiment:** E0: Send a real quote to both HR contacts. Supports it if At least 1 of 2 contacts confirms a booked, dated, paid event within 4 weeks.
- **Experiment:** E1: Run one free or at-cost pilot. Supports it if Participant ratings within range of games-night scores AND sponsor names a specific benefit and states willingness to pay for a repeat.
- **Uncertainty:** What price makes an HR buyer say yes versus hesitate
- **Uncertainty:** What outcome HR must see to call the event worth repeating
- **Uncertainty:** Who signs off budget for a team-building event
- **Uncertainty:** Whether the game holds up with colleagues who did not opt in
- **Uncertainty:** How HR found you and whether that channel repeats
- **Page:** Project brief

### Faster model (Sonnet): 51.9 s

- **H0:** Companies of 20-200 staff will book and pay for Doom Clock as an on-site team-building event that you run in person. (uncertain). Against it: Companies will not book and pay for it. The quote requests and games-night ratings do not turn into paid events, because of price, fit, buyer priorities or how the event is delivered.
- **Hypothesis:** HR and team leads at 20-200 person companies find their current team-building events forced, and would pay for something that gets people talking. (assumption). These buyers are content enough with their current events, or will not pay extra for a different one. 'Forced' is your impression, not their stated problem.
- **Hypothesis:** The two quote requests show real buying intent that can become paid bookings. (uncertain). The two quote requests were casual curiosity or price-checking. They will not become bookings at a price that works for you.
- **Hypothesis:** The enjoyment players showed at games nights will carry over to a workplace group of colleagues, including less willing participants. (assumption). Games-night ratings came from self-selected, willing players. A mixed workplace group, with hierarchy and some people who did not choose to be there, will not respond the same way.
- **Hypothesis:** You can run Doom Clock on site, alone, at a price companies accept and that is worth your time. (uncertain). Preparation, travel, group size limits or your own capacity make the event either too expensive for buyers or not worth running for you.
- **Experiment:** E0: Send a priced quote. Supports it if At least 1 of 2 confirms a specific date and agrees to pay (deposit, signed quote or purchase order) within 21 days of the quote.
- **Experiment:** E1: Interview HR contacts about their last team-building event. Supports it if At least 5 of 8 describe a specific recent event as flat, forced or poorly attended without prompting, and at least 4 of 8 can name a budget or approver.
- **Experiment:** E2: Run a pilot. Supports it if At least 70% rate 4 or 5 on 'would do it again' and at least 60% say they talked to someone they rarely speak to, with the reluctant participants within 1 point of the average.
- **Experiment:** E3: Cost the event and compare it with stated budgets. Supports it if Break-even is at most 80% of the median budget stated by buyers, with at least 4 budgets stated.
- **Uncertainty:** What the two HR contacts actually want, what budget they have, and when they would need the event.
- **Uncertainty:** Who decides and pays: HR, a team lead, or a separate budget holder.
- **Uncertainty:** What a successful event looks like for the buyer and for the employees. You skipped this question, so it is still open.
- **Uncertainty:** Whether games-night players were friends or self-selected, which would make their ratings a weak guide to colleagues.
- **Uncertainty:** What price a company will pay, and what price makes the event worth running for you.
- **Uncertainty:** How many players one run supports, and whether a company group of 20-200 fits a single event.
- **Uncertainty:** What companies use today instead (outings, quizzes, escape rooms, workshops) and what Doom Clock has to beat.
- **Page:** Project brief
- **Page:** Evidence and assumptions log

