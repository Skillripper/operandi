//! Preferences > Defaults > Application > Use OS Color Picker for the
//! shared colour field (colour_field.rs). Off (the default), a click opens
//! Operandi's colour picker. On, it opens the system colour panel (macOS
//! NSColorPanel) and the field follows it while the panel stays open; other
//! platforms keep Operandi's picker.
use eframe::egui::Id;
use std::cell::Cell;

thread_local! {
    static ENABLED: Cell<bool> = const { Cell::new(false) };
    /// The swatch the system panel is open for.
    static OWNER: Cell<Option<Id>> = const { Cell::new(None) };
}

pub fn set_enabled(on: bool) {
    ENABLED.with(|c| c.set(on));
    if !on && OWNER.with(|o| o.take()).is_some() {
        color_panel::close();
    }
}
fn enabled() -> bool {
    ENABLED.with(Cell::get) && cfg!(target_os = "macos")
}

/// The shared colour field's click with the preference on: open the system
/// panel for field `owner` on linear `rgb`. False (nothing opened) when the
/// preference is off, so the field opens its own picker.
pub(crate) fn open_system_for(owner: Id, rgb: [f64; 3]) -> bool {
    if !enabled() {
        return false;
    }
    let srgb = rgb.map(|c| operandi_core::colour_math::linear_to_srgb(c.clamp(0., 1.)) as f32);
    if color_panel::open(srgb) {
        OWNER.with(|o| o.set(Some(owner)));
        true
    } else {
        false
    }
}

/// Whether the system panel is open for `owner`.
pub(crate) fn system_open_for(owner: Id) -> bool {
    OWNER.with(Cell::get) == Some(owner)
}

/// While the system panel is open for `owner`: its colour (linear) when it
/// differs from `rgb`.
pub(crate) fn follow_system(owner: Id, rgb: [f64; 3]) -> Option<[f64; 3]> {
    if OWNER.with(Cell::get) != Some(owner) {
        return None;
    }
    match color_panel::poll() {
        Some(picked) => {
            let linear = picked.map(|c| operandi_core::colour_math::srgb_to_linear(f64::from(c)));
            let now = rgb.map(|c| operandi_core::colour_math::linear_to_srgb(c.clamp(0., 1.)));
            picked
                .iter()
                .zip(now)
                .any(|(&p, n)| (f64::from(p) - n).abs() > 1. / 512.)
                .then_some(linear)
        }
        None => {
            OWNER.with(|o| o.set(None));
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_system_panel_opens_only_with_the_preference() {
        let owner = Id::new("field");
        set_enabled(false);
        assert!(!enabled());
        assert!(!open_system_for(owner, [0.5; 3]));
        assert!(!system_open_for(owner));
        assert_eq!(follow_system(owner, [0.5; 3]), None);
        set_enabled(true);
        assert_eq!(enabled(), cfg!(target_os = "macos"));
        set_enabled(false);
    }
}

/// The system color picker (macOS NSColorPanel), used by Set Material's
/// Color swatch. It is a floating panel, not a modal: the dialog opens it
/// with a starting color and reads the panel's color back every frame while
/// the panel is visible. Colors here are sRGB components, 0 to 1.
#[cfg(target_os = "macos")]
mod color_panel {
    use objc2::rc::Retained;
    use objc2_app_kit::{NSColor, NSColorPanel, NSColorSpace};
    use objc2_foundation::MainThreadMarker;

    fn panel() -> Option<Retained<NSColorPanel>> {
        let mtm = MainThreadMarker::new()?;
        // SAFETY: on the main thread (checked above); the shared panel is
        // owned by AppKit and lives for the process.
        Some(unsafe { NSColorPanel::sharedColorPanel(mtm) })
    }
    fn ns_color(rgb: [f32; 3]) -> Retained<NSColor> {
        let [r, g, b] = rgb.map(|c| c.clamp(0.0, 1.0) as f64);
        // SAFETY: a plain class constructor taking finite components.
        unsafe { NSColor::colorWithSRGBRed_green_blue_alpha(r, g, b, 1.0) }
    }
    /// Shows the panel with `rgb`. False when there is no main thread.
    pub fn open(rgb: [f32; 3]) -> bool {
        let Some(panel) = panel() else {
            return false;
        };
        // SAFETY: main-thread AppKit calls on the live shared panel with
        // an owned NSColor.
        unsafe {
            panel.setShowsAlpha(false);
            panel.setContinuous(true);
            panel.setColor(&ns_color(rgb));
        }
        panel.orderFront(None);
        true
    }
    /// Moves the panel to `rgb` (after the fields changed) without showing it.
    /// Unused since Set Material moved to the shared form (its colour field
    /// drives the panel itself); kept for the next caller.
    #[allow(dead_code)]
    pub fn set(rgb: [f32; 3]) {
        if let Some(panel) = panel() {
            // SAFETY: as in `open`.
            unsafe { panel.setColor(&ns_color(rgb)) };
        }
    }
    /// The panel's color while it is on screen; `None` once it's closed.
    pub fn poll() -> Option<[f32; 3]> {
        let panel = panel()?;
        if !panel.isVisible() {
            return None;
        }
        // SAFETY: main-thread reads from the live shared panel. The color is
        // converted to sRGB first, since component accessors are only valid
        // for RGB colors; a color that can't convert reads as None.
        unsafe {
            let color = panel.color();
            let srgb = color.colorUsingColorSpace(&NSColorSpace::sRGBColorSpace())?;
            Some(
                [
                    srgb.redComponent(),
                    srgb.greenComponent(),
                    srgb.blueComponent(),
                ]
                .map(|c| c as f32),
            )
        }
    }
    pub fn close() {
        if let Some(panel) = panel()
            && panel.isVisible()
        {
            panel.orderOut(None);
        }
    }
}
/// Other platforms have no native picker hooked up yet; the dialog falls
/// back to egui's own color picker.
#[cfg(not(target_os = "macos"))]
mod color_panel {
    pub fn open(_: [f32; 3]) -> bool {
        false
    }
    #[allow(dead_code)]
    pub fn set(_: [f32; 3]) {}
    pub fn poll() -> Option<[f32; 3]> {
        None
    }
    pub fn close() {}
}
