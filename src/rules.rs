//! The pure rules of the features beyond the lamps, testable by `cargo test` alone.
//!
//! Everything here follows Silent's Gems 5.2.0: `GemsConfig` (light and rabbit settings),
//! `GemsEvents.handleCoffeeTimerTicks` (the coffee timer), `GlowroseBlock.mayPlaceOn`,
//! `SparklingBoneMealItem.useOn` and `TeleporterLinker`/`TeleporterBlockEntity` (saved
//! destinations). The game glue lives in [`crate::features`].

use std::collections::BTreeMap;

use mod_api::{Compound, Tag};

/// `glowrose.normalLight`'s default.
pub const GLOWROSE_LIGHT_DEFAULT: f64 = 10.0;
/// `glowrose.pottedLight`'s default.
pub const POTTED_LIGHT_DEFAULT: f64 = 15.0;
/// `features.rabbitCoffeeDelay`'s default: ten minutes of ticks.
pub const COFFEE_DELAY_MINUTES_DEFAULT: f64 = 10.0;
/// The smallest and largest coffee delay, in minutes (`TimeUtils.ticksFromMinutes(1)` and
/// `ticksFromHours(24)`).
pub const COFFEE_DELAY_MINUTES_RANGE: (f64, f64) = (1.0, 1440.0);
const TICKS_PER_MINUTE: i64 = 1200;

/// A light setting as a block light level, 0 to 15 (`defineInRange("glowrose.normalLight", 10, 0, 15)`).
pub fn light_level(setting: Option<&str>, default: f64) -> u8 {
    let value = setting
        .and_then(|text| text.trim().parse::<f64>().ok())
        .unwrap_or(default);
    value.round().clamp(0.0, 15.0) as u8
}

/// The rabbit coffee delay setting, in ticks.
pub fn coffee_delay_ticks(setting: Option<&str>) -> i64 {
    let minutes = setting
        .and_then(|text| text.trim().parse::<f64>().ok())
        .unwrap_or(COFFEE_DELAY_MINUTES_DEFAULT)
        .clamp(COFFEE_DELAY_MINUTES_RANGE.0, COFFEE_DELAY_MINUTES_RANGE.1);
    minutes.round() as i64 * TICKS_PER_MINUTE
}

/// `handleCoffeeTimerTicks`: at the delay the animal drops a cup and starts over, else the timer
/// counts one more tick. Returns the new timer and whether a cup drops.
pub fn coffee_step(timer: i64, delay: i64) -> (i64, bool) {
    if timer >= delay {
        (0, true)
    } else {
        (timer + 1, false)
    }
}

/// The ground blocks `GlowroseBlock.mayPlaceOn` adds to a flower's own: `BASE_STONE_NETHER`,
/// nether quartz ore, `NYLIUM` and NeoForge's `END_STONES`.
pub fn glowrose_extra_ground(block: &str) -> bool {
    matches!(
        block,
        "minecraft:netherrack"
            | "minecraft:basalt"
            | "minecraft:blackstone"
            | "minecraft:nether_quartz_ore"
            | "minecraft:crimson_nylium"
            | "minecraft:warped_nylium"
            | "minecraft:end_stone"
    )
}

/// Which spot offsets `SparklingBoneMealItem` scatters a glowrose onto: four clusters of 32 tries
/// around the block above the clicked grass, each at `(-4..4, -3..3, -4..4)`.
pub const BONE_MEAL_CLUSTERS: usize = 4;
/// Tries per cluster.
pub const BONE_MEAL_TRIES: usize = 32;

/// `RandomSource.nextInt(min, max)` offsets of one try, from three draws below the bounds.
pub fn bone_meal_offset(draws: [u32; 3]) -> [i32; 3] {
    [
        (draws[0] % 8) as i32 - 4,
        (draws[1] % 6) as i32 - 3,
        (draws[2] % 8) as i32 - 4,
    ]
}

/// A small xorshift source for the bone meal's scatter, seeded from the game time and the player.
pub struct Rng(u64);

impl Rng {
    /// A source for `seed`.
    pub fn new(seed: u64) -> Self {
        Self(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1)
    }

    /// A number from 0 up to `bound`.
    pub fn below(&mut self, bound: u32) -> u32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        ((self.0 >> 32) % u64::from(bound)) as u32
    }
}

/// A place in a dimension: `DimPos`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DimPos {
    /// The dimension's id (`minecraft:the_nether`).
    pub dimension: String,
    /// The block position.
    pub pos: [i32; 3],
}

impl DimPos {
    /// One block up, as the linker sets a destination (`offset(Direction.UP, 1)`).
    pub fn above(&self) -> Self {
        Self {
            dimension: self.dimension.clone(),
            pos: [self.pos[0], self.pos[1] + 1, self.pos[2]],
        }
    }

    /// Saved as `dimension|x|y|z`.
    pub fn encode(&self) -> String {
        format!(
            "{}|{}|{}|{}",
            self.dimension, self.pos[0], self.pos[1], self.pos[2]
        )
    }

    /// The inverse of [`Self::encode`].
    pub fn decode(text: &str) -> Option<Self> {
        let mut parts = text.rsplitn(4, '|');
        let z = parts.next()?.parse().ok()?;
        let y = parts.next()?.parse().ok()?;
        let x = parts.next()?.parse().ok()?;
        let dimension = parts.next()?;
        if dimension.is_empty() {
            return None;
        }
        Some(Self {
            dimension: dimension.to_owned(),
            pos: [x, y, z],
        })
    }

    /// Where an entity stands when sent here: the middle of the block.
    pub fn centre(&self) -> [f64; 3] {
        [
            f64::from(self.pos[0]) + 0.5,
            f64::from(self.pos[1]),
            f64::from(self.pos[2]) + 0.5,
        ]
    }
}

/// A level's teleporters and where each one sends: `TeleporterBlockEntity`'s `Destination`, kept
/// by position on the level that holds the blocks.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Destinations(pub BTreeMap<[i32; 3], DimPos>);

impl Destinations {
    /// Saved as one string per teleporter, keyed `x,y,z`.
    pub fn to_nbt(&self) -> Compound {
        let mut out = Compound::new();
        for (pos, destination) in &self.0 {
            out.insert(
                format!("{},{},{}", pos[0], pos[1], pos[2]).as_str().into(),
                Tag::String(destination.encode().as_str().into()),
            );
        }
        out
    }

    /// The inverse of [`Self::to_nbt`]; an entry that does not parse is dropped.
    pub fn from_nbt(saved: &Compound) -> Self {
        let mut map = BTreeMap::new();
        for (key, value) in saved.iter() {
            let Tag::String(text) = value else { continue };
            let (Ok(key), Ok(text)) = (key.to_string(), text.to_string()) else {
                continue;
            };
            let coordinates: Vec<i32> = key
                .split(',')
                .filter_map(|part| part.parse().ok())
                .collect();
            let (Ok(pos), Some(destination)) =
                (<[i32; 3]>::try_from(coordinates), DimPos::decode(&text))
            else {
                continue;
            };
            map.insert(pos, destination);
        }
        Self(map)
    }
}

/// What a linker click does. `stored` is the position kept on the item.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LinkClick {
    /// Nothing stored: remember the clicked teleporter.
    Store(DimPos),
    /// A teleporter was stored: each now sends to the block above the other, and the item forgets.
    Link {
        /// The earlier teleporter and its new destination.
        first: (DimPos, DimPos),
        /// The clicked teleporter and its new destination.
        second: (DimPos, DimPos),
    },
}

/// `TeleporterLinker.useOn`.
pub fn link_click(stored: Option<DimPos>, clicked: DimPos) -> LinkClick {
    match stored {
        None => LinkClick::Store(clicked),
        Some(first) => {
            let first_destination = clicked.above();
            let second_destination = first.above();
            LinkClick::Link {
                first: (first, first_destination),
                second: (clicked, second_destination),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn light_settings_clamp_to_block_light() {
        assert_eq!(light_level(None, GLOWROSE_LIGHT_DEFAULT), 10);
        assert_eq!(light_level(Some("15"), 10.0), 15);
        assert_eq!(light_level(Some("99"), 10.0), 15);
        assert_eq!(light_level(Some("-3"), 10.0), 0);
        assert_eq!(light_level(Some("junk"), 15.0), 15);
    }

    #[test]
    fn coffee_delay_is_minutes_in_range() {
        assert_eq!(coffee_delay_ticks(None), 12_000);
        assert_eq!(coffee_delay_ticks(Some("0")), 1_200);
        assert_eq!(coffee_delay_ticks(Some("100000")), 1_728_000);
    }

    #[test]
    fn coffee_timer_counts_to_the_delay_then_drops_a_cup() {
        assert_eq!(coffee_step(0, 3), (1, false));
        assert_eq!(coffee_step(2, 3), (3, false));
        assert_eq!(coffee_step(3, 3), (0, true));
        assert_eq!(coffee_step(9, 3), (0, true));
    }

    #[test]
    fn extra_glowrose_ground_matches_upstream_tags() {
        assert!(glowrose_extra_ground("minecraft:netherrack"));
        assert!(glowrose_extra_ground("minecraft:crimson_nylium"));
        assert!(glowrose_extra_ground("minecraft:end_stone"));
        assert!(!glowrose_extra_ground("minecraft:stone"));
    }

    #[test]
    fn bone_meal_offsets_stay_in_the_upstream_box() {
        for a in 0..16 {
            for b in 0..12 {
                let offset = bone_meal_offset([a, b, a + b]);
                assert!((-4..4).contains(&offset[0]));
                assert!((-3..3).contains(&offset[1]));
                assert!((-4..4).contains(&offset[2]));
            }
        }
    }

    #[test]
    fn dim_pos_round_trips_through_text() {
        let place = DimPos {
            dimension: "minecraft:the_nether".into(),
            pos: [-4, 70, 12],
        };
        assert_eq!(DimPos::decode(&place.encode()), Some(place.clone()));
        assert_eq!(DimPos::decode("nonsense"), None);
        assert_eq!(place.above().pos, [-4, 71, 12]);
        assert_eq!(place.centre(), [-3.5, 70.0, 12.5]);
    }

    #[test]
    fn destinations_round_trip_through_nbt() {
        let mut saved = Destinations::default();
        saved.0.insert(
            [1, 2, 3],
            DimPos {
                dimension: "minecraft:overworld".into(),
                pos: [9, 8, 7],
            },
        );
        saved.0.insert(
            [-1, 0, 5],
            DimPos {
                dimension: "minecraft:the_end".into(),
                pos: [0, 64, 0],
            },
        );
        assert_eq!(Destinations::from_nbt(&saved.to_nbt()), saved);
    }

    #[test]
    fn linking_sends_each_teleporter_above_the_other() {
        let a = DimPos {
            dimension: "minecraft:overworld".into(),
            pos: [0, 64, 0],
        };
        let b = DimPos {
            dimension: "minecraft:the_nether".into(),
            pos: [8, 70, 8],
        };
        assert_eq!(link_click(None, a.clone()), LinkClick::Store(a.clone()));
        let LinkClick::Link { first, second } = link_click(Some(a.clone()), b.clone()) else {
            panic!("expected a link");
        };
        assert_eq!(first, (a.clone(), b.above()));
        assert_eq!(second, (b, a.above()));
    }
}
