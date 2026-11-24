//! The animation switch: animation's UI shows by default (Jason,
//! 2026-11-23; hidden until then since 2026-11-22), and the preference
//! turns it off. One function decides it: [`enabled`].
//!
//! Off hides the Animate layout, the timeline strip and its clock button,
//! the graph editor, the Animate menu, the animation keys (their chords
//! keep their usual meaning), the property rows' keying (status-light
//! clicks and key menus), Render Frame Range, Playblast and the USD
//! animation preference. Kept either way: scenes with keys open, play
//! back their pose and save their keys; every command stays registered,
//! so scripts, the MCP bridge and tests still reach animation.
//!
//! Off: Preferences > Defaults > Application > Experimental > Animation,
//! or `OPERANDI_ANIMATION=0` in the environment (it wins over the
//! preference; `OPERANDI_ANIMATION=1` forces it on).
use std::cell::Cell;

/// Whether animation shows when neither the preference nor the
/// environment says otherwise.
pub(crate) const DEFAULT_ON: bool = true;

/// The environment variable that overrides everything.
#[cfg_attr(test, allow(dead_code))]
pub(crate) const ENV: &str = "OPERANDI_ANIMATION";

thread_local! {
    /// The preference as loaded (`None` until the app loads it).
    static PREFERENCE: Cell<Option<bool>> = const { Cell::new(None) };
    /// Tests: on unless a test turns it off (animation's own tests run
    /// with it on; the switch's tests turn it off).
    #[cfg(test)]
    static TEST: Cell<bool> = const { Cell::new(true) };
    /// Tests: what the environment forces (none unless a test says).
    #[cfg(test)]
    static FORCED: Cell<Option<bool>> = const { Cell::new(None) };
}

/// Whether animation's UI shows.
pub(crate) fn enabled() -> bool {
    #[cfg(test)]
    {
        TEST.with(Cell::get)
    }
    #[cfg(not(test))]
    {
        from(
            std::env::var(ENV).ok().as_deref(),
            PREFERENCE.with(Cell::get),
        )
    }
}

/// The rule: the environment wins, then the preference, then the default.
pub(crate) fn from(env: Option<&str>, preference: Option<bool>) -> bool {
    forced_by(env).unwrap_or(preference.unwrap_or(DEFAULT_ON))
}

/// What the environment variable forces, if it is set to on or off.
pub(crate) fn forced_by(env: Option<&str>) -> Option<bool> {
    match env.map(str::trim) {
        Some("1" | "true" | "on" | "yes") => Some(true),
        Some("0" | "false" | "off" | "no") => Some(false),
        _ => None,
    }
}

/// What the environment forces right now: the preference row then shows
/// that state, greyed, and says why.
pub(crate) fn forced() -> Option<bool> {
    #[cfg(test)]
    {
        FORCED.with(Cell::get)
    }
    #[cfg(not(test))]
    {
        forced_by(std::env::var(ENV).ok().as_deref())
    }
}

/// The preference changed (or was loaded).
pub(crate) fn set_preference(on: bool) {
    PREFERENCE.with(|p| p.set(Some(on)));
}

/// Tests: turn animation's UI on or off for this thread.
#[cfg(test)]
pub(crate) fn set_for_test(on: bool) {
    TEST.with(|t| t.set(on));
}

/// Tests: pretend the environment forces the switch (or not).
#[cfg(test)]
pub(crate) fn force_for_test(on: Option<bool>) {
    FORCED.with(|f| f.set(on));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_environment_wins_then_the_preference_then_the_default() {
        assert!(from(Some("1"), Some(false)));
        assert!(!from(Some("0"), Some(true)));
        assert!(from(None, Some(true)));
        assert!(!from(None, Some(false)));
        assert_eq!(from(None, None), DEFAULT_ON);
        assert_eq!(from(Some(""), None), DEFAULT_ON);
    }
}
