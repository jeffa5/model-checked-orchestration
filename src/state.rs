use std::collections::{BTreeMap, BTreeSet};

use crate::abstract_model::Change;

#[derive(Default, Clone, Debug, PartialEq, Eq, Hash)]
pub struct State {
    pub nodes: BTreeMap<usize, NodeResource>,
    pub schedulers: BTreeSet<usize>,
    pub replicaset_controllers: BTreeSet<usize>,
    pub pods: BTreeMap<u32, PodResource>,
    pub replica_sets: BTreeMap<u32, ReplicaSetResource>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PodResource {
    pub id: u32,
    pub node_name: Option<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ReplicaSetResource {
    pub id: u32,
    pub replicas: u32,
}

impl ReplicaSetResource {
    pub fn pods(&self) -> Vec<u32> {
        (0..self.replicas).map(|i| (self.id * 1000) + i).collect()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeResource {
    pub running: BTreeSet<u32>,
    pub ready: bool,
}

impl State {
    pub fn apply_change(&mut self, change: Change) {
        match change {
            Change::NodeJoin(i) => {
                self.nodes.insert(
                    i,
                    NodeResource {
                        running: BTreeSet::new(),
                        ready: true,
                    },
                );
            }
            Change::SchedulerJoin(i) => {
                self.schedulers.insert(i);
            }
            Change::ReplicasetJoin(i) => {
                self.replicaset_controllers.insert(i);
            }
            Change::NewPod(i) => {
                self.pods.insert(
                    i,
                    PodResource {
                        id: i,
                        node_name: None,
                    },
                );
            }
            Change::SchedulePod(pod, node) => {
                if let Some(pod) = self.pods.get_mut(&pod) {
                    pod.node_name = Some(node);
                }
            }
            Change::RunPod(pod, node) => {
                self.nodes.get_mut(&node).unwrap().running.insert(pod);
            }
        }
    }
}
