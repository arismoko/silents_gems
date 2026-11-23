//! The lamp rule's game glue: a queued neighbour write, applied after the level's tick.
//!
//! Upstream's `GemLampBlock` switches its own block to the one its state table names, on place,
//! on a neighbour change and on a scheduled tick (`block/GemLampBlock.java:44-66`). A `BlockPart`
//! is the scoped way to hear the neighbour change, but that hook's host cannot write the level,
//! so this module keeps the positions it must revisit and applies them at
//! `Hooks::level_tick_post`, where the level can be written — the
//! same one-tick shape More Red and Glassential use.
//!
//! The queue holds positions only: at the tick the current block id is parsed back to
//! (gem, state) with [`crate::runtime::lamp_of`], so a position whose block changed in between
//! (broken, replaced by a player) is simply dropped. A write made here reports `block_changed`
//! like any other; that queues the position once more, and the next tick finds the target equal
//! to the current state and writes nothing, so the rule settles in one pass.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Mutex, MutexGuard, OnceLock};

use mod_api::{
    self, BlockChangedEvent, BlockPart, BlockUpdateFlags, LevelHost, LevelRef, LevelTicksHost,
    NeighborChangedEvent, Registrar, ServerHost,
};

use crate::runtime;

/// The lamp's block part, attached to every one of the 84 lamp blocks.
///
/// The hook fires only for the blocks the part is attached to, which is what keeps a neighbour
/// update of an unrelated block from reaching this port at all.
pub struct Lamp;

#[mod_api::sunrise_part]
impl BlockPart for Lamp {
    /// `GemLampBlock.neighborChanged`: remember the lamp for this tick's write pass.
    fn neighbor_changed(&self, _level: &mut dyn LevelTicksHost, event: &mut NeighborChangedEvent) {
        enqueue(
            event.dimension.key().id(),
            [event.pos.x, event.pos.y, event.pos.z],
        );
    }
}

/// How many positions one level may keep queued. A `/fill` of lamp blocks enqueues one entry per
/// block, and the pass below drains them all; the cap only bounds a pathological edit.
const MAX_QUEUED: usize = 8192;

/// The positions a lamp rule still owes, per dimension.
#[derive(Default)]
struct Queue {
    by_level: BTreeMap<&'static str, BTreeSet<[i32; 3]>>,
}

fn locked() -> MutexGuard<'static, Queue> {
    static QUEUE: OnceLock<Mutex<Queue>> = OnceLock::new();
    QUEUE
        .get_or_init(|| Mutex::new(Queue::default()))
        .lock()
        .expect("the lamp queue lock is never held across a panic")
}

fn enqueue(dimension: &'static str, pos: [i32; 3]) {
    let mut queue = locked();
    let waiting = queue.by_level.entry(dimension).or_default();
    if waiting.len() < MAX_QUEUED {
        waiting.insert(pos);
    }
}

fn take(dimension: &str) -> Vec<[i32; 3]> {
    locked()
        .by_level
        .remove(dimension)
        .map(|waiting| waiting.into_iter().collect())
        .unwrap_or_default()
}

/// Attach [`Lamp`] to every lamp block the package declares.
pub fn declare(r: &mut dyn Registrar) {
    for id in runtime::lamp_block_ids() {
        let block = r.block_key(&id);
        r.block_part(block, Lamp);
    }
}

/// A block was committed in the world: a lamp that was placed (or written) is due a check.
///
/// This is the `onPlace` half of upstream's rule. It runs for every block change in the level, so
/// it rejects anything whose new block id is not one of the 84 before it touches the queue.
pub fn on_block_changed(level: &dyn LevelHost, event: &mut BlockChangedEvent) {
    let Some(id) = level.state_block(event.new) else {
        return;
    };
    if runtime::lamp_of(id).is_none() {
        return;
    }
    if event.old == event.new {
        return;
    }
    enqueue(
        event.dimension.key().id(),
        [event.pos.x, event.pos.y, event.pos.z],
    );
}

/// Apply the queued lamps for the level that just ticked.
///
/// A lamp takes the state its own table names for the signal it reads now
/// (`runtime::LampState::with_power`); a lamp already in that state is left untouched, which is
/// what makes the pass idempotent and the reported `block_changed` harmless.
pub fn on_level_tick_post(host: &mut dyn ServerHost, level: LevelRef) {
    if !level.in_place(&*host) {
        return;
    }
    let Some(dimension) = level.dimension().id(&*host) else {
        return;
    };
    let waiting = take(dimension);
    if waiting.is_empty() {
        return;
    }
    for pos in waiting {
        let Some(block) = level.block(&*host, pos) else {
            continue;
        };
        let Some((gem, current)) = block.id(&*host).and_then(runtime::lamp_of) else {
            continue;
        };
        let powered = level.best_neighbor_signal(&*host, pos).unwrap_or(0) > 0;
        let target = current.with_power(powered);
        if target == current {
            continue;
        }
        let Some(state) = host.block_state_id(&target.block_name(gem.name)) else {
            continue;
        };
        // `UPDATE_CLIENTS`, as upstream's `world.setBlock(pos, newState, UPDATE_CLIENTS)`:
        // the state change reaches the client without a neighbour broadcast that would queue
        // every lamp beside it again.
        let _ = level.set_block(host, pos, state, BlockUpdateFlags::CLIENTS);
    }
}
