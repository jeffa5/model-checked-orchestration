use stateright::{Model, Property};

use crate::controller::{Controller, Controllers};
use crate::state::{ConsistencyLevel, State, StateView, Revision};

#[derive(Debug)]
pub struct AbstractModelCfg {
    /// The controllers running in this configuration.
    pub controllers: Vec<Controllers>,
    /// The initial state.
    pub initial_state: StateView,
    /// The consistency level of the state.
    pub consistency_level: ConsistencyLevel,
}

/// Changes to a state.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Change {
    /// The revision of the state that this change was generated from.
    pub revision: Revision,
    /// The operation to perform on the state.
    pub operation: Operation,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Operation {
    NodeJoin(usize),
    SchedulerJoin(usize),
    ReplicasetJoin(usize),
    NewPod(u32),
    SchedulePod(u32, usize),
    RunPod(u32, usize),
    NodeCrash(usize),
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Action {
    ControllerStep(usize, String, Vec<Change>),
    NodeCrash(usize),
}

impl Model for AbstractModelCfg {
    type State = State;

    type Action = Action;

    fn init_states(&self) -> Vec<Self::State> {
        vec![State::default()
            .with_initial(self.initial_state.clone())
            .with_consistency_level(self.consistency_level.clone())]
    }

    fn actions(&self, state: &Self::State, actions: &mut Vec<Self::Action>) {
        let views = state.views();
        for view in views {
            for (i, controller) in self.controllers.iter().enumerate() {
                let changes = controller.step(i, &view);
                actions.push(Action::ControllerStep(i, controller.name(), changes));
            }
            for (node_id, node) in &view.nodes {
                if node.ready {
                    actions.push(Action::NodeCrash(*node_id));
                }
            }
        }
    }

    fn next_state(&self, last_state: &Self::State, action: Self::Action) -> Option<Self::State> {
        match action {
            Action::ControllerStep(_, _, changes) => {
                let mut state = last_state.clone();
                state.push_changes(changes.into_iter());
                Some(state)
            }
            Action::NodeCrash(node) => {
                let mut state = last_state.clone();
                state.push_change(Change {
                    revision: last_state.max_revision(),
                    operation: Operation::NodeCrash(node),
                });
                Some(state)
            }
        }
    }

    fn properties(&self) -> Vec<stateright::Property<Self>> {
        vec![Property::<Self>::eventually(
            "every pod gets scheduled",
            |_model, state| {
                let state = state.view_at(state.max_revision());
                state.pods.values().all(|pod| pod.node_name.is_some())
            },
        )]
    }
}
