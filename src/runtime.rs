//! Silent's Gems — the pure registry and the lamp rule.
//!
//! Ported from Silent's Gems 5.2.0 (`SilentChaos512/SilentGems`, MIT): the 21 gems are
//! `setup/Gems.java:38-165`, the four lamp states are `block/GemLampBlock.java:14-42` and the
//! lamp ids are `Gems.registerLamp` (`:563-576`). This module holds no game type, so the whole
//! rule set runs under `cargo test` without staging the game.
//!
//! Everything else in the package is content: `data/` declares the blocks, loot, recipes and
//! tags, `assets/` carries the models and textures. The one behaviour is the lamp: a redstone
//! signal switches a lamp to the state its own state table names.

/// The upstream `net.minecraft.world.item.Rarity` of a gem item.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rarity {
    /// No colour in the name.
    Common,
    /// Yellow name.
    Uncommon,
    /// Aqua name.
    Rare,
    /// Light purple name.
    Epic,
}

/// One Silent's Gems gem and the values its declarations use.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Gem {
    /// The serialized name used in item, block and tag ids.
    pub name: &'static str,
    /// The English display name from `gem.silentgems.<name>`.
    pub title: &'static str,
    /// The RGB value from the upstream `Gems` enum constant.
    pub color: u32,
    /// The item rarity upstream passes to `Item.Properties.rarity`.
    pub rarity: Rarity,
}

/// How many gems a storage block is made from (`compressionRecipes`).
pub const GEMS_PER_BLOCK: u32 = 9;

/// The 21 gems, in upstream enum order.
pub const GEMS: &[Gem] = &[
    Gem {
        name: "ruby",
        title: "Ruby",
        color: 0xE6_1D_1D,
        rarity: Rarity::Common,
    },
    Gem {
        name: "carnelian",
        title: "Carnelian",
        color: 0xE0_4D_1D,
        rarity: Rarity::Uncommon,
    },
    Gem {
        name: "topaz",
        title: "Topaz",
        color: 0xE6_71_1D,
        rarity: Rarity::Common,
    },
    Gem {
        name: "citrine",
        title: "Citrine",
        color: 0xC7_8B_03,
        rarity: Rarity::Uncommon,
    },
    Gem {
        name: "heliodor",
        title: "Heliodor",
        color: 0xE6_C5_1D,
        rarity: Rarity::Common,
    },
    Gem {
        name: "moldavite",
        title: "Moldavite",
        color: 0xA6_D9_23,
        rarity: Rarity::Uncommon,
    },
    Gem {
        name: "peridot",
        title: "Peridot",
        color: 0x29_DB_18,
        rarity: Rarity::Common,
    },
    Gem {
        name: "turquoise",
        title: "Turquoise",
        color: 0x3D_F4_BD,
        rarity: Rarity::Rare,
    },
    Gem {
        name: "kyanite",
        title: "Kyanite",
        color: 0x41_C4_F3,
        rarity: Rarity::Rare,
    },
    Gem {
        name: "sapphire",
        title: "Sapphire",
        color: 0x1D_60_E5,
        rarity: Rarity::Common,
    },
    Gem {
        name: "iolite",
        title: "Iolite",
        color: 0x75_43_F5,
        rarity: Rarity::Uncommon,
    },
    Gem {
        name: "alexandrite",
        title: "Alexandrite",
        color: 0xAB_37_E5,
        rarity: Rarity::Uncommon,
    },
    Gem {
        name: "ammolite",
        title: "Ammolite",
        color: 0xDB_2B_FF,
        rarity: Rarity::Rare,
    },
    Gem {
        name: "rose_quartz",
        title: "Rose Quartz",
        color: 0xFF_4E_AB,
        rarity: Rarity::Rare,
    },
    Gem {
        name: "black_diamond",
        title: "Black Diamond",
        color: 0x5F_52_4C,
        rarity: Rarity::Epic,
    },
    Gem {
        name: "white_diamond",
        title: "White Diamond",
        color: 0xD5_C1_D2,
        rarity: Rarity::Epic,
    },
    Gem {
        name: "garnet",
        title: "Garnet",
        color: 0x97_00_00,
        rarity: Rarity::Uncommon,
    },
    Gem {
        name: "aquamarine",
        title: "Aquamarine",
        color: 0x6E_E9_F4,
        rarity: Rarity::Uncommon,
    },
    Gem {
        name: "tanzanite",
        title: "Tanzanite",
        color: 0x3D_00_B9,
        rarity: Rarity::Uncommon,
    },
    Gem {
        name: "opal",
        title: "Opal",
        color: 0xDD_FF_E4,
        rarity: Rarity::Rare,
    },
    Gem {
        name: "pearl",
        title: "Pearl",
        color: 0xD3_CB_BF,
        rarity: Rarity::Rare,
    },
];

impl Gem {
    /// The gem with this serialized name, if it is a Silent's Gems gem.
    #[must_use]
    pub fn from_name(name: &str) -> Option<&'static Gem> {
        GEMS.iter().find(|gem| gem.name == name)
    }

    /// The gem's 0-based position in the upstream enum.
    #[must_use]
    pub fn index(self) -> Option<usize> {
        GEMS.iter().position(|gem| *gem == self)
    }

    /// The gem item id, `silentgems:<name>`.
    #[must_use]
    pub fn item_id(self) -> String {
        format!("silentgems:{}", self.name)
    }

    /// The storage block id, `silentgems:<name>_block`.
    #[must_use]
    pub fn block_id(self) -> String {
        format!("silentgems:{}_block", self.name)
    }

    /// The convention item tag the recipes use, `c:gems/<name>`.
    #[must_use]
    pub fn gem_tag(self) -> String {
        format!("c:gems/{}", self.name)
    }

    /// The convention storage-block tag, `c:storage_blocks/<name>`.
    #[must_use]
    pub fn storage_block_tag(self) -> String {
        format!("c:storage_blocks/{}", self.name)
    }

    /// How many gem items one storage block compresses.
    #[must_use]
    pub fn gems_per_block(self) -> u32 {
        GEMS_PER_BLOCK
    }
}

/// The four states of one gem lamp (`GemLampBlock.State`).
///
/// Upstream registers a block per state and names it
/// `<gem>_lamp` + `_inverted` when inverted + `_on` when lit, so a lamp that changes state
/// changes its block id, not a block state. Only the two states a player can place have an item
/// (`State.hasItem` is `lit == inverted`): `Off` and `InvertedOn`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LampState {
    /// Unlit, not inverted: the block an unpowered lamp is placed as.
    Off,
    /// Lit by power, not inverted: no item.
    On,
    /// Lit while unpowered, inverted: the block an inverted lamp is placed as.
    InvertedOn,
    /// Inverted and powered, so unlit: no item.
    InvertedOff,
}

impl LampState {
    /// Every state, in the order upstream declares the enum.
    pub const ALL: [Self; 4] = [Self::Off, Self::On, Self::InvertedOn, Self::InvertedOff];

    /// Whether the lamp emits light (`State.lit`).
    #[must_use]
    pub const fn is_lit(self) -> bool {
        matches!(self, Self::On | Self::InvertedOn)
    }

    /// Whether the lamp is lit while unpowered (`State.inverted`).
    #[must_use]
    pub const fn is_inverted(self) -> bool {
        matches!(self, Self::InvertedOn | Self::InvertedOff)
    }

    /// Whether the state is the one a player can hold (`State.hasItem`).
    #[must_use]
    pub const fn has_item(self) -> bool {
        self.is_lit() == self.is_inverted()
    }

    /// The light level upstream gives the state's block (`state.lit() ? 15 : 0`).
    #[must_use]
    pub const fn light(self) -> u8 {
        if self.is_lit() { 15 } else { 0 }
    }

    /// The state this lamp takes when its neighbour signal is `powered` (`State.withPower`).
    #[must_use]
    pub const fn with_power(self, powered: bool) -> Self {
        if self.is_inverted() {
            if powered {
                Self::InvertedOff
            } else {
                Self::InvertedOn
            }
        } else if powered {
            Self::On
        } else {
            Self::Off
        }
    }

    /// The block id of this state for `gem`, `silentgems:<gem>_lamp[_inverted][_on]`.
    #[must_use]
    pub fn block_name(self, gem: &str) -> String {
        let mut name = format!("silentgems:{gem}_lamp");
        if self.is_inverted() {
            name.push_str("_inverted");
        }
        if self.is_lit() {
            name.push_str("_on");
        }
        name
    }

    /// The item id of this state, or `None` for a state upstream registers without an item.
    #[must_use]
    pub fn item_name(self, gem: &str) -> Option<String> {
        self.has_item().then(|| self.block_name(gem))
    }

    /// The block this state drops: upstream always drops the unpowered form
    /// (`GemsLootTableProvider:69-72`), which is the item-bearing one.
    #[must_use]
    pub fn drop_name(self, gem: &str) -> String {
        self.with_power(false).block_name(gem)
    }
}

/// The gem and state a lamp block id names, or `None` for any other block.
///
/// This is the rule the server half asks on a neighbour change and a block change: a state that
/// does not parse is not this mod's lamp and is left alone.
#[must_use]
pub fn lamp_of(block_id: &str) -> Option<(&'static Gem, LampState)> {
    let name = block_id.strip_prefix("silentgems:")?;
    GEMS.iter().find_map(|gem| {
        let state = match name.strip_prefix(gem.name)? {
            "_lamp" => LampState::Off,
            "_lamp_on" => LampState::On,
            "_lamp_inverted" => LampState::InvertedOff,
            "_lamp_inverted_on" => LampState::InvertedOn,
            _ => return None,
        };
        Some((gem, state))
    })
}

/// Every lamp block id, in gem order and `LampState::ALL` order: 84 in all.
#[must_use]
pub fn lamp_block_ids() -> Vec<String> {
    let mut ids = Vec::with_capacity(GEMS.len() * LampState::ALL.len());
    for gem in GEMS {
        for state in LampState::ALL {
            ids.push(state.block_name(gem.name));
        }
    }
    ids
}

/// Every lamp item id, in gem order: the off and inverted-on states, 42 in all.
#[must_use]
pub fn lamp_item_ids() -> Vec<String> {
    let mut ids = Vec::with_capacity(GEMS.len() * 2);
    for gem in GEMS {
        for state in LampState::ALL {
            if let Some(id) = state.item_name(gem.name) {
                ids.push(id);
            }
        }
    }
    ids
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The exact upstream English names, in `Gems.java` order.
    const NAMES: [&str; 21] = [
        "ruby",
        "carnelian",
        "topaz",
        "citrine",
        "heliodor",
        "moldavite",
        "peridot",
        "turquoise",
        "kyanite",
        "sapphire",
        "iolite",
        "alexandrite",
        "ammolite",
        "rose_quartz",
        "black_diamond",
        "white_diamond",
        "garnet",
        "aquamarine",
        "tanzanite",
        "opal",
        "pearl",
    ];

    /// The exact upstream RGB constants (`Gems.java:39-165`).
    const COLOURS: [(&str, u32); 21] = [
        ("ruby", 0xE6_1D_1D),
        ("carnelian", 0xE0_4D_1D),
        ("topaz", 0xE6_71_1D),
        ("citrine", 0xC7_8B_03),
        ("heliodor", 0xE6_C5_1D),
        ("moldavite", 0xA6_D9_23),
        ("peridot", 0x29_DB_18),
        ("turquoise", 0x3D_F4_BD),
        ("kyanite", 0x41_C4_F3),
        ("sapphire", 0x1D_60_E5),
        ("iolite", 0x75_43_F5),
        ("alexandrite", 0xAB_37_E5),
        ("ammolite", 0xDB_2B_FF),
        ("rose_quartz", 0xFF_4E_AB),
        ("black_diamond", 0x5F_52_4C),
        ("white_diamond", 0xD5_C1_D2),
        ("garnet", 0x97_00_00),
        ("aquamarine", 0x6E_E9_F4),
        ("tanzanite", 0x3D_00_B9),
        ("opal", 0xDD_FF_E4),
        ("pearl", 0xD3_CB_BF),
    ];

    #[test]
    fn registry_has_the_twenty_one_gems_in_upstream_order() {
        assert_eq!(GEMS.len(), 21);
        let names: Vec<&str> = GEMS.iter().map(|gem| gem.name).collect();
        assert_eq!(names, NAMES);
    }

    #[test]
    fn every_gem_has_a_unique_lowercase_name() {
        let mut names: Vec<&str> = GEMS.iter().map(|gem| gem.name).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), 21, "duplicate gem names");
        for gem in GEMS {
            assert_eq!(
                gem.name,
                gem.name.to_lowercase(),
                "{} is not lower case",
                gem.name
            );
        }
    }

    #[test]
    fn index_round_trips_every_gem() {
        assert_eq!(GEMS.len(), 21);
        for (position, gem) in GEMS.iter().enumerate() {
            assert_eq!(
                gem.index(),
                Some(position),
                "{} has the wrong index",
                gem.name
            );
        }
    }

    #[test]
    fn from_name_is_exact_and_rejects_unknown_names() {
        assert_eq!(Gem::from_name("ruby").map(|gem| gem.name), Some("ruby"));
        assert_eq!(
            Gem::from_name("black_diamond").map(|gem| gem.name),
            Some("black_diamond")
        );
        assert_eq!(Gem::from_name("Ruby"), None, "names are lower case");
        assert_eq!(Gem::from_name("ruby_block"), None);
        assert_eq!(Gem::from_name(""), None);
        assert_eq!(Gem::from_name("diamond"), None, "vanilla is not a gem");
    }

    #[test]
    fn colours_match_the_upstream_enum_constants() {
        for (name, color) in COLOURS {
            let gem = Gem::from_name(name).unwrap_or_else(|| panic!("{name} is registered"));
            assert_eq!(gem.color, color, "{name} has the wrong colour");
        }
        let mut colours: Vec<u32> = GEMS.iter().map(|gem| gem.color).collect();
        colours.sort_unstable();
        colours.dedup();
        assert_eq!(colours.len(), 21, "two gems share a colour");
        for gem in GEMS {
            assert!(gem.color <= 0xFF_FF_FF, "{} is not RGB", gem.name);
        }
    }

    #[test]
    fn rarities_match_the_upstream_enum() {
        let count = |rarity: Rarity| GEMS.iter().filter(|gem| gem.rarity == rarity).count();
        assert_eq!(count(Rarity::Common), 5);
        assert_eq!(count(Rarity::Uncommon), 8);
        assert_eq!(count(Rarity::Rare), 6);
        assert_eq!(count(Rarity::Epic), 2);
        let by = |name: &str| Gem::from_name(name).map(|gem| gem.rarity);
        assert_eq!(by("ruby"), Some(Rarity::Common));
        assert_eq!(by("garnet"), Some(Rarity::Uncommon));
        assert_eq!(by("opal"), Some(Rarity::Rare));
        assert_eq!(by("white_diamond"), Some(Rarity::Epic));
    }

    #[test]
    fn titles_match_the_upstream_lang_file() {
        assert_eq!(GEMS.len(), 21);
        for gem in GEMS {
            let key = format!("gem.silentgems.{}", gem.name);
            assert_eq!(
                gem.title,
                expected_title(gem.name),
                "{key} has the wrong English name"
            );
        }
    }

    /// The `gem.silentgems.*` values from `en_us.json`.
    fn expected_title(name: &str) -> &'static str {
        match name {
            "alexandrite" => "Alexandrite",
            "ammolite" => "Ammolite",
            "aquamarine" => "Aquamarine",
            "black_diamond" => "Black Diamond",
            "carnelian" => "Carnelian",
            "citrine" => "Citrine",
            "garnet" => "Garnet",
            "heliodor" => "Heliodor",
            "iolite" => "Iolite",
            "kyanite" => "Kyanite",
            "moldavite" => "Moldavite",
            "opal" => "Opal",
            "pearl" => "Pearl",
            "peridot" => "Peridot",
            "rose_quartz" => "Rose Quartz",
            "ruby" => "Ruby",
            "sapphire" => "Sapphire",
            "tanzanite" => "Tanzanite",
            "topaz" => "Topaz",
            "turquoise" => "Turquoise",
            "white_diamond" => "White Diamond",
            other => panic!("no expected title for {other}"),
        }
    }

    #[test]
    fn item_block_and_tag_ids_are_namespaced_by_gem() {
        let rose = Gem::from_name("rose_quartz").expect("rose_quartz is registered");
        assert_eq!(rose.item_id(), "silentgems:rose_quartz");
        assert_eq!(rose.block_id(), "silentgems:rose_quartz_block");
        assert_eq!(rose.gem_tag(), "c:gems/rose_quartz");
        assert_eq!(rose.storage_block_tag(), "c:storage_blocks/rose_quartz");
        let ruby = Gem::from_name("ruby").expect("ruby is registered");
        assert_eq!(ruby.item_id(), "silentgems:ruby");
        assert_eq!(ruby.block_id(), "silentgems:ruby_block");
        assert_eq!(ruby.gem_tag(), "c:gems/ruby");
        assert_eq!(ruby.storage_block_tag(), "c:storage_blocks/ruby");
    }

    #[test]
    fn every_gem_compresses_nine_to_one_storage_block() {
        assert_eq!(GEMS_PER_BLOCK, 9);
        assert_eq!(GEMS.len(), 21);
        for gem in GEMS {
            assert_eq!(gem.gems_per_block(), 9, "{} has the wrong ratio", gem.name);
        }
    }

    #[test]
    fn every_state_maps_its_ids_as_upstream_names_them() {
        assert_eq!(LampState::Off.block_name("ruby"), "silentgems:ruby_lamp");
        assert_eq!(LampState::On.block_name("ruby"), "silentgems:ruby_lamp_on");
        assert_eq!(
            LampState::InvertedOn.block_name("ruby"),
            "silentgems:ruby_lamp_inverted_on"
        );
        assert_eq!(
            LampState::InvertedOff.block_name("ruby"),
            "silentgems:ruby_lamp_inverted"
        );
        assert_eq!(
            LampState::InvertedOff.block_name("rose_quartz"),
            "silentgems:rose_quartz_lamp_inverted"
        );
    }

    #[test]
    fn only_the_two_placed_states_have_items() {
        let items: Vec<LampState> = LampState::ALL
            .into_iter()
            .filter(|state| state.has_item())
            .collect();
        assert_eq!(items, vec![LampState::Off, LampState::InvertedOn]);
        assert_eq!(
            LampState::Off.item_name("ruby").as_deref(),
            Some("silentgems:ruby_lamp")
        );
        assert_eq!(
            LampState::InvertedOn.item_name("ruby").as_deref(),
            Some("silentgems:ruby_lamp_inverted_on")
        );
        assert_eq!(LampState::On.item_name("ruby"), None);
        assert_eq!(LampState::InvertedOff.item_name("ruby"), None);
    }

    #[test]
    fn a_placed_lamp_takes_the_state_its_signal_names() {
        let placed = |id, powered| {
            lamp_of(id).map(|(gem, state)| state.with_power(powered).block_name(gem.name))
        };
        assert_eq!(
            placed("silentgems:ruby_lamp", true).as_deref(),
            Some("silentgems:ruby_lamp_on")
        );
        assert_eq!(
            placed("silentgems:ruby_lamp", false).as_deref(),
            Some("silentgems:ruby_lamp")
        );
        assert_eq!(
            placed("silentgems:ruby_lamp_inverted_on", true).as_deref(),
            Some("silentgems:ruby_lamp_inverted")
        );
        assert_eq!(placed("minecraft:stone", true), None);
    }

    #[test]
    fn a_signal_switches_each_state_both_ways() {
        assert_eq!(LampState::Off.with_power(true), LampState::On);
        assert_eq!(LampState::On.with_power(false), LampState::Off);
        assert_eq!(LampState::On.with_power(true), LampState::On);
        assert_eq!(LampState::Off.with_power(false), LampState::Off);
        assert_eq!(
            LampState::InvertedOn.with_power(true),
            LampState::InvertedOff
        );
        assert_eq!(
            LampState::InvertedOff.with_power(false),
            LampState::InvertedOn
        );
        assert_eq!(
            LampState::InvertedOn.with_power(false),
            LampState::InvertedOn
        );
        assert_eq!(
            LampState::InvertedOff.with_power(true),
            LampState::InvertedOff
        );
    }

    #[test]
    fn the_lit_states_emit_fifteen_and_the_unlit_ones_none() {
        for state in LampState::ALL {
            let want = if state.is_lit() { 15 } else { 0 };
            assert_eq!(state.light(), want, "{state:?} has the wrong light");
        }
        assert!(LampState::On.is_lit() && LampState::InvertedOn.is_lit());
        assert!(!LampState::Off.is_lit() && !LampState::InvertedOff.is_lit());
    }

    #[test]
    fn every_lamp_state_drops_the_unpowered_item_bearing_form() {
        for gem in GEMS {
            for state in LampState::ALL {
                let dropped = state.drop_name(gem.name);
                let (dropped_gem, dropped_state) =
                    lamp_of(&dropped).unwrap_or_else(|| panic!("{dropped} does not parse"));
                assert_eq!(dropped_gem.name, gem.name);
                assert!(
                    dropped_state.has_item(),
                    "{dropped} is not the item-bearing form"
                );
                assert_eq!(
                    dropped_state.with_power(false),
                    dropped_state,
                    "{dropped} does not drop itself again"
                );
            }
            assert_eq!(
                LampState::On.drop_name(gem.name),
                LampState::Off.block_name(gem.name)
            );
            assert_eq!(
                LampState::InvertedOff.drop_name(gem.name),
                LampState::InvertedOn.block_name(gem.name)
            );
        }
    }

    #[test]
    fn the_package_declares_eighty_four_blocks_and_forty_two_items() {
        let blocks = lamp_block_ids();
        assert_eq!(blocks.len(), 84);
        let unique: std::collections::BTreeSet<&String> = blocks.iter().collect();
        assert_eq!(unique.len(), 84, "two lamp blocks share an id");
        assert!(blocks.contains(&"silentgems:ruby_lamp".to_owned()));
        assert!(blocks.contains(&"silentgems:pearl_lamp_inverted_on".to_owned()));
        let items = lamp_item_ids();
        assert_eq!(items.len(), 42);
        let unique: std::collections::BTreeSet<&String> = items.iter().collect();
        assert_eq!(unique.len(), 42, "two lamp items share an id");
        assert!(items.contains(&"silentgems:ruby_lamp".to_owned()));
        assert!(items.contains(&"silentgems:opal_lamp_inverted_on".to_owned()));
        assert!(!items.contains(&"silentgems:opal_lamp_on".to_owned()));
    }

    #[test]
    fn every_lamp_block_id_parses_back_to_its_gem_and_state() {
        for (index, gem) in GEMS.iter().enumerate() {
            for state in LampState::ALL {
                let id = state.block_name(gem.name);
                let (parsed_gem, parsed_state) =
                    lamp_of(&id).unwrap_or_else(|| panic!("{id} does not parse"));
                assert_eq!(parsed_gem.name, GEMS[index].name);
                assert_eq!(parsed_state, state, "{id} parsed to {parsed_state:?}");
            }
        }
        assert_eq!(lamp_of("silentgems:ruby_block"), None);
        assert_eq!(lamp_of("minecraft:redstone_lamp"), None);
        assert_eq!(lamp_of("silentgems:ruby_lamp_on_extra"), None);
        assert_eq!(lamp_of("ruby_lamp"), None, "the namespace is required");
        assert_eq!(lamp_of(""), None);
    }
}
