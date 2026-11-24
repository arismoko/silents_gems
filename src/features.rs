//! The game glue of the features beyond the lamps: glowroses, rabbit coffee, the sparkling bone
//! meal, the pet summoners, the teleporters and their linker. The rules are in [`crate::rules`].

use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Mutex, MutexGuard, OnceLock};

use mod_api::{
    self, AttachmentCodec, AttachmentType, BlockPart, BlockState, BlockUpdateFlags, Component,
    Compound, EntityTickListenedEvent, EntityTickPostEvent, Hand, Host, HostSide,
    InteractionResult, ItemPart, ItemStack, LevelRef, LevelSide, LevelWriteHost,
    LightEmissionEvent, MayPlaceOnEvent, NeighborChangedEvent, PlacementSource, Registrar,
    RotateEvent, ServerHost, SettingSpec, StateForPlacementEvent, StructureRotation, Tag,
    UseOnEvent, UseWithoutItemEvent, WorldgenHost,
};

use crate::rules::{self, Destinations, DimPos, LinkClick};
use crate::runtime::GEMS;

const MOD_ID: &str = "silentgems";
const GLOWROSE_LIGHT: &str = "glowrose_light";
const POTTED_LIGHT: &str = "potted_glowrose_light";
const RABBIT_COFFEE: &str = "rabbits_produce_coffee";
const RABBIT_DELAY: &str = "rabbit_coffee_delay_minutes";

/// The coffee timer `GemsAttachmentTypes.COFFEE_TIMER` keeps on a producer, saved with it.
static COFFEE_TIMER: AttachmentType<i64> = AttachmentType {
    namespace: MOD_ID,
    name: "coffee_timer",
    default: || 0,
    codec: Some(AttachmentCodec {
        encode: |value| {
            let mut out = Compound::new();
            out.insert("coffee_timer".into(), Tag::Long(*value));
            Ok(out)
        },
        decode: |saved| match saved.get(&"coffee_timer".into()) {
            Some(Tag::Long(value)) => Ok(*value),
            _ => Err(mod_api::AttachmentError::Decode),
        },
    }),
    copy_on_death: false,
    sync: false,
};

/// Where each of a level's teleporters sends, kept on the level that holds them.
static DESTINATIONS: AttachmentType<Destinations> = AttachmentType {
    namespace: MOD_ID,
    name: "teleporter_destinations",
    default: Destinations::default,
    codec: Some(AttachmentCodec {
        encode: |value| Ok(value.to_nbt()),
        decode: |saved| Ok(Destinations::from_nbt(saved)),
    }),
    copy_on_death: false,
    sync: false,
};

/// `minecraft:custom_data`, where the linker keeps its stored position
/// (`GemsDataComponents.LINKED_POS`).
struct CustomData;

impl Component for CustomData {
    type Value = Compound;
    fn id(&self) -> &str {
        "minecraft:custom_data"
    }
    fn decode(&self, value: &Tag) -> Option<Compound> {
        match value {
            Tag::Compound(compound) => Some(compound.clone()),
            _ => None,
        }
    }
    fn encode(&self, value: Compound) -> Tag {
        Tag::Compound(value)
    }
}

/// `minecraft:custom_name`, carried over to a summoned pet as `PetSummonerItem` does.
struct CustomName;

impl Component for CustomName {
    type Value = Tag;
    fn id(&self) -> &str {
        "minecraft:custom_name"
    }
    fn decode(&self, value: &Tag) -> Option<Tag> {
        Some(value.clone())
    }
    fn encode(&self, value: Tag) -> Tag {
        value
    }
}

const LINKED_POS: &str = "silentgems_linked_pos";

fn stored_position(stack: &ItemStack) -> Option<DimPos> {
    let data = stack.get(&CustomData)?;
    match data.get(&LINKED_POS.into())? {
        Tag::String(text) => DimPos::decode(&text.to_string().ok()?),
        _ => None,
    }
}

/// A free-standing glowrose: light from its setting, and the extra ground upstream allows.
pub struct Glowrose;

#[mod_api::sunrise_part]
impl BlockPart for Glowrose {
    /// `GlowroseBlock.getLightEmission`.
    fn light_emission(&self, host: &dyn Host, _: &mut LightEmissionEvent) -> Option<u8> {
        Some(rules::light_level(
            host.setting(MOD_ID, GLOWROSE_LIGHT).as_deref(),
            rules::GLOWROSE_LIGHT_DEFAULT,
        ))
    }

    /// `GlowroseBlock.mayPlaceOn`: a flower's own ground, and netherrack-like stone, quartz ore,
    /// nylium and end stone.
    fn may_place_on(&self, level: LevelSide, event: &mut MayPlaceOnEvent) -> Option<bool> {
        let (LevelSide::Server(level) | LevelSide::Client(level)) = level;
        let block = level.state_block(event.ground)?;
        rules::glowrose_extra_ground(block).then_some(true)
    }
}

/// A glowrose in a flower pot: `PottedGlowroseBlock.getLightEmission`, and the pot's own take-out
/// and pick-block, which the template pot would answer with a poppy.
pub struct PottedGlowrose;

fn rose_item_of_potted(block: &str) -> Option<mod_api::ItemKey> {
    ROSES
        .get()?
        .iter()
        .find(|(_, potted)| potted == block)
        .map(|(item, _)| *item)
}

#[mod_api::sunrise_part]
impl BlockPart for PottedGlowrose {
    fn light_emission(&self, host: &dyn Host, _: &mut LightEmissionEvent) -> Option<u8> {
        Some(rules::light_level(
            host.setting(MOD_ID, POTTED_LIGHT).as_deref(),
            rules::POTTED_LIGHT_DEFAULT,
        ))
    }

    /// `FlowerPotBlock.useWithoutItem`: the plant comes out and the pot is empty again.
    fn use_without_item(
        &self,
        host: &mut dyn ServerHost,
        event: &mut UseWithoutItemEvent<'_>,
    ) -> Option<InteractionResult> {
        let click = event.click;
        let level = host.level(click.dimension.key().id())?;
        let block = level.block(&*host, click.pos)?.id(&*host)?.to_owned();
        let rose = ItemStack::new(rose_item_of_potted(&block)?, 1)?;
        let empty = host.block_state_id("minecraft:flower_pot")?;
        level
            .set_block(
                host,
                click.pos,
                empty,
                BlockUpdateFlags::NEIGHBORS | BlockUpdateFlags::CLIENTS,
            )
            .ok()?;
        let player = host.player(click.player)?;
        player.give_stack(host, rose).ok()?;
        Some(InteractionResult::Success)
    }

    /// `getCloneItemStack`: the glowrose, not the template's poppy.
    fn clone_item_stack(
        &self,
        _: &dyn Host,
        event: &mut mod_api::CloneItemStackEvent<'_>,
    ) -> Option<ItemStack> {
        ItemStack::new(rose_item_of_potted(event.block)?, 1)
    }
}

/// Which item is which glowrose, bound at registration: (item, the potted block it makes).
static ROSES: OnceLock<Vec<(mod_api::ItemKey, String)>> = OnceLock::new();

/// The vanilla flower pot: it takes a glowrose, as `FlowerPotBlock.addPlant` lets it
/// (`SilentGems`' `Gems.registerBlocks`).
pub struct EmptyPot;

#[mod_api::sunrise_part]
impl BlockPart for EmptyPot {
    fn use_item_on(
        &self,
        host: &mut dyn ServerHost,
        event: &mut mod_api::UseItemOnEvent<'_>,
    ) -> Option<InteractionResult> {
        let click = event.click;
        let stack = click.stack?;
        let (_, potted) = ROSES
            .get()?
            .iter()
            .find(|(item, _)| *item == stack.item())?;
        let state = host.block_state_id(potted)?;
        let level = host.level(click.dimension.key().id())?;
        level
            .set_block(
                host,
                click.pos,
                state,
                BlockUpdateFlags::NEIGHBORS | BlockUpdateFlags::CLIENTS,
            )
            .ok()?;
        let player = host.player(click.player)?;
        if !player.is_creative(&*host).unwrap_or(false) {
            let slot = match click.hand {
                Hand::Main => u32::try_from(player.selected_slot(&*host)?).ok()?,
                Hand::Off => 40,
            };
            player
                .set_inventory_slot(host, slot, stack.clone().shrink(1))
                .ok()?;
        }
        Some(InteractionResult::Success)
    }
}

/// A gem glass block: `GemGlassBlock.getBeaconColorMultiplier` tints a beacon's beam with the gem.
pub struct GemGlass(pub u32);

#[mod_api::sunrise_part]
impl BlockPart for GemGlass {
    fn beacon_color_multiplier(
        &self,
        _: &dyn Host,
        event: &mut mod_api::BeaconColorMultiplierEvent<'_>,
    ) {
        *event.color = Some(self.0);
    }
}

/// The horizontal directions in turning order, clockwise from north.
const HORIZONTAL: [&str; 4] = ["north", "east", "south", "west"];

/// `BlockPlaceContext.getHorizontalDirection`: a player's facing (`Direction.fromYRot`), or a
/// directional context's own direction, north when it is vertical.
fn horizontal_facing(source: &PlacementSource) -> Option<&'static str> {
    Some(match *source {
        PlacementSource::Player { yaw, .. } => {
            match ((f64::from(yaw) / 90.0 + 0.5).floor() as i64).rem_euclid(4) {
                0 => "south",
                1 => "west",
                2 => "north",
                _ => "east",
            }
        }
        PlacementSource::Directional { direction } => match direction {
            mod_api::Direction::East => "east",
            mod_api::Direction::South => "south",
            mod_api::Direction::West => "west",
            _ => "north",
        },
        _ => return None,
    })
}

/// `AbstractTeleporterBlock.getStateForPlacement`: the default state facing the placer's way.
fn teleporter_placement(
    level: LevelSide,
    event: &StateForPlacementEvent<'_>,
) -> Option<Option<BlockState>> {
    let (LevelSide::Server(level) | LevelSide::Client(level)) = level;
    let facing = horizontal_facing(&event.context.source)?;
    Some(level.state_with_property(event.state, "facing", facing))
}

/// `AbstractTeleporterBlock.rotate`: `rotation.rotate(facing)`.
fn teleporter_rotate(context: &dyn WorldgenHost, event: &mut RotateEvent<'_>) {
    let turns = match event.rotation {
        StructureRotation::Clockwise90 => 1,
        StructureRotation::Clockwise180 => 2,
        StructureRotation::Counterclockwise90 => 3,
        _ => return,
    };
    let Some(now) = HORIZONTAL
        .iter()
        .position(|f| context.state_with_property(event.state, "facing", f) == Some(event.state))
    else {
        return;
    };
    if let Some(state) =
        context.state_with_property(event.state, "facing", HORIZONTAL[(now + turns) % 4])
    {
        *event.rotated = state;
    }
}

/// A teleporter and the anchor: a click sends the clicker to the saved destination.
pub struct Teleporter {
    /// The anchor takes no click of its own (`TeleporterAnchorBlock.useWithoutItem`).
    pub anchor: bool,
}

#[mod_api::sunrise_part]
impl BlockPart for Teleporter {
    /// `AbstractTeleporterBlock.useWithoutItem`.
    fn use_without_item(
        &self,
        host: &mut dyn ServerHost,
        event: &mut UseWithoutItemEvent<'_>,
    ) -> Option<InteractionResult> {
        if self.anchor {
            return Some(InteractionResult::Pass);
        }
        let click = event.click;
        if click.stack.is_some_and(is_linker) {
            return Some(InteractionResult::Pass);
        }
        let level = host.level(click.dimension.key().id())?;
        if let Some(destination) = destination_of(host, click.pos) {
            let player = host.entity(u32::try_from(click.player_entity_id).ok()?)?;
            teleport(host, level, player, &destination);
        }
        Some(InteractionResult::Success)
    }

    fn state_for_placement(
        &self,
        level: LevelSide,
        event: &mut StateForPlacementEvent<'_>,
    ) -> Option<Option<BlockState>> {
        teleporter_placement(level, event)
    }

    fn rotate(&self, context: &dyn WorldgenHost, event: &mut RotateEvent<'_>) {
        teleporter_rotate(context, event);
    }
}

/// A redstone teleporter: sends everything within five blocks when a signal arrives.
pub struct RedstoneTeleporter;

#[mod_api::sunrise_part]
impl BlockPart for RedstoneTeleporter {
    fn use_without_item(
        &self,
        host: &mut dyn ServerHost,
        event: &mut UseWithoutItemEvent<'_>,
    ) -> Option<InteractionResult> {
        Teleporter { anchor: false }.use_without_item(host, event)
    }

    fn state_for_placement(
        &self,
        level: LevelSide,
        event: &mut StateForPlacementEvent<'_>,
    ) -> Option<Option<BlockState>> {
        teleporter_placement(level, event)
    }

    fn rotate(&self, context: &dyn WorldgenHost, event: &mut RotateEvent<'_>) {
        teleporter_rotate(context, event);
    }

    /// `GemRedstoneTeleporterBlock.neighborChanged`: remember the block for this tick's pass, as
    /// moving entities needs the server host this hook doesn't lend.
    fn neighbor_changed(&self, _: &mut dyn LevelWriteHost, event: &mut NeighborChangedEvent) {
        queue()
            .lock_pending()
            .redstone
            .entry(event.dimension.key().id().to_owned())
            .or_default()
            .insert([event.pos.x, event.pos.y, event.pos.z]);
    }
}

fn is_linker(stack: &ItemStack) -> bool {
    stack.item() == linker_key()
}

fn linker_key() -> mod_api::ItemKey {
    *LINKER
        .get()
        .expect("the linker key is bound at registration")
}

static CUP: OnceLock<mod_api::ItemKey> = OnceLock::new();
static LINKER: OnceLock<mod_api::ItemKey> = OnceLock::new();

fn destination_of(host: &mut dyn ServerHost, pos: [i32; 3]) -> Option<DimPos> {
    let all = DESTINATIONS
        .get(host.attachments(), mod_api::Holder::Level)
        .ok()?;
    all.0.get(&pos).cloned()
}

/// Queue a destination for the level that holds the teleporter, which may not be the one in place:
/// the write lands when that level ticks.
fn set_destination(at: &DimPos, destination: DimPos) {
    queue()
        .lock_pending()
        .sets
        .entry(at.dimension.clone())
        .or_default()
        .push((at.pos, destination));
}

/// `AbstractTeleporterBlock.teleportEntity` with its sound at each end.
fn teleport(host: &mut dyn ServerHost, from: LevelRef, entity: mod_api::EntityRef, to: &DimPos) {
    let from_pos = entity.position(&*host).unwrap_or([0.0; 3]);
    from.play_sound(
        host,
        "minecraft:entity.player.teleport",
        from_pos,
        mod_api::SoundSource::Blocks,
        0.7,
        1.2,
        None,
    );
    if entity.teleport_to_level(host, &to.dimension, to.centre(), None)
        && let Some(level) = host.level(&to.dimension)
    {
        level.play_sound(
            host,
            "minecraft:entity.player.teleport",
            to.centre(),
            mod_api::SoundSource::Blocks,
            0.7,
            1.2,
            None,
        );
    }
}

/// The teleporter linker: `TeleporterLinker.useOn`.
pub struct Linker;

#[mod_api::sunrise_part]
impl ItemPart for Linker {
    fn use_on(&self, host: HostSide, event: &mut UseOnEvent<'_>) -> Option<InteractionResult> {
        let HostSide::Server(host) = host else {
            return Some(InteractionResult::Success);
        };
        let dimension = host.dimension();
        let level = host.level(dimension)?;
        let block = level.block(&*host, event.pos)?;
        if !block.id(&*host).is_some_and(is_teleporter) {
            return None;
        }
        let clicked = DimPos {
            dimension: dimension.to_owned(),
            pos: event.pos,
        };
        let player = host.player(event.player)?;
        let mut stack = event.stack.clone();
        let message = match rules::link_click(stored_position(&stack), clicked) {
            LinkClick::Store(place) => {
                let mut data = stack.get(&CustomData).unwrap_or_default();
                data.insert(
                    LINKED_POS.into(),
                    Tag::String(place.encode().as_str().into()),
                );
                stack.set(&CustomData, data).ok()?;
                "Teleporter position stored!"
            }
            LinkClick::Link { first, second } => {
                for (place, destination) in [first, second] {
                    set_destination(&place, destination);
                }
                let mut data = stack.get(&CustomData).unwrap_or_default();
                data.remove(&LINKED_POS.into());
                if data.is_empty() {
                    stack.remove(&CustomData);
                } else {
                    stack.set(&CustomData, data).ok()?;
                }
                "Teleporters linked!"
            }
        };
        let slot = match event.hand {
            Hand::Main => u32::try_from(player.selected_slot(&*host)?).ok()?,
            Hand::Off => 40,
        };
        player.set_inventory_slot(host, slot, Some(stack)).ok()?;
        let _ = player.action_bar(host, mod_api::Text::literal(message));
        Some(InteractionResult::Success)
    }
}

fn is_teleporter(id: &str) -> bool {
    id == "silentgems:teleporter_anchor"
        || GEMS.iter().any(|gem| {
            id.strip_prefix("silentgems:")
                .and_then(|name| name.strip_prefix(gem.name))
                .is_some_and(|rest| rest == "_teleporter" || rest == "_redstone_teleporter")
        })
}

/// A pet summoner: `PetSummonerItem.useOn`.
pub struct Summoner {
    /// The vanilla entity it calls, `minecraft:cat` or `minecraft:wolf`.
    pub kind: &'static str,
}

#[mod_api::sunrise_part]
impl ItemPart for Summoner {
    fn use_on(&self, host: HostSide, event: &mut UseOnEvent<'_>) -> Option<InteractionResult> {
        let HostSide::Server(host) = host else {
            return Some(InteractionResult::Success);
        };
        let player = host.player(event.player)?;
        let target = [
            event.pos[0] + event.face[0],
            event.pos[1] + event.face[1],
            event.pos[2] + event.face[2],
        ];
        let face = match event.face {
            [0, -1, 0] => mod_api::BlockFace::Down,
            [0, 1, 0] => mod_api::BlockFace::Up,
            [0, 0, -1] => mod_api::BlockFace::North,
            [0, 0, 1] => mod_api::BlockFace::South,
            [1, 0, 0] => mod_api::BlockFace::East,
            [-1, 0, 0] => mod_api::BlockFace::West,
            _ => return Some(InteractionResult::Pass),
        };
        if !player.may_use_item_at(&*host, target, face, event.stack)? {
            return Some(InteractionResult::Pass);
        }
        let level = host.level(host.dimension())?;
        let at = [
            f64::from(target[0]),
            f64::from(target[1]),
            f64::from(target[2]),
        ];
        let pet = level.spawn_mob(host, self.kind, at, mod_api::SpawnReason::MobSummoned)?;
        let mut tame = Compound::new();
        let uuid = event.player;
        tame.insert(
            "Owner".into(),
            Tag::IntArray(vec![
                (uuid >> 96) as i32,
                (uuid >> 64) as i32,
                (uuid >> 32) as i32,
                uuid as i32,
            ]),
        );
        pet.merge_nbt(host, tame);
        if let Some(name) = event.stack.get(&CustomName) {
            let mut named = Compound::new();
            named.insert("CustomName".into(), name);
            pet.merge_nbt(host, named);
        }
        if !player.is_creative(&*host).unwrap_or(false) {
            let slot = match event.hand {
                Hand::Main => u32::try_from(player.selected_slot(&*host)?).ok()?,
                Hand::Off => 40,
            };
            player
                .set_inventory_slot(host, slot, event.stack.clone().shrink(1))
                .ok()?;
        }
        Some(InteractionResult::Success)
    }
}

/// Sparkling bone meal: `SparklingBoneMealItem.useOn`.
pub struct SparklingBoneMeal;

#[mod_api::sunrise_part]
impl ItemPart for SparklingBoneMeal {
    fn use_on(&self, host: HostSide, event: &mut UseOnEvent<'_>) -> Option<InteractionResult> {
        let HostSide::Server(host) = host else {
            return Some(InteractionResult::Success);
        };
        let level = host.level(host.dimension())?;
        if level.block(&*host, event.pos)?.id(&*host) != Some("minecraft:grass_block") {
            return None;
        }
        let above = [event.pos[0], event.pos[1] + 1, event.pos[2]];
        let mut rng =
            rules::Rng::new(level.game_time(&*host).unwrap_or(0) as u64 ^ event.player as u64);
        for _ in 0..rules::BONE_MEAL_CLUSTERS {
            let gem = GEMS[rng.below(GEMS.len() as u32) as usize];
            let rose = host.block_state_id(&format!("silentgems:{}_glowrose", gem.name))?;
            if !level
                .block(&*host, above)?
                .id(&*host)
                .is_some_and(|id| id == "minecraft:air")
            {
                continue;
            }
            for _ in 0..rules::BONE_MEAL_TRIES {
                let draws = [rng.below(8), rng.below(6), rng.below(8)];
                let offset = rules::bone_meal_offset(draws);
                let target = [
                    above[0] + offset[0],
                    above[1] + offset[1],
                    above[2] + offset[2],
                ];
                let ground = [target[0], target[1] - 1, target[2]];
                let on_grass = level
                    .block(&*host, ground)
                    .and_then(|b| b.id(&*host).map(|id| id == "minecraft:grass_block"));
                let is_air = level
                    .block(&*host, target)
                    .and_then(|b| b.id(&*host).map(|id| id == "minecraft:air"));
                if on_grass == Some(true) && is_air == Some(true) {
                    let _ = level.set_block(
                        host,
                        target,
                        rose,
                        BlockUpdateFlags::NEIGHBORS | BlockUpdateFlags::CLIENTS,
                    );
                }
            }
        }
        let player = host.player(event.player)?;
        let slot = match event.hand {
            Hand::Main => u32::try_from(player.selected_slot(&*host)?).ok()?,
            Hand::Off => 40,
        };
        player
            .set_inventory_slot(host, slot, event.stack.clone().shrink(1))
            .ok()?;
        Some(InteractionResult::Consume)
    }
}

/// What the mod queues between a hook that cannot write and the tick pass that can.
#[derive(Default)]
pub struct Pending {
    /// Redstone teleporters whose neighbours changed, per dimension.
    pub redstone: BTreeMap<String, BTreeSet<[i32; 3]>>,
    /// Destinations to save, per dimension of the teleporter they belong to.
    pub sets: BTreeMap<String, Vec<([i32; 3], DimPos)>>,
}

struct Queue(Mutex<Pending>);

impl Queue {
    fn lock_pending(&self) -> MutexGuard<'_, Pending> {
        self.0
            .lock()
            .expect("the pending queue lock is never held across a panic")
    }
}

fn queue() -> &'static Queue {
    static QUEUE: OnceLock<Queue> = OnceLock::new();
    QUEUE.get_or_init(|| Queue(Mutex::new(Pending::default())))
}

/// Declare the settings, the attachment types and every part.
pub fn declare(r: &mut dyn Registrar) {
    let _ = r.register_setting(SettingSpec::range(
        GLOWROSE_LIGHT,
        "silentgems.configuration.glowrose",
        0.0,
        15.0,
        1.0,
        rules::GLOWROSE_LIGHT_DEFAULT,
    ));
    let _ = r.register_setting(SettingSpec::range(
        POTTED_LIGHT,
        "silentgems.configuration.pottedGlowrose",
        0.0,
        15.0,
        1.0,
        rules::POTTED_LIGHT_DEFAULT,
    ));
    let _ = r.register_setting(SettingSpec::toggle(
        RABBIT_COFFEE,
        "silentgems.configuration.rabbitsProduceCoffee",
        true,
    ));
    let _ = r.register_setting(SettingSpec::range(
        RABBIT_DELAY,
        "silentgems.configuration.rabbitCoffeeDelay",
        rules::COFFEE_DELAY_MINUTES_RANGE.0,
        rules::COFFEE_DELAY_MINUTES_RANGE.1,
        1.0,
        rules::COFFEE_DELAY_MINUTES_DEFAULT,
    ));
    let _ = r.register_attachment_type("silentgems:coffee_timer", false);
    let _ = r.register_attachment_type("silentgems:teleporter_destinations", false);

    for gem in GEMS {
        let rose = r.block_key(&format!("silentgems:{}_glowrose", gem.name));
        r.block_part(rose, Glowrose);
        let potted = r.block_key(&format!("silentgems:potted_{}_glowrose", gem.name));
        r.block_part(potted, PottedGlowrose);
        let glass = r.block_key(&format!("silentgems:{}_glass", gem.name));
        r.block_part(glass, GemGlass(gem.color));
        let plain = r.block_key(&format!("silentgems:{}_teleporter", gem.name));
        r.block_part(plain, Teleporter { anchor: false });
        let redstone = r.block_key(&format!("silentgems:{}_redstone_teleporter", gem.name));
        r.block_part(redstone, RedstoneTeleporter);
    }
    let pot = r.block_key("minecraft:flower_pot");
    r.block_part(pot, EmptyPot);
    let roses = GEMS
        .iter()
        .map(|gem| {
            (
                r.item_key(&format!("silentgems:{}_glowrose", gem.name)),
                format!("silentgems:potted_{}_glowrose", gem.name),
            )
        })
        .collect();
    let _ = ROSES.set(roses);
    let anchor = r.block_key("silentgems:teleporter_anchor");
    r.block_part(anchor, Teleporter { anchor: true });

    let _ = CUP.set(r.item_key("silentgems:cup_of_coffee"));
    let linker = r.item_key("silentgems:teleporter_linker");
    let _ = LINKER.set(linker);
    r.item_part(linker, Linker);
    let bone_meal = r.item_key("silentgems:sparkling_bone_meal");
    r.item_part(bone_meal, SparklingBoneMeal);
    let kitty = r.item_key("silentgems:summon_kitty");
    r.item_part(
        kitty,
        Summoner {
            kind: "minecraft:cat",
        },
    );
    let puppy = r.item_key("silentgems:summon_puppy");
    r.item_part(
        puppy,
        Summoner {
            kind: "minecraft:wolf",
        },
    );
}

/// `entity_tick_listened`: the rabbit coffee needs every entity's tick told.
pub fn on_tick_listened(_: HostSide, event: &mut EntityTickListenedEvent) {
    event.listened = true;
}

/// `GemsEvents.handleCoffeeTimerTicks`: each coffee producer counts its timer and drops a cup.
pub fn on_entity_tick_post(host: HostSide, event: &mut EntityTickPostEvent<'_>) {
    let HostSide::Server(host) = host else {
        return;
    };
    if host.setting(MOD_ID, RABBIT_COFFEE).as_deref() == Some("false") {
        return;
    }
    let delay = rules::coffee_delay_ticks(host.setting(MOD_ID, RABBIT_DELAY).as_deref());
    for (&id, &kind) in event.entities.iter().zip(event.kinds) {
        if kind != "minecraft:rabbit" {
            continue;
        }
        let Some(rabbit) = host.entity(id) else {
            continue;
        };
        let holder = mod_api::Holder::Entity(rabbit.id());
        let Ok(timer) = COFFEE_TIMER.get(host.attachments(), holder) else {
            continue;
        };
        let (next, drops) = rules::coffee_step(timer, delay);
        let _ = COFFEE_TIMER.set(host.attachments(), holder, next);
        if drops {
            let Some(cup) = CUP.get() else {
                continue;
            };
            if let Some(stack) = ItemStack::new(*cup, 1) {
                let _ = rabbit.spawn_at_location(host, stack, 0.0);
            }
            if let Some(level) = host.level(event.dimension)
                && let Some(at) = rabbit.position(&*host)
            {
                level.play_sound(
                    host,
                    "minecraft:block.froglight.step",
                    at,
                    mod_api::SoundSource::Neutral,
                    1.0,
                    1.0,
                    None,
                );
            }
        }
    }
}

/// What the redstone teleporter's search leaves alone: `c:teleporting_not_supported`.
const NOT_TELEPORTABLE: [&str; 2] = ["minecraft:ender_dragon", "minecraft:wither"];

/// After the level's tick: save the queued destinations, then run the redstone teleporters whose
/// neighbours changed (`GemRedstoneTeleporterBlock.neighborChanged`).
pub fn on_level_tick_post(host: &mut dyn ServerHost, level: LevelRef) {
    if !level.in_place(&*host) {
        return;
    }
    let Some(dimension) = level.dimension().id(&*host) else {
        return;
    };
    let (sets, redstone) = {
        let mut pending = queue().lock_pending();
        (
            pending.sets.remove(dimension),
            pending.redstone.remove(dimension),
        )
    };
    if let Some(sets) = sets {
        let holder = mod_api::Holder::Level;
        if let Ok(mut all) = DESTINATIONS.get(host.attachments(), holder) {
            all.0.extend(sets);
            let _ = DESTINATIONS.set(host.attachments(), holder, all);
        }
    }
    for pos in redstone.unwrap_or_default() {
        redstone_pass(host, level, pos);
    }
}

fn redstone_pass(host: &mut dyn ServerHost, level: LevelRef, pos: [i32; 3]) {
    let Some(block) = level.block(&*host, pos) else {
        return;
    };
    if !block
        .id(&*host)
        .is_some_and(|id| id.ends_with("_redstone_teleporter") && is_teleporter(id))
    {
        return;
    }
    let above = [pos[0], pos[1] + 1, pos[2]];
    let powered =
        level.has_neighbor_signal(&*host, pos) || level.has_neighbor_signal(&*host, above);
    let triggered = level.block_property(&*host, pos, "triggered").as_deref() == Some("true");
    if powered == triggered {
        return;
    }
    if powered {
        teleport_nearby(host, level, pos);
    }
    let _ =
        host.set_block_property_with(pos, "triggered", if powered { "true" } else { "false" }, 2);
}

/// `tryTeleportNearbyEntities`: everything within five blocks goes to the destination.
fn teleport_nearby(host: &mut dyn ServerHost, level: LevelRef, pos: [i32; 3]) {
    let Some(destination) = destination_of(host, pos) else {
        return;
    };
    let min = [
        f64::from(pos[0] - 5),
        f64::from(pos[1] - 5),
        f64::from(pos[2] - 5),
    ];
    let max = [
        f64::from(pos[0] + 6),
        f64::from(pos[1] + 6),
        f64::from(pos[2] + 6),
    ];
    let mut found = Vec::new();
    let _ = level.visit_entities_in(&*host, min, max, &mut |entity| {
        found.push(entity);
        std::ops::ControlFlow::Continue(())
    });
    let mut moved = false;
    for entity in found {
        if entity
            .kind(&*host)
            .and_then(|kind| kind.id(&*host))
            .is_some_and(|kind| NOT_TELEPORTABLE.contains(&kind.as_str()))
        {
            continue;
        }
        moved |= entity.teleport_to_level(host, &destination.dimension, destination.centre(), None);
    }
    if moved {
        let at = [
            f64::from(pos[0]) + 0.5,
            f64::from(pos[1]),
            f64::from(pos[2]) + 0.5,
        ];
        level.play_sound(
            host,
            "minecraft:entity.player.teleport",
            at,
            mod_api::SoundSource::Blocks,
            0.7,
            1.2,
            None,
        );
        if let Some(target) = host.level(&destination.dimension) {
            target.play_sound(
                host,
                "minecraft:entity.player.teleport",
                destination.centre(),
                mod_api::SoundSource::Blocks,
                0.7,
                1.2,
                None,
            );
        }
    }
}

/// A world closed: what was queued for it never lands in the next one opened in this session.
pub fn clear_pending() {
    *queue().lock_pending() = Pending::default();
}

#[cfg(test)]
mod facing_tests {
    use super::*;

    #[test]
    fn placement_faces_where_the_player_looks() {
        let facing = |yaw| {
            horizontal_facing(&PlacementSource::Player {
                yaw,
                pitch: 0.0,
                sneaking: false,
            })
        };
        assert_eq!(facing(0.0), Some("south"));
        assert_eq!(facing(90.0), Some("west"));
        assert_eq!(facing(180.0), Some("north"));
        assert_eq!(facing(-90.0), Some("east"));
        assert_eq!(facing(44.9), Some("south"));
        assert_eq!(facing(45.0), Some("west"));
        let up = PlacementSource::Directional {
            direction: mod_api::Direction::Up,
        };
        assert_eq!(horizontal_facing(&up), Some("north"));
    }
}
