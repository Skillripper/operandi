//! Where the mouse is on screen, asked of the system (macOS) rather than
//! taken from a window's last pointer event.
/// Where the mouse is now, in screen points from the primary screen's
/// top-left (the frame winit and egui give window positions in). Read from
/// the system rather than a window's last pointer event, so it is right
/// even when another of our windows has the keyboard.
#[cfg(target_os = "macos")]
pub fn mouse_location() -> Option<[f32; 2]> {
    use objc2_app_kit::{NSEvent, NSScreen};
    use objc2_foundation::MainThreadMarker;
    let mtm = MainThreadMarker::new()?;
    // AppKit's screen frame has its origin at the primary screen's
    // bottom-left with y up; the primary screen is the first one.
    let primary = NSScreen::screens(mtm).first_retained()?.frame();
    // SAFETY: +[NSEvent mouseLocation] takes no arguments and returns a
    // plain NSPoint; called on the main thread.
    let p = unsafe { NSEvent::mouseLocation() };
    Some([
        p.x as f32,
        (primary.origin.y + primary.size.height - p.y) as f32,
    ])
}
#[cfg(not(target_os = "macos"))]
pub fn mouse_location() -> Option<[f32; 2]> {
    None
}
