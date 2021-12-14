use std::fmt;
use std::hash::{Hash, Hasher};

use npc_engine_turn::{AgentId, Task, StateDiffRef, StateDiffRefMut, Domain};

use crate::{config, Action, Direction, Lumberjacks, State, StateMut, Tile, DIRECTIONS, GlobalStateRef, GlobalStateRefMut};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Barrier {
    pub direction: Direction,
}

impl fmt::Display for Barrier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Barrier({})", self.direction)
    }
}

impl Task<Lumberjacks> for Barrier {
    fn weight(&self, _: StateDiffRef<Lumberjacks>, _: AgentId) -> f32 {
        config().action_weights.barrier
    }

    fn execute(
        &self,
        mut snapshot: StateDiffRefMut<Lumberjacks>,
        agent: AgentId,
    ) -> Option<Box<dyn Task<Lumberjacks>>> {
        // FIXME: cleanup compat code
        let mut state = GlobalStateRefMut::Snapshot(snapshot);
        state.increment_time();

        if let Some((x, y)) = state.find_agent(agent) {
            let (x, y) = self.direction.apply(x, y);
            state.set_tile(x, y, Tile::Barrier);
            state.decrement_inventory(agent);

            None
        } else {
            unreachable!()
        }
    }

    fn display_action(&self) -> <Lumberjacks as Domain>::DisplayAction {
        Action::Barrier(self.direction)
    }

    fn is_valid(&self, snapshot: StateDiffRef<Lumberjacks>, agent: AgentId) -> bool {
        // FIXME: cleanup compat code
        let state = GlobalStateRef::Snapshot(snapshot);
        if let Some((x, y)) = state.find_agent(agent) {
            let (x, y) = self.direction.apply(x, y);
            let empty = matches!(state.get_tile(x, y), Some(Tile::Empty));
            let supported = DIRECTIONS
                .iter()
                .filter(|direction| {
                    let (x, y) = direction.apply(x, y);
                    state
                        .get_tile(x, y)
                        .map(|tile| tile.is_support())
                        .unwrap_or(false)
                })
                .count()
                >= 1;

            empty && supported
        } else {
            unreachable!()
        }
    }

    fn box_clone(&self) -> Box<dyn Task<Lumberjacks>> {
        Box::new(self.clone())
    }

    fn box_hash(&self, mut state: &mut dyn Hasher) {
        self.hash(&mut state)
    }

    fn box_eq(&self, other: &Box<dyn Task<Lumberjacks>>) -> bool {
        if let Some(other) = other.downcast_ref::<Self>() {
            self.eq(other)
        } else {
            false
        }
    }
}
