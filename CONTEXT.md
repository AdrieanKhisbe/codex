# Codex terminal hyperlinks

This glossary describes the link distinctions used by the Codex terminal UI.

## Language

**Additional hyperlink protocol**:
A URL scheme explicitly enabled by the user for clickable Markdown links in addition to HTTP and HTTPS.
_Avoid_: File opener, trusted artifact type

**Explicit URL**:
A link destination that already names its scheme, such as an editor URL or an absolute file URL.
_Avoid_: Plain path, file citation

**File citation**:
A reference to a local file, optionally with a location, whose opening behavior follows the user's file-opener preference.
_Avoid_: Explicit file URL

**Trusted artifact**:
A generated or validated local artifact with its own permission to be linked by the terminal UI.
_Avoid_: Allowlisted URL

**Project configuration**:
Settings supplied by a repository, including its profile settings, regardless of whether the project is trusted.
_Avoid_: User configuration
