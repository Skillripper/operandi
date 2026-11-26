//! Where 3D mouse input comes from. The viewport only sees [`Event`]s
//! through a [`Source`], so another way of reading the device (a platform
//! driver, say) can slot in without touching the navigation code.
//!
//! On macOS with 3DxWare installed the driver is the source
//! (`framework.rs`). Otherwise the device is read as raw HID with `hidapi` on a
//! background thread (macOS and Windows). It finds the device, reopens it
//! after an unplug, and wakes the UI on each report. With no device, or on
//! a platform it doesn't cover, it just reports nothing.
use super::decode::DeviceState;

/// What a source tells the UI thread.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    /// A device was opened (its product name).
    Connected(String),
    /// The device went away; axes and buttons are back at rest.
    Disconnected,
    /// The latest axes and buttons.
    State(DeviceState),
    /// The device is there but couldn't be read (shown in Preferences).
    Problem(String),
}

pub trait Source {
    /// The next event, without waiting.
    fn try_next(&mut self) -> Option<Event>;
}

/// The best source there is: on macOS the 3DxWare driver when it's
/// installed (it keeps the device to itself, so raw HID can't open it
/// then), else raw HID; `None` where neither is built in.
pub fn start(ctx: &eframe::egui::Context) -> Option<Box<dyn Source>> {
    // Why 3DxWare is installed but unusable, so raw HID's failure to open
    // the device it holds can say what's really wrong.
    #[cfg(target_os = "macos")]
    let driver_problem = match super::framework::FrameworkSource::start(ctx) {
        Ok(Some(source)) => return Some(Box::new(source)),
        Ok(None) => None,
        Err(problem) => {
            eprintln!("3D mouse: 3DxWare is installed but unusable: {problem}");
            Some(problem)
        }
    };
    #[cfg(target_os = "windows")]
    let driver_problem = None;
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    {
        Some(Box::new(hid::HidSource::start(ctx.clone(), driver_problem)))
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        let _ = ctx;
        None
    }
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
mod hid {
    use super::{Event, Source};
    use crate::space_mouse::decode::{DeviceState, apply_report, is_space_mouse};
    use eframe::egui::Context;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::mpsc::{Receiver, Sender, channel};
    use std::time::Duration;

    /// How often to look for a device while none is open.
    const SCAN: Duration = Duration::from_secs(1);
    /// How long one read waits, so the thread notices it should stop.
    const READ_MS: i32 = 100;

    pub struct HidSource {
        events: Receiver<Event>,
        stop: Arc<AtomicBool>,
    }

    impl HidSource {
        pub fn start(ctx: Context, driver_problem: Option<String>) -> Self {
            let (send, events) = channel();
            let stop = Arc::new(AtomicBool::new(false));
            let flag = stop.clone();
            let spawned = std::thread::Builder::new()
                .name("space-mouse".into())
                .spawn(move || run(&send, &flag, &ctx, driver_problem.as_deref()));
            if let Err(error) = spawned {
                // The receiver stays empty: no device, no harm.
                eprintln!("3D mouse: could not start its reader: {error}");
            }
            Self { events, stop }
        }
    }

    impl Drop for HidSource {
        fn drop(&mut self) {
            // The thread ends within one read; nothing waits for it.
            self.stop.store(true, Ordering::Relaxed);
        }
    }

    impl Source for HidSource {
        fn try_next(&mut self) -> Option<Event> {
            self.events.try_recv().ok()
        }
    }

    /// What Preferences says when the device is there but won't open:
    /// the fix first (the row cuts long text; hovering shows the rest).
    fn open_problem(name: &str, error: &str, driver_problem: Option<&str>) -> String {
        if let Some(driver) = driver_problem {
            format!(
                "Can't reach the 3DxWare driver, which holds the {name}: {driver}. \
                 Reopen Operandi; if it keeps happening, reinstall 3DxWare."
            )
        } else if cfg!(target_os = "macos") {
            format!(
                "macOS won't let Operandi open the {name}. Allow Operandi in System \
                 Settings > Privacy & Security > Input Monitoring, quit other apps \
                 using it, then reopen Operandi. ({error})"
            )
        } else {
            format!("{name} found but could not be opened: {error}")
        }
    }

    fn run(send: &Sender<Event>, stop: &AtomicBool, ctx: &Context, driver_problem: Option<&str>) {
        let tell = |e: Event| {
            let ok = send.send(e).is_ok();
            ctx.request_repaint();
            ok
        };
        let mut api = match hidapi::HidApi::new() {
            Ok(api) => api,
            Err(error) => {
                tell(Event::Problem(format!("HID unavailable: {error}")));
                return;
            }
        };
        // Share the device with the system and any driver that has it.
        #[cfg(target_os = "macos")]
        api.set_open_exclusive(false);
        let mut last_problem = None;
        while !stop.load(Ordering::Relaxed) {
            let _ = api.refresh_devices();
            let found = api.device_list().find(|d| {
                is_space_mouse(
                    d.vendor_id(),
                    d.product_id(),
                    Some((d.usage_page(), d.usage())).filter(|&(page, _)| page != 0),
                )
            });
            let Some(info) = found else {
                std::thread::sleep(SCAN);
                continue;
            };
            let name = info.product_string().unwrap_or("3D mouse").to_owned();
            let device = match info.open_device(&api) {
                Ok(device) => device,
                Err(error) => {
                    let problem = open_problem(&name, &error.to_string(), driver_problem);
                    if last_problem.as_ref() != Some(&problem) {
                        tell(Event::Problem(problem.clone()));
                        last_problem = Some(problem);
                    }
                    std::thread::sleep(SCAN * 2);
                    continue;
                }
            };
            last_problem = None;
            if !tell(Event::Connected(name)) {
                return;
            }
            let mut state = DeviceState::default();
            let mut buf = [0u8; 64];
            while !stop.load(Ordering::Relaxed) {
                match device.read_timeout(&mut buf, READ_MS) {
                    Ok(0) => {}
                    Ok(n) => {
                        let before = state;
                        if apply_report(&mut state, &buf[..n]).is_some()
                            && state != before
                            && !tell(Event::State(state))
                        {
                            return;
                        }
                    }
                    // Unplugged (or the system took it away).
                    Err(_) => break,
                }
            }
            if !tell(Event::Disconnected) {
                return;
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::open_problem;

        #[test]
        fn open_problem_says_what_to_do() {
            let held = open_problem("SpaceNavigator", "hid_open_path failed", Some("it said no"));
            assert!(
                held.contains("3DxWare") && held.contains("it said no"),
                "{held}"
            );
            let plain = open_problem("SpaceNavigator", "hid_open_path failed", None);
            assert!(plain.contains("hid_open_path failed"), "{plain}");
            if cfg!(target_os = "macos") {
                assert!(plain.contains("Input Monitoring"), "{plain}");
            }
        }
    }
}
