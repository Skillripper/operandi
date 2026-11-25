//! UIC-78: Apply leaves its result live. A numeric session's Apply (or a
//! click in a 3D view) records the result as one undo step under a gesture
//! of its own, kept in the session's `applied_step`. While that step is the
//! newest, nothing was undone and the scene still shows it, a changed value
//! remakes the result from the step's `before` and folds into the same
//! step. A new selection, Undo or any other edit settles it; Cancel takes
//! a live result back with its step. Used by Bridge and Spike.
use crate::*;

impl Editor {
    /// A fresh undo gesture for one Apply. The app's own gesture ids count
    /// up from 1, so the top bit keeps these apart.
    pub(crate) fn live_apply_fold(&self) -> u64 {
        (1u64 << 63) | self.history_mark
    }

    /// The newest undo step is `fold`'s and nothing was undone (cheap: no
    /// scene comparison, so a panel can ask every frame).
    pub(crate) fn live_step_newest(&self, fold: Option<u64>) -> bool {
        fold.is_some()
            && self.redo.is_empty()
            && self.undo.last().is_some_and(|last| last.fold == fold)
    }

    /// `live_step_newest`, and the scene still shows that step's result.
    pub(crate) fn live_step_shown(&self, fold: Option<u64>) -> bool {
        self.live_step_newest(fold) && self.undo.last().is_some_and(|l| l.after == self.scene)
    }

    /// Record `before` → the current scene as `label` under `fold`, so a
    /// later record under the same fold replaces this step's result.
    pub(crate) fn record_folded(&mut self, label: &str, before: Scene, fold: u64) {
        let outer = self.undo_fold.replace(fold);
        self.record(label.into(), before);
        self.undo_fold = outer;
    }

    /// Cancel of a live result: back to the step's `before`, and the step
    /// leaves the history.
    pub(crate) fn pop_live_step(&mut self) -> Result<()> {
        if let Some(step) = self.undo.pop() {
            self.scene = step.before.clone();
            if let Some(ids) = &step.identity_before {
                self.identities.current = ids.clone();
                self.identities.observe(&self.scene)?;
            }
            self.history_serial = self.history_serial.wrapping_add(1);
        }
        Ok(())
    }
}
