//! The Help menu (Jason 2026-11-16): User Guide (Documentation until
//! 2026-11-23, greyed until the guide had a home), and Report Bug or Idea…
//! (Report Bug until 2026-11-20; its "…" per UIC-03, as the browser form
//! asks for more before anything is sent), moved off the layout bar. Both
//! run registry commands (UIC-220): `operandi.help.documentation` and
//! `operandi.help.reportBug` (report_bug_dialog.rs: a dialog, then the
//! pre-filled form).
use super::*;
use crate::update_check::Channel;

/// Where the User Guide lives on the website. Each release has its own
/// folder (`guide/<version>/`), test-channel builds share `guide/test/`,
/// built from main (docs/guide, published by .github/workflows/guide.yml).
pub const GUIDE_SITE: &str = "https://operandi3d.org/guide/";

/// Help > User Guide's address for a build of `version` on `channel`.
/// The channel is the one the build was made for (test builds are made
/// with OPERANDI_DEFAULT_UPDATE_CHANNEL=test), not the Update Channel
/// preference: a release build's guide matches its own version.
pub fn guide_url_for(version: &str, channel: Channel) -> String {
    match channel {
        Channel::Test => format!("{GUIDE_SITE}test/"),
        Channel::Release => format!("{GUIDE_SITE}{version}/"),
    }
}

/// Help > User Guide's address for this build.
pub fn guide_url() -> String {
    guide_url_for(env!("CARGO_PKG_VERSION"), Channel::build_default())
}

impl Operandi {
    /// `operandi.help.documentation`: the User Guide in the browser.
    pub(super) fn open_documentation(&mut self) {
        let url = guide_url();
        self.status = match self.open_in_browser(&url) {
            Ok(()) => "Opened the User Guide in your browser".into(),
            Err(e) => {
                self.copy_next = Some(url);
                format!("Couldn't open the browser ({e}); the link is on the clipboard")
            }
        };
    }

    /// Hands `url` to the default browser. Tests record it instead.
    pub(super) fn open_in_browser(&mut self, url: &str) -> std::io::Result<()> {
        self.opened_url = Some(url.to_owned());
        if cfg!(test) {
            return Ok(());
        }
        bug_report::open_url(url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_menu::Entry;

    #[test]
    fn help_menu_is_last_and_runs_report_bug_and_documentation() {
        let mut a = Operandi::empty();
        let ctx = Context::default();
        let model = a.menu_model(&ctx);
        let help = model.last().unwrap();
        assert_eq!(help.title, "Help");
        let leaves = help.leaves();
        let bug = leaves
            .iter()
            .find(|i| i.action == crate::app_menu::line_action("operandi.help.reportBug"))
            .unwrap();
        assert!(bug.enabled);
        let docs = leaves
            .iter()
            .find(|i| i.action == crate::app_menu::line_action("operandi.help.documentation"))
            .unwrap();
        assert_eq!(docs.title, "User Guide");
        assert!(docs.enabled);
        assert!(help.entries.iter().all(|e| !matches!(e, Entry::Submenu(_))));

        // The menu runs the command; the command opens the issue link.
        a.menu_action(
            &ctx,
            crate::app_menu::line_action("operandi.help.reportBug"),
        );
        // The menu runs the bare command: the capture, then the dialog,
        // and nothing opens before OK (BUG-01).
        assert!(a.report_bug.is_modal(), "{}", a.status);
        assert!(a.opened_url.is_none());
        a.report_bug = Default::default();
        a.execute(
            "operandi.help.reportBug screenshot:false toolState:false commands:false log:false",
        );
        let url = a.opened_url.take().expect("the full line opens the link");
        assert!(url.contains("issues/new"), "{url}");

        // User Guide opens this build's guide.
        a.menu_action(
            &ctx,
            crate::app_menu::line_action("operandi.help.documentation"),
        );
        assert_eq!(a.opened_url.take(), Some(guide_url()));
        a.execute("operandi.help.documentation");
        assert_eq!(a.opened_url.take(), Some(guide_url()));
    }

    #[test]
    fn user_guide_opens_the_version_folder_or_the_test_guide() {
        assert_eq!(
            guide_url_for("0.2.0", Channel::Release),
            "https://operandi3d.org/guide/0.2.0/"
        );
        assert_eq!(
            guide_url_for("0.2.0", Channel::Test),
            "https://operandi3d.org/guide/test/"
        );
        assert!(guide_url().starts_with(GUIDE_SITE) && guide_url().ends_with('/'));
    }
}
