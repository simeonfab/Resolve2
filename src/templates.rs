//! Built-in page templates. Any page can also be saved as a template of your own.

pub struct PageTemplate {
    pub name: &'static str,
    pub icon: &'static str,
    pub markdown: &'static str,
}

pub const PAGE_TEMPLATES: [PageTemplate; 6] = [
    PageTemplate {
        name: "Meeting notes",
        icon: "🗒",
        markdown: "## Details\nDate: \nAttendees: \n\n## Agenda\n- \n\n## Notes\n\n## Decisions\n- \n\n## Action items\n- [ ] \n",
    },
    PageTemplate {
        name: "Business plan",
        icon: "📈",
        markdown: "> 💡 One sentence: what you sell, to whom, and why they'll pay.\n\n## Problem\n\n## Solution\n\n## Target customers\n- \n\n## Market & competitors\n\n## Business model\nPricing: \nChannels: \n\n## Go-to-market\n1. \n\n## Financials\nCosts: \nRevenue goal (12 months): \n\n## Milestones\n- [ ] \n\n## Risks\n- \n",
    },
    PageTemplate {
        name: "Product spec",
        icon: "🧩",
        markdown: "## Summary\n\n## Problem & goals\n- \n\n## Non-goals\n- \n\n## Users & use cases\n\n## Requirements\n- [ ] \n\n## Design\n\n## Open questions\n- \n\n## Launch plan\n",
    },
    PageTemplate {
        name: "Project brief",
        icon: "📁",
        markdown: "## Goal\n\n## Why now\n\n## Scope\n- \n\n## Milestones\n- [ ] \n\n## Team\n\n## Links\n",
    },
    PageTemplate {
        name: "Weekly review",
        icon: "🔁",
        markdown: "## Wins\n- \n\n## What slipped\n- \n\n## Next week's big three\n1. \n2. \n3. \n\n## Notes\n",
    },
    PageTemplate {
        name: "Daily journal",
        icon: "📓",
        markdown: "## Grateful for\n- \n\n## Today's focus\n- [ ] \n\n## Notes\n\n## Reflection\n",
    },
];
