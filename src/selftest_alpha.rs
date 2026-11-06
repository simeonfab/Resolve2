//! Checks for the alpha release: the 8-question cap, handoffs written for the request, file
//! import (reading .md/.txt/.docx/.pdf, sorting, moving, attachments feeding the intake), and the
//! problem report carrying no personal data.

use crate::model::*;
use crate::resolve::{files, intake::Attachment, journey, roles, Intake, Kind, ProjectContext, Record};
use crate::selftest::T;
use std::io::Write;

pub fn run(t: &mut T, base: &std::path::Path) {
    println!("\n-- Alpha release --");

    // ---- question cap: never more than 8, never the same question twice
    let mut i = Intake { id: new_id(), input: "I want to sell candles online.".into(), stage: "questions".into(), ..Default::default() };
    let mut asked = 0;
    for _ in 0..12 {
        let out = journey::run_answer(None, &i, i.qas.last().map(|q| q.id.clone()).as_deref(), true);
        journey::apply_answer(&mut i, out);
        if let Some(q) = i.qas.last_mut() {
            if q.answer.is_empty() {
                q.answer = "An answer.".into();
            }
        }
        asked = i.qas.len();
        if i.stage != "questions" {
            break;
        }
    }
    t.check("the intake never asks more than 8 questions", asked <= journey::MAX_QUESTIONS, asked);
    let mut big = Intake { id: new_id(), stage: "questions".into(), ..Default::default() };
    for n in 0..8 {
        big.qas.push(journey::Qa { id: new_id(), question: format!("Question number {n} about something different {n}"), answer: "yes".into(), ..Default::default() });
    }
    let last = big.qas.last().map(|q| q.id.clone());
    let out = journey::run_answer(None, &big, last.as_deref(), true);
    t.check("after the 8th answer there is no 9th question", out.next.is_none() && out.done, format!("{:?}", out.next.map(|q| q.question)));
    journey::apply_answer(&mut big, journey::AnswerOut { heard: Some((last.unwrap_or_default(), vec![])), ..Default::default() });
    t.check("after the 8th answer the intake moves on to the proposal", big.stage == "confirm", &big.stage);
    let mut dup = Intake { id: new_id(), stage: "questions".into(), ..Default::default() };
    dup.qas.push(journey::Qa { id: new_id(), question: "Who exactly is this for, and what do they do today?".into(), answer: "Cafés".into(), ..Default::default() });
    let again = journey::Qa { id: new_id(), question: "Who exactly is this for and what do they do today".into(), ..Default::default() };
    journey::apply_answer(&mut dup, journey::AnswerOut { next: Some(again), ..Default::default() });
    t.check("a question with the same meaning is never asked again", dup.qas.len() == 1, dup.qas.len());

    // ---- handoffs: two requests, two different briefs
    let mut d = WorkspaceData::default();
    let pid = new_id();
    d.projects.push(Project { id: pid.clone(), name: "Café bookkeeping".into(), updated: now_ms(), ..Default::default() });
    let mut ctx = ProjectContext { id: new_id(), project_id: pid.clone(), outcome: "Café owners know their monthly cost".into(), ..Default::default() };
    ctx.understanding.goal = "A flat monthly fee".into();
    d.contexts.push(ctx.clone());
    let mut t1 = Task::new("Set up two-factor login for the client portal");
    t1.project_id = Some(pid.clone());
    d.tasks.push(t1);
    let mut r = Record::new(Kind::Question, Some(pid.clone()), "How do we find cafés beyond word of mouth?");
    r.body = "Marketing channels are unknown".into();
    d.records.push(r);
    let a = roles::handoff_for(&d, &ctx, "Claude Code", "Security hardening of logins");
    let b = roles::handoff_for(&d, &ctx, "Sam", "Marketing beyond word of mouth");
    t.check("two different handoff requests give two different briefs", a != b && a.contains("Security hardening") && b.contains("Marketing"), "same text");
    t.check("each brief pulls in what the project has on that request", a.contains("two-factor") && !a.contains("word of mouth?") && b.contains("word of mouth?") && !b.contains("two-factor"), format!("A:\n{a}\n\nB:\n{b}"));

    // ---- files: read each format, sort, move
    let dir = base.join("files");
    let _ = std::fs::create_dir_all(&dir);
    let _ = std::fs::write(dir.join("Customer interview notes.md"), "# Interviews\nRosa would pay monthly.\n");
    let _ = std::fs::write(dir.join("Requirements.txt"), "Must include VAT returns.\n");
    {
        let f = std::fs::File::create(dir.join("Results.docx")).unwrap();
        let mut z = zip::ZipWriter::new(f);
        let opts: zip::write::SimpleFileOptions = zip::write::SimpleFileOptions::default();
        let _ = z.start_file("word/document.xml", opts);
        let _ = z.write_all(br#"<w:document><w:body><w:p><w:pPr><w:pStyle w:val="Heading1"/></w:pPr><w:r><w:t>Hours log</w:t></w:r></w:p><w:p><w:r><w:t xml:space="preserve">Average 4.5 hours &amp; peak 11.</w:t></w:r></w:p></w:body></w:document>"#);
        let _ = z.finish();
    }
    {
        let content = b"BT /F1 12 Tf 72 720 Td (Step one: collect the receipts every month.) Tj 0 -18 Td (Step two: reconcile the bank feed and the VAT figures.) Tj ET";
        let mut enc = flate2::write::ZlibEncoder::new(vec![], flate2::Compression::default());
        let _ = enc.write_all(content);
        let stream = enc.finish().unwrap_or_default();
        let mut pdf = b"%PDF-1.4\n1 0 obj\n<< /Length ".to_vec();
        pdf.extend_from_slice(format!("{} /Filter /FlateDecode >>\nstream\n", stream.len()).as_bytes());
        pdf.extend_from_slice(&stream);
        pdf.extend_from_slice(b"\nendstream\nendobj\n%%EOF\n");
        let _ = std::fs::write(dir.join("Process guide.pdf"), pdf);
    }
    let md = files::extract(&dir.join("Customer interview notes.md"));
    let txt = files::extract(&dir.join("Requirements.txt"));
    let docx = files::extract(&dir.join("Results.docx"));
    let pdf = files::extract(&dir.join("Process guide.pdf"));
    t.check("reads .md and .txt files", md.as_ref().map_or(false, |x| x.1.contains("Rosa")) && txt.as_ref().map_or(false, |x| x.1.contains("VAT")), format!("{md:?} {txt:?}"));
    t.check("reads .docx text (headings and paragraphs)", docx.as_ref().map_or(false, |x| x.1.contains("## Hours log") && x.1.contains("4.5 hours & peak")), format!("{docx:?}"));
    t.check("reads .pdf text (compressed content streams)", pdf.as_ref().map_or(false, |x| x.1.contains("collect the receipts") && x.1.contains("reconcile the bank feed")), format!("{pdf:?}"));
    t.check("an unreadable file says so plainly", files::extract(&dir.join("missing.pdf")).is_err() && files::extract(std::path::Path::new("x.exe")).is_err(), "");
    let g = files::guess("Customer interview notes", "");
    t.check("without AI, a file is sorted by its name and says it was a guess", g.0 == "research" && g.1.contains("guess"), format!("{g:?}"));
    let page = files::place(&mut d, Some(&pid), "Results", "Average 4.5 hours.", "evidence", "guess");
    let ev = d.records.iter().any(|r| r.kind == Kind::Evidence && r.links.contains(&page));
    t.check("a file sorted as evidence also becomes an Evidence record linked to it", ev && d.page(&page).map_or(false, |p| p.tags.contains(&"evidence".to_string())), "");
    files::recategorise(&mut d, &page, "reference");
    let ev2 = d.records.iter().any(|r| r.kind == Kind::Evidence && r.links.contains(&page));
    t.check("moving a file with one click changes its place (and drops the evidence record)", !ev2 && d.page(&page).and_then(files::category_of) == Some("reference"), "");

    // ---- intake attachments feed the questions and the proposal like typed text
    let mut it = Intake { id: new_id(), input: "My bookkeeping idea.".into(), ..Default::default() };
    it.attachments.push(Attachment { name: "Brief".into(), text: "Cafés pay £150 a month.".into(), place: "brief".into(), how: "guess".into(), why: String::new() });
    let tr = it.transcript();
    t.check("an attached file is part of what the AI reads", tr.contains("Cafés pay £150") && tr.contains("evidence only where the file states it as a fact"), &tr);

    // ---- the problem report: version, Windows, AI, last error — no personal data
    crate::ai::set_last_error("test error");
    let rep = format!("Version: {}\nLast error: {}", crate::store::VERSION, crate::ai::last_error());
    t.check("the version is set (shown in Help and Settings)", crate::store::VERSION.starts_with("Alpha"), crate::store::VERSION);
    t.check("the last error is remembered for Report a problem", rep.contains("test error"), &rep);
}
