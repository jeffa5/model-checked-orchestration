use stateright::actor::{ActorModel, Network};

use crate::{
    abstract_model::AbstractModelCfg,
    actor_model::{ActorModelCfg, Actors, ControllerActor, Datastore},
    controller::{Controllers, Node, ReplicaSet, Scheduler},
    state::State,
};

#[derive(Clone, Debug)]
pub struct OrchestrationModelCfg {
    /// The initial state.
    pub initial_state: State,
    /// The number of datastores to run.
    pub datastores: usize,
    /// The number of schedulers to run.
    pub schedulers: usize,
    /// The number of nodes to run.
    pub nodes: usize,
    /// The number of replicaset controllers to run.
    pub replicaset_controllers: usize,
}

impl OrchestrationModelCfg {
    /// Instantiate a new actor model based on this config.
    pub fn into_actor_model(self) -> ActorModel<Actors, ActorModelCfg, ()> {
        let mut model = ActorModel::new(
            ActorModelCfg {
                initial_pods: self.initial_state.pods.len(),
            },
            (),
        );

        assert!(self.datastores > 0);
        for _ in 0..self.datastores {
            model = model.actor(Actors::Datastore(Datastore {
                initial_state: self.initial_state.clone(),
            }));
        }

        for _ in 0..self.nodes {
            model = model.actor(Actors::Node(ControllerActor::new(Node)));
        }

        for _ in 0..self.schedulers {
            model = model.actor(Actors::Scheduler(ControllerActor::new(Scheduler)));
        }

        for _ in 0..self.replicaset_controllers {
            model = model.actor(Actors::ReplicaSet(ControllerActor::new(ReplicaSet)));
        }

        model = model.init_network(Network::new_unordered_nonduplicating(vec![]));

        model.property(
            // TODO: eventually properties don't seem to work with timers, even though they may be
            // steady state.
            stateright::Expectation::Eventually,
            "every application gets scheduled",
            |model, state| {
                let mut any = false;
                let total_apps = model.cfg.initial_pods;
                let datastore_state = state.actor_states.first().unwrap();
                let all_apps_scheduled =
                    datastore_state.pods.values().all(|a| a.node_name.is_some());
                let num_scheduled_apps = datastore_state.pods.len();
                if all_apps_scheduled && num_scheduled_apps == total_apps {
                    any = true;
                }
                any
            },
        )
    }

    pub fn into_abstract_model(self) -> AbstractModelCfg {
        let mut model = AbstractModelCfg {
            controllers: Vec::new(),
            initial_state: self.initial_state,
        };

        assert!(self.datastores > 0);

        for _ in 0..self.nodes {
            model.controllers.push(Controllers::Node(Node));
        }

        for _ in 0..self.schedulers {
            model.controllers.push(Controllers::Scheduler(Scheduler));
        }

        for _ in 0..self.replicaset_controllers {
            model.controllers.push(Controllers::ReplicaSet(ReplicaSet));
        }

        model
    }
}
