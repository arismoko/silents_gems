//! The lamp rule's game glue.
//!
//! Upstream's `GemLampBlock` switches its own block to the one its state table names, on place
//! and on a neighbour change (`block/GemLampBlock.java`, `checkAndUpdateState`). Both moments are
//! the [`Lamp`] part's: `on_place` and `neighbor_changed` get a host that writes the level, so the
//! lamp switches at once, within the placement or the redstone update, as upstream does.

use mod_api::{
    self, BlockPart, BlockUpdateFlags, LevelWriteHost, NeighborChangedEvent, OnPlaceEvent,
    Registrar,
};

use crate::runtime;

/// The lamp's block part, attached to every one of the 84 lamp blocks.
///
/// The hook fires only for the blocks the part is attached to, which is what keeps a neighbour
/// update of an unrelated block from reaching this port at all.
pub struct Lamp;

#[mod_api::sunrise_part]
impl BlockPart for Lamp {
    /// `GemLampBlock.onPlace`: a lamp written into the level by any means (a player, `/setblock`,
    /// `/fill`, a structure, a piston) takes the state for the signal it reads, within the call.
    fn on_place(&self, level: &mut dyn LevelWriteHost, event: &mut OnPlaceEvent) {
        switch_to_signal(level, [event.pos.x, event.pos.y, event.pos.z], event.state);
    }

    /// `GemLampBlock.neighborChanged`: switch to the lamp its table names for the signal now.
    fn neighbor_changed(&self, level: &mut dyn LevelWriteHost, event: &mut NeighborChangedEvent) {
        switch_to_signal(level, [event.pos.x, event.pos.y, event.pos.z], event.state);
    }
}

/// Write the lamp its own table names for the signal `pos` reads now; a lamp already in that
/// state is left untouched.
fn switch_to_signal(level: &mut dyn LevelWriteHost, pos: [i32; 3], state: mod_api::BlockState) {
    let Some((gem, current)) = level.state_block(state).and_then(runtime::lamp_of) else {
        return;
    };
    let Some(powered) = level.has_neighbor_signal(pos) else {
        return;
    };
    let target = current.with_power(powered);
    if target == current {
        return;
    }
    if let Some(state) = level.block_state_id(&target.block_name(gem.name)) {
        // `UPDATE_CLIENTS`, as upstream's `world.setBlock(pos, newState, UPDATE_CLIENTS)`.
        level.set_block(pos, state, BlockUpdateFlags::CLIENTS);
    }
}

/// Attach [`Lamp`] to every lamp block the package declares.
pub fn declare(r: &mut dyn Registrar) {
    for id in runtime::lamp_block_ids() {
        let block = r.block_key(&id);
        r.block_part(block, Lamp);
    }
}
