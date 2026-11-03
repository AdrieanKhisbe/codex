# Keep additional hyperlink protocols under user control

Additional hyperlink protocols can invoke registered application handlers and expose explicit file URLs outside the workspace when the user clicks them. Resolve protocol opt-ins from user configuration, selected user profiles, and explicit command-line overrides, preserving their normal precedence. Ignore project attempts to change this setting and report them through startup warnings, even for trusted projects, so opening a repository cannot change which application handlers its links can invoke.

Keep this opt-in separate from file-citation opening and trusted-artifact permissions: an enabled scheme authorizes an explicit URL label, and grants no artifact capabilities. Preserve destinations for the registered handler to interpret; rendering does not open them.

Use the existing terminal-dependent destination display for every allowed protocol. Recognized terminals show labels alone; unknown terminals and multiplexers retain visible destinations. This preserves default HTTP(S) rendering and gives opted-in schemes the same fallback.

Keep the resolved allowlist in the TUI's existing shared rendering preferences. It is a client preference, so conversations displayed by one terminal client share it; separate CLI processes remain independent. The common Markdown writer takes a snapshot for each render and classifies each link once. This follows the existing rendering lifecycle and avoids forwarding protocol settings through individual widgets. Cached Markdown is invalidated when the shared preferences change.
