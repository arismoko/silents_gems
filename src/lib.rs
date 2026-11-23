//! Silent's Gems for Sunrise — the one `impl Hooks` at the crate root.
//!
//! Ported from Silent's Gems 5.2.0 (`SilentChaos512/SilentGems`, MIT), migrated from the retired
//! `.hnmod` container to the flat SDK package (`api = "0.2"`). The package carries three slices:
//! the 21 gems and their storage blocks, the 84 ores with their overworld/nether/end generation,
//! and the 84 gem lamps. Everything except the lamps is content in `data/` and `assets/`; the
//! lamp rule lives in [`runtime`] (the state table, tested by `cargo test` alone) and its game
//! glue in [`support`] (one `BlockPart` and the tick write).
//!
//! | Moment | Where | What it does |
//! |---|---|---|
//! | `register` | this module | attaches the lamp part to the 84 lamp blocks |
//! | `neighbor_changed` | [`support::Lamp`] Part | switches a lamp for the signal it reads |
//! | `block_changed` | this module | queues a lamp that was placed or written |
//! | `level_tick_post` | this module | writes each queued lamp's state for the signal it reads |

pub mod features;
pub mod rules;
pub mod runtime;
pub mod support;

use mod_api::{
    self, BlockChangedEvent, EntityTickListenedEvent, EntityTickPostEvent, Hooks, HostSide,
    LevelHost, LevelTickPostEvent, Registrar, RegistrationContext, ServerHost, ServerStoppedEvent,
};

/// The SDK's one `Default` session instance; the queue lives behind the lock in [`support`],
/// because the `BlockPart` is constructed at registration and outlives this value.
#[derive(Default)]
struct Mod;

#[mod_api::sunrise_mod]
impl Hooks for Mod {
    /// Attach the lamp part to every lamp block the package declares.
    fn register(&self, r: &mut dyn Registrar, _: &RegistrationContext) {
        support::declare(r);
        features::declare(r);
    }

    /// A world closed: drop what was queued for it.
    fn server_stopped(&self, _: &mut dyn ServerHost, _: &mut ServerStoppedEvent) {
        features::clear_pending();
        support::clear_queue();
    }

    /// Rabbit coffee needs every entity's tick told.
    fn entity_tick_listened(&self, host: HostSide, event: &mut EntityTickListenedEvent) {
        features::on_tick_listened(host, event);
    }

    /// After entities tick: each rabbit counts its coffee timer.
    fn entity_tick_post(&self, host: HostSide, event: &mut EntityTickPostEvent<'_>) {
        features::on_entity_tick_post(host, event);
    }

    /// A block was committed: a lamp that was placed (or written) is due a check.
    fn block_changed(&self, level: &dyn LevelHost, event: &mut BlockChangedEvent) {
        support::on_block_changed(level, event);
    }

    /// After a level's tick pass, in the level that just ticked: apply the queued lamp states.
    fn level_tick_post(&self, host: HostSide<'_>, event: &mut LevelTickPostEvent) {
        let HostSide::Server(host) = host else {
            return;
        };
        features::on_level_tick_post(host, event.level);
        support::on_level_tick_post(host, event.level);
    }
}
