use bevy::ecs::schedule::StateData;
use bevy::prelude::*;
use iyes_loopless::prelude::*;
use iyes_loopless::condition::ConditionalSystemDescriptor;

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub enum AppState {
    Menu,
    Loading,
    PreGame,
    Game,
    GameFailed,
    GameSuccess,
}

pub trait ConditionalHelpersExt: ConditionHelpers {
    fn run_in_states<State, const N: usize>(self, states: [State; N]) -> Self
    where
        State: StateData,
    {
        self.run_if(move |current_state: Res<CurrentState<State>>| {
            states.contains(&current_state.0)
        })
    }
}

impl ConditionalHelpersExt for ConditionalSystemDescriptor {}
