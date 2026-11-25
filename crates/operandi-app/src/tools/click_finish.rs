//! A click applies a tool that waits for Apply (UIC-78, provisional until
//! the maintainer OKs the wording): a plain left click in empty space in a
//! 3D view (off every handle), without a drag, does what Apply does. The tool stays live on
//! the result, so changing a value reshapes it (still one undo step) until
//! a new selection or another edit settles it. Drags still drive the tool
//! (DRAG-Q01, UIC-76). When
//! the tool can't apply, nothing changes and the status line says why.
//! Clicks never select while the tool is on (the maintainer, 2026-11-25):
//! select first, then pick the tool.
use super::*;
use input::NumericDrag;

/// How far (in pixels) a press may wander and still count as a click, as
/// Polygon Reduction's stationary click does.
pub(crate) const CLICK_SLOP: f32 = 3.;

impl Operandi {
    /// The release of a plain left click in a 3D view during a numeric
    /// session's press (`drag` is the gesture the press began).
    pub(crate) fn click_finish_numeric(&mut self, drag: &NumericDrag) {
        if let NumericDrag::Bridge(_) = drag {
            self.click_apply_bridge();
        }
    }

    /// Bridge: Apply with the panel's values; the tool stays on.
    fn click_apply_bridge(&mut self) {
        if self.editor.bridge_result_live() {
            self.status =
                "Bridge is live: change a value to reshape it; Q keeps it and ends the tool".into();
            return;
        }
        // The panel greys Apply for the same reasons (UIC-92).
        if !self.field_errors.is_empty() {
            self.status = format!("Bridge: {}", crate::unavailable::FIELD_ERRORS);
            return;
        }
        if let Err(e) = self.editor.bridge_eligibility() {
            self.status = format!("Bridge: {e}");
            return;
        }
        if self.run_ok(Command::Apply) {
            self.status = "Bridge applied · change a value to reshape it".into();
        }
    }
}
