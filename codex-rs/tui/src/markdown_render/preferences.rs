//! Content preferences shared by streaming and completed Markdown renderers.
//!
//! Seed before startup previews, then refresh when resolved session settings become active.
//! Tests isolate preferences per thread so parallel renderer tests cannot affect one another.

use codex_config::AdditionalHyperlinkProtocols;
use codex_config::types::TuiRendering;

#[derive(Default)]
struct Preferences {
    rendering: TuiRendering,
    hyperlink_protocols: AdditionalHyperlinkProtocols,
}

#[cfg(not(test))]
static RENDERING: std::sync::LazyLock<std::sync::RwLock<Preferences>> =
    std::sync::LazyLock::new(|| std::sync::RwLock::new(Preferences::default()));

#[cfg(test)]
thread_local! {
    static RENDERING: std::cell::RefCell<Preferences> = std::cell::RefCell::new(Preferences::default());
}

pub(crate) fn init(rendering: TuiRendering, hyperlink_protocols: &AdditionalHyperlinkProtocols) {
    let preferences = Preferences {
        rendering,
        hyperlink_protocols: hyperlink_protocols.clone(),
    };
    #[cfg(not(test))]
    {
        *RENDERING
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = preferences;
    }
    #[cfg(test)]
    RENDERING.set(preferences);
}

pub(crate) fn current() -> TuiRendering {
    #[cfg(not(test))]
    {
        RENDERING
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .rendering
    }
    #[cfg(test)]
    {
        RENDERING.with_borrow(|preferences| preferences.rendering)
    }
}

pub(crate) fn hyperlink_protocols() -> AdditionalHyperlinkProtocols {
    #[cfg(not(test))]
    {
        RENDERING
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .hyperlink_protocols
            .clone()
    }
    #[cfg(test)]
    {
        RENDERING.with_borrow(|preferences| preferences.hyperlink_protocols.clone())
    }
}

#[cfg(test)]
#[path = "preferences_tests.rs"]
mod tests;
