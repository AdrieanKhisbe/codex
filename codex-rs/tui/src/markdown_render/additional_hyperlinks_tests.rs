use super::*;
use crate::history_cell::HistoryCell;
use crate::terminal_hyperlinks::decorate_spans;

fn protocols() -> codex_config::AdditionalHyperlinkProtocols {
    serde_json::from_str(r#"["vscode", "file", "custom"]"#).unwrap()
}

struct ResetPreferences;
impl Drop for ResetPreferences {
    fn drop(&mut self) {
        preferences::init(Default::default(), &Default::default());
    }
}

fn enable_protocols() -> ResetPreferences {
    preferences::init(Default::default(), &protocols());
    ResetPreferences
}

fn render(source: &str, width: usize, hide: bool) -> Vec<HyperlinkLine> {
    let _reset = enable_protocols();
    render_markdown_lines_with_width_cwd_and_hidden_link_destinations(
        source,
        Some(width),
        None,
        &|_| hide,
    )
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
    line.push_span(
        "open".into(),
        TerminalHyperlink::markdown(destination, &protocols()).as_ref(),
    );
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
        let opted_in = render(source, 80, false);
        assert_eq!(default, opted_in);
    }
}

#[test]
fn additional_hyperlinks_survive_stream_finalization_and_history_reflow() {
    let _reset = enable_protocols();
    let destination = "vscode://file/tmp/a.rs:12:3?query=one#fragment";
    let source = format!("See [open]({destination}).\n\nDone.\n");
    let mut stream = crate::streaming::controller::StreamController::new(
        Some(30),
        Path::new("/tmp"),
        crate::history_cell::HistoryRenderMode::Rich,
    );
    for delta in source.as_bytes().chunks(7) {
        stream.push(std::str::from_utf8(delta).unwrap());
    }
    let (cell, final_source) = stream.finalize();
    assert_eq!(final_source.as_deref(), Some(source.as_str()));
    assert_destination(&cell.unwrap().transcript_hyperlink_lines(30), destination);
    let cell = crate::history_cell::AgentMarkdownCell::new(source, Path::new("/tmp"));
    for width in [12, 80] {
        assert_destination(&cell.transcript_hyperlink_lines(width), destination);
    }
}

#[test]
fn additional_hyperlinks_keep_validation_and_existing_trusted_capabilities_separate() {
    let destination =
        |value: &str| TerminalHyperlink::markdown(value, &protocols()).map(|link| link.destination);
    for value in ["file:relative", "file:", "https://", "disabled:opaque"] {
        assert_eq!(destination(value), None, "{value}");
    }
    let oversized = format!("custom:{}", "a".repeat(8192));
    assert_eq!(destination(&oversized), None);
    assert_eq!(
        destination("custom:opaque\u{7}value"),
        Some("custom:opaquevalue".into())
    );
    assert_eq!(
        TerminalHyperlink::markdown("file:///tmp/a.rs", &Default::default()),
        None
    );
    let mut line = HyperlinkLine::new(Line::default());
    line.push_span(
        "open".into(),
        TerminalHyperlink::markdown("file:///tmp/a.rs", &protocols()).as_ref(),
    );
    assert_destination(&[line], "file:///tmp/a.rs");
}

#[test]
fn shared_preferences_apply_to_existing_renderers_without_widget_setup() {
    let _reset = enable_protocols();
    let destination = "custom:opaque";
    let source = format!("[open]({destination})");
    let reasoning = crate::history_cell::ReasoningSummaryCell::new(
        String::new(),
        source.clone(),
        Path::new("/tmp"),
        false,
    );
    assert_destination(&reasoning.transcript_hyperlink_lines(80), destination);
    let question = crate::markdown::render_markdown_for_viewport(&source, 80);
    assert_destination(&question, destination);
    let message = crate::history_cell::AgentMarkdownCell::new(source, Path::new("/tmp"));
    assert_destination(&message.transcript_hyperlink_lines(80), destination);
    preferences::init(Default::default(), &Default::default());
    assert!(
        reasoning
            .transcript_hyperlink_lines(80)
            .iter()
            .all(|line| line.hyperlinks.is_empty())
    );
    assert!(
        message
            .transcript_hyperlink_lines(80)
            .iter()
            .all(|line| line.hyperlinks.is_empty())
    );
}

#[test]
fn each_render_keeps_one_settings_snapshot() {
    let _reset = enable_protocols();
    let source = "[one](custom:one) [two](custom:two)";
    let lines = render_markdown_lines_with_width_cwd_and_hidden_link_destinations(
        source,
        Some(80),
        None,
        &|_| {
            preferences::init(Default::default(), &Default::default());
            true
        },
    );
    let destinations = lines
        .iter()
        .flat_map(|line| &line.hyperlinks)
        .map(|link| link.destination.as_str())
        .collect::<Vec<_>>();
    assert_eq!(destinations, ["custom:one", "custom:two"]);
    assert!(
        render_markdown_lines_with_width_and_cwd(source, Some(80), None)
            .iter()
            .all(|line| line.hyperlinks.is_empty())
    );
}
