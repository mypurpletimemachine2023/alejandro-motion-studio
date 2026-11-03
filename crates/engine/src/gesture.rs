//! A bounded, transient edit checkpoint for cancellation of one pointer gesture.
//! It is never serialized and cannot replace a different project opened in the meantime.

use std::sync::Arc;

use crate::{EditorState, History, Session};
use effectcraft_project::Project;

/// Async imports/opens keep their order and stay queued until the gesture's final UI pass.
pub fn drain_pending<T>(pending: &mut Vec<T>, input_owned: bool) -> Vec<T> {
    if input_owned { Vec::new() } else { std::mem::take(pending) }
}

pub struct Checkpoint {
    project: Arc<Project>,
    history: History,
    state: EditorState,
    project_epoch: u64,
    events: usize,
}

impl Checkpoint {
    pub fn capture(session: &Session) -> Self {
        Self {
            project: session.project.clone(),
            history: session.history.clone(),
            state: session.state.clone(),
            project_epoch: session.script_ui.project_epoch,
            events: session.events.len(),
        }
    }

    pub fn cancel(self, session: &mut Session) -> bool {
        if session.script_ui.project_epoch != self.project_epoch {
            return false;
        }
        let changed = !Arc::ptr_eq(&session.project, &self.project);
        let edited = crate::commands::text_edit::active_editor_snapshot(session);
        session.project = self.project;
        session.history = self.history;
        session.state = self.state;
        session.events.truncate(self.events);
        session.sanitize_after_replacement(edited.as_ref());
        // Keep the process IME high-water mark and monotonic document revision. bump() also
        // recognizes the original saved Arc, restoring its clean/dirty state correctly.
        if changed {
            session.bump();
        }
        changed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn cancelled_touch_restores_content_redo_and_dirty_state() {
        let mut s = Session::default();
        s.execute("comp.new", json!({"name":"Touch test", "width":64, "height":64})).expect("valid test composition");
        s.execute("layer.newSolid", json!({"name":"Before"})).expect("valid test solid");
        s.mark_saved();
        s.execute("layer.newSolid", json!({"name":"Redo"})).expect("valid redo solid");
        assert!(s.undo());
        let before = s.project.clone();
        let redo = s.history.redo.len();
        let checkpoint = Checkpoint::capture(&s);
        s.execute("layer.newSolid", json!({"name":"Interrupted touch"})).expect("valid interrupted solid");
        assert!(checkpoint.cancel(&mut s));
        assert!(Arc::ptr_eq(&before, &s.project));
        assert_eq!(s.history.redo.len(), redo);
        assert!(!s.is_dirty());
        assert!(s.redo());
    }

    #[test]
    fn checkpoint_cannot_rebind_a_new_document() {
        let mut s = Session::default();
        let checkpoint = Checkpoint::capture(&s);
        s.replace_project(Project::default(), None);
        let replacement = s.project.clone();
        assert!(!checkpoint.cancel(&mut s));
        assert!(Arc::ptr_eq(&replacement, &s.project));
    }

    #[test]
    fn queued_import_survives_touch_cancel_and_runs_after_the_boundary() {
        let mut s = Session::default();
        s.execute("comp.new", json!({"name":"Queue test", "width":64, "height":64})).expect("valid composition");
        s.mark_saved();
        let checkpoint = Checkpoint::capture(&s);
        s.execute("layer.newSolid", json!({"name":"Uncommitted touch"})).expect("valid gesture edit");
        let mut pending = vec!["Imported A", "Imported B"];
        assert!(drain_pending(&mut pending, true).is_empty());
        assert!(checkpoint.cancel(&mut s));
        // Release/cancel input has arrived, but the final UI pass still owns the boundary.
        assert!(drain_pending(&mut pending, true).is_empty());
        assert_eq!(pending, ["Imported A", "Imported B"]);
        for name in drain_pending(&mut pending, false) {
            s.execute("layer.newSolid", json!({"name":name})).expect("queued import applies");
        }
        let names: Vec<_> = s.project.comps().next().map(|(_, comp)| comp.layers.iter().map(|layer| layer.name.as_str()).collect()).unwrap_or_default();
        assert!(names.contains(&"Imported A") && names.contains(&"Imported B"));
        assert!(!names.contains(&"Uncommitted touch"));
        assert!(pending.is_empty());
    }
}
