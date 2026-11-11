# Careers intake: one real run on Claude Code

10 November 2026, build 0.8.0 (careers/s2-intake). Command, in a sandbox folder under %TEMP%:

```n resolve2 --testlab --data-dir <sandbox> --lab-run "Job search (Careers)" --provider claude-code --adopt
```

The scenario's notes, answers and CV are fictional test data (resolveos/intake-scenarios.json). Everything the AI wrote below is Claude Code's real output, unedited.

```nprovider: claude-code / 
Q1 [pay] What's the lowest yearly base salary, in pounds, you'd accept for a Head of Product role?
    why: Your pay floor is usually a hard gate, and it's still blank in your notes, so I can't screen out jobs that pay too little.
    A: At least 95,000 base salary.
    time: 9.7s [careers:classify 3.4s, intake:next-question 6.3s]
Q2 [contract] Do you want a permanent role only, or would you also consider a fixed-term or contract role?
    why: This sets whether contract type is a hard gate, so we can drop jobs that don't fit before they reach you.
    A: Permanent only. I won't take contract or interim roles.
    time: 14.7s [careers:classify 6.3s, intake:next-question 8.4s]
Q3 [disciplines] Besides Head of Product, which other job titles would you also consider, such as Director of Platform, Group Product Manager or Head of Partnerships?
    why: This sharpens your role gate by showing whether neighbouring titles can pass, so good jobs aren't rejected only because of the title.
    A: I'd also look at Director of Product Operations, but not pure delivery roles.
    time: 15.1s [careers:classify 6.6s, intake:next-question 8.6s]
Resolve suggests stopping: coverage is good enough.
heard (gate) At least £95,000 base salary
heard (gate) Permanent roles only
heard (gate) No contract or interim roles
heard (preference) I'd also consider Director of Product Operations
heard (gate) No pure delivery roles
skipped (02b answered them): 5
    time: 12.4s [careers:cards 12.4s]
04 The job you want: You want to be Head of Product in identity or platform products, stepping up from Senior Product Manager.
04 Your hard gates: At least £95,000 base · Permanent only · No contract or interim · No pure delivery roles · No agencies
04 Your preferences: Birmingham, hybrid · Two office days a week at most · Occasional London is fine · Scale-ups and larger companies · Would also consider Director of Product Operations
04 What a win looks like: A permanent Head of Product role (or Director of Product Operations) in identity or platform, at £95,000+ base, by March.
04 Strengths: Your CV shows platform ownership end to end and partner programmes that grew partner-sourced revenue by 40%, plus an identity sign-in journey used by 3 million people that cut recovery calls by 25%.
04 Gaps: You're pitching for Head of Product, but your titles and LinkedIn headline still read Senior Product Manager / Senior Product Owner, leadership of a team of 6 is buried, and the level gap may be why few applications reach a first interview.
04 What to fix: Lead with scope and leadership: reframe your headline and titles around platform and identity product leadership, and put the team-of-6 and business outcomes up front.
04 broad: false
    time: 19.4s [careers:kit-assets 19.4s, careers:kit-search 11.2s]
06 Do this first: Rewrite your LinkedIn headline and CV titles around platform and identity product leadership. (why: You send lots of applications but get very few first interviews, and your headline and titles still read Senior Product Manager / Senior Product Owner, so the level gap is likely stopping you before the first call.)
07 Hard gate: Base salary of at least £95,000 (why: You told us £95,000 is the lowest base you will accept for a Head of Product role.)
07 Hard gate: Permanent roles only (why: You will not take contract or interim work.)
07 Hard gate: No pure delivery roles (why: You would consider Director of Product Operations but not a pure delivery role.)
07 Hard gate: No agencies (why: You said you do not want to work for agencies.)
07 Hard gate: Head of Product, or Director of Product Operations (why: These are the two titles you are targeting for this step up.)
07 Preference: Identity or platform products (why: This is where your platform ownership and 3-million-user identity sign-in work are strongest.)
07 Preference: Birmingham, hybrid, two office days a week at most, with occasional London fine (why: This matches where you are based and how often you want to be in an office.)
07 Preference: Scale-ups and larger companies (why: You said these are the kinds of company you want to work for.)
08 Fix your LinkedIn headline · Mon
08 Rebuild your CV on the Head of Product (Platform and Identity) template · Tue
08 Set your weekly target: 10 applications · Tue
08 Run your first daily hunt · Wed
08 Apply to your first three roles · Thu to Fri
09 Search name: Head of Product search
09 Gates: 5 hard gates · 3 preferences
09 CV templates: Head of Product (Platform and Identity) · Director of Product Operations
09 Cover letter template: Hook, proof, fit, close
09 Evidence bank: 5 proof points · 1 need a number
09 LinkedIn fixes: Headline · About · 2 roles
09 Weekly target: 10 applications · 20 found a day
   template Head of Product (Platform and Identity) (default): suits Head of Product roles in identity or platform products at scale-ups and larger companies | headline Head of Product | Platform and Identity | Led a team of 6, grew partner-sourced revenue 40%
   template Director of Product Operations: suits Director of Product Operations roles, excluding pure delivery | headline Product Leader | Platform Ownership End to End | Partner Programmes and Identity Journeys
   proof: Owned the managed services platform end to end [Platform ownership] needs a number | Senior Product Manager, Northfield Software
   proof: Ran the partner programme and grew partner-sourced revenue [Partner programmes and commercial growth] 40% | Senior Product Manager, Northfield Software
   proof: Led a team of 6 [Leadership] 6 | Senior Product Manager, Northfield Software
   proof: Shipped a sign-in journey for a national identity service used by 3 million people [Identity journeys] 3 million | Senior Product Owner, Brightway Consulting
   proof: Cut account recovery calls through the identity sign-in journey [Identity and operational impact] 25% | Senior Product Owner, Brightway Consulting
   LinkedIn Headline: Replace 'Senior Product Owner | Agile | Platforms' with a platform and identity product leadership headline that signals the Head of Product level, for example 'Product Leader | Platform and Identity | Led a team of 6'.
   LinkedIn About: Open with scope and leadership: platform ownership end to end, leading a team of 6, the 40% partner-sourced revenue growth, and the identity sign-in journey used by 3 million people that cut recovery calls by 25%. Frame the story as stepping up to Head of Product.
   LinkedIn Senior Product Manager, Northfield Software: Lead the role with the team-of-6 leadership and the business outcomes rather than burying them, so the scope reads at Head of Product level.
   LinkedIn Senior Product Owner, Brightway Consulting: Foreground the identity work and the 3 million user scale and 25% reduction in recovery calls, so your identity depth is clear up front.
    time: 9.3s [careers:rebuild-cv 9.3s]
Rebuild my CV:
# Alex Morgan
Birmingham

## Head of Product (Platform and Identity)

## Profile
Product leader who builds and runs platform and identity products end to end. I lead teams and own commercial outcomes, from a managed services platform and partner programme to a national identity sign-in service used by millions. Comfortable stepping up to set product direction and grow the business.

## Experience

### Senior Product Manager, Northfield Software (2022 to now)
- Led a team of 6.
- Grew partner-sourced revenue by 40% in two years by running the partner programme.
- Owned the managed services platform end to end.

### Senior Product Owner, Brightway Consulting (2019 to 2022)
- Shipped a sign-in journey for a national identity service used by 3 million people.
- Cut account recovery calls by 25%.
adopted → workspace "Job search" (mode Careers), search "Head of Product search": 8 gates, 2 templates, 5 proof points, 5 tasks
exported: C:\Users\simeo\AppData\Local\Temp\resolve2-careers-run-20261110-231535\exports\run-The-job-I-want_-Head-of-Product,-in-iden-2981b8.md
          C:\Users\simeo\AppData\Local\Temp\resolve2-careers-run-20261110-231535\exports\run-The-job-I-want_-Head-of-Product,-in-iden-2981b8.json
seconds: 85.5
```n