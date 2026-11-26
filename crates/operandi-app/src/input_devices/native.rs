#[derive(Clone, Copy, Debug)]
pub struct NumberKey {
    pub digit: u8,
    pub keypad: bool,
    pub pressed: bool,
}
/// kVK_ANSI_KeypadEnter, from the macOS SDK's Events.h.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
const KEYPAD_ENTER: u16 = 0x4c;
/// Public AppKit virtual key codes from the installed macOS SDK Events.h.
/// Used by the macOS event monitor; elsewhere only its tests call it.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
fn number_key(code: u16, pressed: bool) -> Option<NumberKey> {
    if code == 0x2f || code == 0x41 {
        return Some(NumberKey {
            digit: 10,
            keypad: code == 0x41,
            pressed,
        });
    }
    let main = [0x1d, 0x12, 0x13, 0x14, 0x15, 0x17, 0x16, 0x1a, 0x1c, 0x19];
    let keypad = [0x52, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5b, 0x5c];
    main.iter()
        .position(|&key| key == code)
        .map(|digit| NumberKey {
            digit: digit as u8,
            keypad: false,
            pressed,
        })
        .or_else(|| {
            keypad
                .iter()
                .position(|&key| key == code)
                .map(|digit| NumberKey {
                    digit: digit as u8,
                    keypad: true,
                    pressed,
                })
        })
}
// Graceful native macOS termination, including menu, Command-Q and Dock Quit.
// The native callback only posts intent; document decisions remain in the app.
#[cfg(target_os = "macos")]
mod mac {
    use objc2::{
        runtime::{AnyObject, Imp, Sel},
        sel,
    };
    use objc2_app_kit::{NSApplication, NSEvent, NSEventType};
    use objc2_foundation::MainThreadMarker;
    use std::sync::{
        OnceLock,
        atomic::{AtomicBool, Ordering},
    };
    type SendEvent = unsafe extern "C" fn(&AnyObject, Sel, &NSEvent);
    static ORIGINAL_EVENT: OnceLock<SendEvent> = OnceLock::new();
    static NUMBERS: std::sync::Mutex<Vec<super::NumberKey>> = std::sync::Mutex::new(Vec::new());
    static NUMBER_OVERFLOW: AtomicBool = AtomicBool::new(false);
    /// Keypad Enter presses since the last frame (Numeric Enter as Tab).
    static KEYPAD_ENTERS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    extern "C" fn observe_event(app: &AnyObject, selector: Sel, event: &NSEvent) {
        // SAFETY: AppKit calls sendEvent: on its main thread with a live NSEvent.
        // Only documented type/keyCode accessors are used; no native borrow is
        // retained, no event is modified/swallowed, and the original ABI-matched
        // implementation is always forwarded exactly once. Mutex failure and
        // bounded queue overflow disable physical-key dispatch for this frame.
        unsafe {
            let kind = event.r#type();
            if kind == NSEventType::KeyDown
                && !event.isARepeat()
                && event.keyCode() == super::KEYPAD_ENTER
            {
                KEYPAD_ENTERS.fetch_add(1, Ordering::AcqRel);
            }
            if (kind == NSEventType::KeyDown || kind == NSEventType::KeyUp)
                && (kind == NSEventType::KeyUp || !event.isARepeat())
                && let Some(key) = super::number_key(event.keyCode(), kind == NSEventType::KeyDown)
                && let Ok(mut keys) = NUMBERS.lock()
            {
                if keys.len() < 256 {
                    keys.push(key);
                } else {
                    keys.clear();
                    NUMBER_OVERFLOW.store(true, Ordering::Release);
                }
            }
            record_tablet(event, kind);
            if let Some(original) = ORIGINAL_EVENT.get() {
                original(app, selector, event);
            }
        }
    }

    /// Copies pen values out of a mouse or tablet event for `crate::tablet`
    /// (pen tablet plan, stage 1). Read-only: the event goes on unchanged.
    ///
    /// AppKit raises an exception when a pen accessor is called on an event
    /// that doesn't carry it, so each is read only where the documentation
    /// says it's valid: `subtype` on mouse events; `pressure`, `tilt` and
    /// `rotation` on tablet points (a bare TabletPoint event, or a mouse
    /// event whose subtype is TabletPoint); `isEnteringProximity` and
    /// `pointingDeviceType` on proximity events (likewise).
    ///
    /// # Safety
    /// Main thread, inside sendEvent:, with a live `event` of type `kind`.
    unsafe fn record_tablet(event: &NSEvent, kind: NSEventType) {
        use crate::tablet::{PenTool, Phase, Raw, record};
        use objc2_app_kit::{NSEventSubtype, NSPointingDeviceType};
        let mouse = [
            NSEventType::LeftMouseDown,
            NSEventType::LeftMouseUp,
            NSEventType::RightMouseDown,
            NSEventType::RightMouseUp,
            NSEventType::OtherMouseDown,
            NSEventType::OtherMouseUp,
            NSEventType::MouseMoved,
            NSEventType::LeftMouseDragged,
            NSEventType::RightMouseDragged,
            NSEventType::OtherMouseDragged,
        ]
        .contains(&kind);
        let point = kind == NSEventType::TabletPoint;
        let proximity = kind == NSEventType::TabletProximity;
        if !(mouse || point || proximity) {
            return;
        }
        // SAFETY (all accessors below): the type checks above and the
        // subtype checks below gate each accessor to events that carry it.
        unsafe {
            let subtype = if mouse {
                event.subtype()
            } else {
                NSEventSubtype(0)
            };
            if proximity || subtype == NSEventSubtype::TabletProximity {
                let tool = match event.pointingDeviceType() {
                    NSPointingDeviceType::Eraser => PenTool::Eraser,
                    NSPointingDeviceType::Cursor => PenTool::Mouse,
                    _ => PenTool::Tip,
                };
                record(Raw::Proximity {
                    entering: event.isEnteringProximity(),
                    tool,
                });
                return;
            }
            let timestamp = event.timestamp();
            let pos = MainThreadMarker::new()
                .and_then(|mtm| event.window(mtm))
                .map(|window| {
                    // Window base coordinates (origin bottom-left) to the
                    // content's points from its top-left, egui's frame.
                    let content = window.contentRectForFrameRect(window.frame());
                    let at = event.locationInWindow();
                    [at.x as f32, (content.size.height - at.y) as f32]
                });
            let phase = match kind {
                NSEventType::LeftMouseDown
                | NSEventType::RightMouseDown
                | NSEventType::OtherMouseDown => Phase::Down,
                NSEventType::LeftMouseUp
                | NSEventType::RightMouseUp
                | NSEventType::OtherMouseUp => Phase::Up,
                NSEventType::LeftMouseDragged
                | NSEventType::RightMouseDragged
                | NSEventType::OtherMouseDragged => Phase::Drag,
                // Mouse moves, and bare tablet points (winit makes no
                // pointer event of those).
                _ => Phase::Hover,
            };
            if point || subtype == NSEventSubtype::TabletPoint {
                let tilt = event.tilt();
                record(Raw::Point {
                    pressure: event.pressure(),
                    tilt: [tilt.x as f32, tilt.y as f32],
                    rotation: event.rotation(),
                    phase,
                    timestamp,
                    pos,
                });
            } else {
                record(Raw::Mouse {
                    phase,
                    timestamp,
                    pos,
                });
            }
        }
    }
    pub fn take_keypad_enters() -> usize {
        KEYPAD_ENTERS.swap(0, Ordering::AcqRel)
    }
    pub fn take_numbers() -> Vec<super::NumberKey> {
        let numbers = NUMBERS
            .lock()
            .map(|mut keys| std::mem::take(&mut *keys))
            .unwrap_or_default();
        if NUMBER_OVERFLOW.swap(false, Ordering::AcqRel) {
            Vec::new()
        } else {
            numbers
        }
    }
    static QUIT: AtomicBool = AtomicBool::new(false);
    static CONTEXT: OnceLock<eframe::egui::Context> = OnceLock::new();
    extern "C" fn request_quit(_: &AnyObject, _: Sel, _: *mut AnyObject) {
        QUIT.store(true, Ordering::Release);
        if let Some(ctx) = CONTEXT.get() {
            ctx.request_repaint();
        }
    }
    pub fn install(ctx: eframe::egui::Context) {
        if CONTEXT.set(ctx).is_err() {
            return;
        }
        let Some(mtm) = MainThreadMarker::new() else {
            return;
        };
        // Extra windows (Render, floating panels) stay separate windows:
        // never let macOS merge them into the main window as tabs.
        // SAFETY: main thread; +[NSWindow setAllowsAutomaticWindowTabbing:]
        // takes one BOOL and returns void.
        unsafe {
            let _: () = objc2::msg_send![
                objc2::class!(NSWindow),
                setAllowsAutomaticWindowTabbing: objc2::runtime::Bool::NO
            ];
        }
        let app = NSApplication::sharedApplication(mtm);
        if let Some(method) = app.class().instance_method(sel!(sendEvent:)) {
            // SAFETY: one installation, main thread, ABI void(id,SEL,NSEvent*).
            // Store the existing process implementation before replacing it;
            // both function pointers and class live until process termination.
            unsafe {
                let original = std::mem::transmute::<Imp, SendEvent>(method.implementation());
                if ORIGINAL_EVENT.set(original).is_ok() {
                    method.set_implementation(std::mem::transmute::<
                        extern "C" fn(&AnyObject, Sel, &NSEvent),
                        Imp,
                    >(observe_event));
                }
            }
        }
        let Some(method) = app.class().instance_method(sel!(terminate:)) else {
            return;
        };
        // SAFETY: called once on AppKit's main thread after NSApplication exists.
        // terminate: has ABI void(id, SEL, id). The process-lifetime callback uses
        // no borrowed native state and never unwinds. All termination requests
        // are deferred to eframe's normal root-window Close after protection.
        unsafe {
            let callback = std::mem::transmute::<extern "C" fn(&AnyObject, Sel, *mut AnyObject), Imp>(
                request_quit,
            );
            method.set_implementation(callback);
        }
    }
    pub fn take_quit() -> bool {
        QUIT.swap(false, Ordering::AcqRel)
    }
}
#[cfg(target_os = "macos")]
pub use mac::{install, take_numbers, take_quit};
#[cfg(not(target_os = "macos"))]
pub fn install(_: eframe::egui::Context) {}
#[cfg(not(target_os = "macos"))]
pub fn take_quit() -> bool {
    false
}

/// Linux and Windows: our patched egui-winit (vendor/egui-winit) reads each
/// digit's scan code, so the keypad and the main row stay distinct.
#[cfg(not(target_os = "macos"))]
pub fn take_numbers() -> Vec<NumberKey> {
    egui_winit::operandi_keys::take_numbers()
        .into_iter()
        .map(|n| NumberKey {
            digit: n.digit,
            keypad: n.keypad,
            pressed: n.pressed,
        })
        .collect()
}
#[cfg(not(target_os = "macos"))]
pub fn take_keypad_enters() -> usize {
    egui_winit::operandi_keys::take_keypad_enters()
}
#[cfg(target_os = "macos")]
pub use mac::take_keypad_enters;

#[cfg(test)]
mod tests {
    #[test]
    fn appkit_main_and_keypad_digits_are_distinct() {
        for digit in 0..10 {
            let main = [0x1d, 0x12, 0x13, 0x14, 0x15, 0x17, 0x16, 0x1a, 0x1c, 0x19][digit];
            let pad = [0x52, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5b, 0x5c][digit];
            let a = super::number_key(main, true).unwrap();
            let b = super::number_key(pad, true).unwrap();
            assert_eq!(a.digit, digit as u8);
            assert_eq!(b.digit, a.digit);
            assert!(!a.keypad && b.keypad);
        }
        assert!(super::number_key(0xffff, true).is_none());
    }
}
