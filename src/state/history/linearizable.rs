use std::sync::Arc;

use crate::{
    abstract_model::Change,
    state::{revision::Revision, RawState, StateView},
};

use super::History;

#[derive(Default, Clone, PartialEq, Eq, Hash, Debug)]
pub struct LinearizableHistory {
    state: Arc<StateView>,
}

impl LinearizableHistory {
    pub fn new(initial_state: RawState) -> Self {
        Self {
            state: Arc::new(initial_state.into()),
        }
    }
}

impl History for LinearizableHistory {
    fn add_change(&mut self, change: Change) -> Revision {
        let new_revision = self.max_revision().increment();
        Arc::make_mut(&mut self.state).apply_operation(change.operation, new_revision);
        self.max_revision()
    }

    fn max_revision(&self) -> Revision {
        self.state.revision.clone()
    }

    fn state_at(&self, revision: Revision) -> StateView {
        assert_eq!(revision, self.state.revision);
        (*self.state).clone()
    }

    fn valid_revisions(&self, _min_revision: Revision) -> Vec<Revision> {
        vec![self.state.revision.clone()]
    }
}