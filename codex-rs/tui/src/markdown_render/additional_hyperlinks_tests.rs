use super::*;
use crate::history_cell::HistoryCell;
use crate::terminal_hyperlinks::decorate_spans;

fn policy() -> HyperlinkPolicy {
    HyperlinkPolicy(serde_json::from_str(r#"["vscode", "file", "custom"]"#).unwrap())
}

fn render(source: &str, width: usize, hide: bool) -> Vec<HyperlinkLine> {
    render_markdown_lines_with_policy(source, Some(width), None, &|_| hide, &policy())
}

fn assert_destination(lines: &[HyperlinkLine], destination: &str) {
    let mut count = 0;
    for line in lines {
        for link in &line.hyperlinks {
            assert!(link.columns.end <= line.width());
            assert_eq!(link.terminal_destination().as_deref(), Some(destination));
            count += 1;
        }
        if !line.hyperlinks.is_empty() {
            let encoded = decorate_spans(line)
                .iter()
                .map(|span| span.content.as_ref())
                .collect::<String>();
            assert!(encoded.contains(&format!("\x1b]8;;{destination}\x07")));
        }
    }
    assert!(count > 0);
}

#[test]
fn opted_in_hyperlinks_preserve_urls_through_nested_labels_wrapping_and_tables() {
    for destination in [
        "vscode://file/tmp/a%20b.rs:12:3?q=one#L12",
        "file:///outside/workspace/a%20b.rs:12:3?q=one#L12",
        "custom:opaque-value?query=one#fragment",
    ] {
        for width in [12, 80] {
            for source in [
                format!("See [**répo** and `main`]({destination})."),
                format!(
                    "| Item | Link |\n| --- | --- |\n| source | [**répo** and `main`]({destination}) |"
                ),
                format!("| [open]({destination}) |\n| --- |"),
            ] {
                let lines = render(&source, width, true);
                assert_destination(&lines, destination);
                assert!(
                    !visible_lines(lines)
                        .iter()
                        .any(|line| line.to_string().contains(destination))
                );
            }
        }
    }
}

#[test]
fn opted_in_file_urls_follow_the_same_visible_destination_fallback_as_web_links() {
    for destination in [
        "https://example.com/a",
        "vscode://file/tmp/a.rs",
        "file:///tmp/a.rs",
        "custom:opaque",
    ] {
        let source = format!("[open]({destination})");
        let lines = render(&source, 120, false);
        assert_eq!(
            visible_lines(lines.clone())[0].to_string(),
            format!("open ({destination})")
        );
        assert_destination(&lines, destination);
        assert_eq!(render(&source, 120, true)[0].line.to_string(), "open");
    }
}

#[test]
fn additional_hyperlinks_keep_copy_wrappers_and_source_ranges() {
    let destination = "vscode://file/tmp/a%20b.rs:12:3?q=one#L12";
    let lines = render(
        &format!("See [**répo** and `main`]({destination})."),
        12,
        true,
    );
    let mut selected = Vec::new();
    let mut plain = String::new();
    for line in &lines {
        let source = line.source.clone().unwrap();
        plain.push_str(&source.text[source.range.clone()]);
        crate::markdown_copy::SelectedLine::append(
            &mut selected,
            source.clone(),
            source.range.clone(),
            "",
        );
    }
    let (copied, _) = crate::markdown_copy::selection(&selected, &plain);
    assert!(copied.contains(destination));
    assert!(copied.contains("répo"));
    assert!(copied.contains("`main`"));
    assert_eq!(
        crate::markdown_copy::literal_selection(&selected, &plain),
        "See répo and main."
    );
    assert_eq!(
        copied,
        format!("See [**répo** and `main`](<{destination}>).")
    );
}

#[test]
fn additional_hyperlinks_only_mark_the_label_columns_and_reach_the_terminal_buffer() {
    let destination = "custom:opaque";
    let lines = render(
        &format!("See [**répo** and `main`]({destination})."),
        80,
        true,
    );
    assert_eq!(lines[0].line.to_string(), "See répo and main.");
    let linked_columns = lines[0]
        .hyperlinks
        .iter()
        .flat_map(|link| link.columns.clone())
        .collect::<Vec<_>>();
    assert_eq!(linked_columns, (4..17).collect::<Vec<_>>());
    let mut line = HyperlinkLine::new(Line::default());
    line.push_span_with_policy("open".into(), Some(destination), &policy());
    let mut buffer = ratatui::buffer::Buffer::with_lines(["open"]);
    let area = buffer.area;
    crate::terminal_hyperlinks::mark_buffer_hyperlinks(&mut buffer, area, &[line], 0);
    assert!(
        buffer[(0, 0)]
            .symbol()
            .contains(&format!("\x1b]8;;{destination}\x07"))
    );
    assert_eq!(
        crate::terminal_hyperlinks::osc8_hyperlink(destination, "open"),
        "open"
    );
}

#[test]
fn additional_hyperlinks_leave_disabled_protocols_and_plain_paths_unchanged() {
    for source in [
        "[open](disabled:opaque)",
        "[source](/tmp/a.rs:12)",
        "[source](relative/a.rs)",
        "file:///tmp/a.rs",
    ] {
        let default = render_markdown_lines_with_width_and_cwd(source, Some(80), None);
        let opted_in =
            render_markdown_lines_with_policy(source, Some(80), None, &|_| false, &policy());
        assert_eq!(default, opted_in);
    }
}

#[test]
fn additional_hyperlinks_survive_stream_finalization_and_history_reflow() {
    let destination = "vscode://file/tmp/a.rs:12:3?query=one#fragment";
    let source = format!("See [open]({destination}).\n\nDone.\n");
    let mut stream = crate::streaming::controller::StreamController::new(
        Some(30),
        Path::new("/tmp"),
        crate::history_cell::HistoryRenderMode::Rich,
    )
    .with_hyperlink_policy(policy());
    for delta in source.as_bytes().chunks(7) {
        stream.push(std::str::from_utf8(delta).unwrap());
    }
    let (cell, final_source) = stream.finalize();
    assert_eq!(final_source.as_deref(), Some(source.as_str()));
    assert_destination(&cell.unwrap().transcript_hyperlink_lines(30), destination);
    let cell = crate::history_cell::AgentMarkdownCell::new(source, Path::new("/tmp"))
        .with_hyperlink_policy(policy());
    for width in [12, 80] {
        assert_destination(&cell.transcript_hyperlink_lines(width), destination);
    }
}

#[test]
fn additional_hyperlinks_keep_validation_and_existing_trusted_capabilities_separate() {
    let policy = policy();
    for destination in ["file:relative", "file:", "https://", "disabled:opaque"] {
        assert_eq!(policy.destination(destination), None, "{destination}");
    }
    let oversized = format!("custom:{}", "a".repeat(8192));
    assert_eq!(policy.destination(&oversized), None);
    assert_eq!(
        policy.destination("custom:opaque\u{7}value"),
        Some("custom:opaquevalue".into())
    );
    assert_eq!(
        HyperlinkPolicy::default().destination("file:///tmp/a.rs"),
        None
    );
    let mut line = HyperlinkLine::new(Line::default());
    line.push_span_with_policy("open".into(), Some("file:///tmp/a.rs"), &policy);
    assert_destination(&[line], "file:///tmp/a.rs");
}
