//! The animation switch: animation's UI always shows (Jason, 2026-11-24:
//! no Experimental preference any more, so every user, old preferences
//! included, gets it). One function decides it: [`enabled`].
//!
//! Off hides the Animate layout, the timeline strip and its clock button,
//! the graph editor, the Animate menu, the animation keys (their chords
//! keep their usual meaning), the property rows' keying (status-light
//! clicks and key menus), Render Frame Range, Playblast and the USD
//! animation preference. Kept either way: scenes with keys open, play
//! back their pose and save their keys; every command stays registered,
//! so scripts, the MCP bridge and tests still reach animation.
//!
//! Off only for developers: `OPERANDI_ANIMATION=0` in the environment.
//! The old Experimental > Animation preference is still read from saved
//! preferences (so they load) but ignored.
#[cfg(test)]
use std::cell::Cell;

/// The environment variable that turns it off.
#[cfg_attr(test, allow(dead_code))]
pub(crate) const ENV: &str = "OPERANDI_ANIMATION";

#[cfg(test)]
thread_local! {
    /// Tests: on unless a test turns it off (animation's own tests run
    /// with it on; the switch's tests turn it off).
    static TEST: Cell<bool> = const { Cell::new(true) };
}

/// Whether animation's UI shows.
pub(crate) fn enabled() -> bool {
    #[cfg(test)]
    {
        TEST.with(Cell::get)
    }
    #[cfg(not(test))]
    {
        from(std::env::var(ENV).ok().as_deref())
    }
}

/// The rule: on unless the environment turns it off.
pub(crate) fn from(env: Option<&str>) -> bool {
    forced_by(env).unwrap_or(true)
}

/// What the environment variable forces, if it is set to on or off.
pub(crate) fn forced_by(env: Option<&str>) -> Option<bool> {
    match env.map(str::trim) {
        Some("1" | "true" | "on" | "yes") => Some(true),
        Some("0" | "false" | "off" | "no") => Some(false),
        _ => None,
    }
}

/// Tests: turn animation's UI on or off for this thread.
#[cfg(test)]
pub(crate) fn set_for_test(on: bool) {
    TEST.with(|t| t.set(on));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn on_unless_the_environment_turns_it_off() {
        assert!(from(None));
        assert!(from(Some("")));
        assert!(from(Some("1")));
        assert!(!from(Some("0")));
        assert!(!from(Some(" off ")));
    }
}
