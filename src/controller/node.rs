use crate::abstract_model::Operation;
use crate::controller::Controller;
use crate::resources::ResourceQuantities;
use crate::state::StateView;

#[derive(Clone, Debug)]
pub struct Node {
    pub name: String,
}

pub struct NodeState {
    pub running: Vec<String>,
}

impl Controller for Node {
    type State = NodeState;

    fn step(
        &self,
        id: usize,
        global_state: &StateView,
        local_state: &mut Self::State,
    ) -> Option<Operation> {
        if let Some(node) = global_state.nodes.get(&id) {
            for pod in global_state
                .pods
                .values()
                .filter(|p| p.spec.node_name.as_ref().map_or(false, |n| n == &self.name))
            {
                if !local_state.running.contains(&pod.metadata.name) {
                    return Some(Operation::RunPod(pod.metadata.name.clone(), id));
                }
            }
        } else {
            return Some(Operation::NodeJoin(
                id,
                ResourceQuantities {
                    cpu_cores: Some(4.into()),
                    memory_mb: Some(4000.into()),
                    pods: Some(32.into()),
                },
            ));
        }
        None
    }

    fn name(&self) -> String {
        "Node".to_owned()
    }
}
