# Configuration

For basic configuration instructions, see [this documentation](https://developers.openai.com/codex/config-basic).

For advanced configuration instructions, see [this documentation](https://developers.openai.com/codex/config-advanced).

For a full configuration reference, see [this documentation](https://developers.openai.com/codex/config-reference).

## Lifecycle hooks

Admins can set top-level `allow_managed_hooks_only = true` in
`requirements.toml` to ignore user, project, and session hook configs while
still allowing managed hooks from requirements and managed config layers. This
setting is only supported in `requirements.toml`; putting it in `config.toml`
does not enable managed-hooks-only mode.

## Additional terminal hyperlink protocols

Explicit Markdown links can opt into application or file URL schemes:

```toml
[tui]
additional_hyperlink_protocols = ["vscode", "file"]
```

Set this in user configuration, a selected user profile (`<name>.config.toml`), or
an explicit override such as `-c 'tui.additional_hyperlink_protocols=["vscode","file"]'`.
Higher-priority arrays replace earlier arrays. An absent or empty array retains
built-in HTTP/HTTPS support. Scheme names are case-insensitive, ignore duplicates,
and must match `[A-Za-z][A-Za-z0-9+.-]*`, without a colon.

Project settings, including trusted projects and project profile tables, cannot
change this option. Codex ignores those attempts and reports a startup warning.
Allowed protocols use the same terminal-dependent display as HTTP/HTTPS: labels
alone in recognized terminals, and visible destinations in unknown terminals or
multiplexers. Clicking requires terminal OSC-8 support and a registered handler.

Explicit URLs retain their path, query, fragment, and location suffix for the
handler to interpret. Enabling `file` permits valid absolute file URLs outside
the workspace too. Rendering never opens a destination. Plain paths, file
citations, `file_opener`, and trusted artifact permissions retain their existing
behavior.
