use stateright::actor::Actor;
use stateright::actor::Id;
use stateright::actor::Out;
use std::borrow::Cow;
use std::fmt::Debug;
use std::hash::Hash;

use crate::datastore;
use crate::node;
use crate::scheduler;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Root {
    Scheduler(scheduler::Scheduler),
    Node(node::Node),
    Datastore(datastore::Datastore),
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum RootState {
    Scheduler(<scheduler::Scheduler as Actor>::State),
    Node(<node::Node as Actor>::State),
    Datastore(<datastore::Datastore as Actor>::State),
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum RootMsg {
    /// A message specific to the register system's internal protocol.
    Scheduler(scheduler::SchedulerMsg),
    Node(node::NodeMsg),

    Datastore(datastore::DatastoreMsg),
}

impl Actor for Root {
    type Msg = RootMsg;

    type State = RootState;

    type Timer = ();

    fn on_start(&self, id: Id, o: &mut Out<Self>) -> Self::State {
        match self {
            Root::Scheduler(client_actor) => {
                let mut client_out = Out::new();
                let state = RootState::Scheduler(client_actor.on_start(id, &mut client_out));
                o.append(&mut client_out);
                state
            }
            Root::Node(client_actor) => {
                let mut client_out = Out::new();
                let state = RootState::Node(client_actor.on_start(id, &mut client_out));
                o.append(&mut client_out);
                state
            }
            Root::Datastore(client_actor) => {
                let mut client_out = Out::new();
                let state = RootState::Datastore(client_actor.on_start(id, &mut client_out));
                o.append(&mut client_out);
                state
            }
        }
    }

    fn on_msg(
        &self,
        id: Id,
        state: &mut Cow<Self::State>,
        src: Id,
        msg: Self::Msg,
        o: &mut Out<Self>,
    ) {
        use Root as A;
        use RootState as S;

        match (self, &**state) {
            (A::Scheduler(client_actor), S::Scheduler(client_state)) => {
                let mut client_state = Cow::Borrowed(client_state);
                let mut client_out = Out::new();
                client_actor.on_msg(id, &mut client_state, src, msg, &mut client_out);
                if let Cow::Owned(client_state) = client_state {
                    *state = Cow::Owned(RootState::Scheduler(client_state))
                }
                o.append(&mut client_out);
            }
            _ => todo!(),
        }
    }

    fn on_timeout(
        &self,
        _id: Id,
        state: &mut Cow<Self::State>,
        _timer: &Self::Timer,
        _o: &mut Out<Self>,
    ) {
        use Root as A;
        use RootState as S;
        match (self, &**state) {
            (A::Scheduler(_), S::Scheduler(_)) => {}
            _ => todo!(),
        }
    }
}
