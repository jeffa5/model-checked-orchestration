use crate::abstract_model::Operation;
use crate::state::StateView;
use crate::{abstract_model::Change, controller::Controller};

#[derive(Clone, Debug)]
pub struct Scheduler;

impl Controller for Scheduler {
    fn step(&self, id: usize, state: &StateView) -> Vec<Change> {
        let mut actions = Vec::new();
        if !state.schedulers.contains(&id) {
            actions.push(Change {
                revision: state.revision,
                operation: Operation::SchedulerJoin(id),
            })
        } else {
            for pod in state.pods.values() {
                let least_loaded_node = state
                    .nodes
                    .iter()
                    .map(|(n, node)| (n, node.running.len()))
                    .min_by_key(|(_, pods)| *pods);
                if let Some((node, _)) = least_loaded_node {
                    if pod.node_name.is_none() {
                        actions.push(Change {
                            revision: state.revision,
                            operation: Operation::SchedulePod(pod.id, *node),
                        });
                    }
                }
            }
        }
        actions
    }

    fn name(&self) -> String {
        "Scheduler".to_owned()
    }
}
