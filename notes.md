# Silent's Gems — port notes

Ported from **Silent's Gems 5.2.0** (`SilentChaos512/SilentGems`, branch
`master`, commit `222af28c70a1ced2cf98e63eb597e3827caaece0`,
`gradle.properties:16`) for Minecraft Java **26.3** on the campaign pin
**Horizon `0a875da91dbb0fdcb7ba8c42c11341aa229af51a`** + horizon-mixins
**`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`** (`api = "0.2"`). Slice 3 is the
migration from the retired `.hnmod` container to the flat SDK package, plus the
gem lamps; slices 1-2 were verified at `83ad384f` / re-pinned at `1d63e0a5`
before that, and the old bundle is kept under
`ports/_shared/evidence/silentgems/retired-5.2.0/`.

**Class + tier** (from `ports/ledger.csv`, per `horizon/docs/modpacks.md`):
**content / logic**. Slices 1-2 are **carry-over** (ore block declarations,
loot, tags, the upstream generated worldgen files, textures and lang) plus the
port's own 26.3 adaptation of the parts the schema cannot carry verbatim.
Slice 3 adds the mod's one **logic** family: the gem lamp's redstone rule
(a `BlockPart::on_place` and `neighbor_changed`), whose
table is the pure, unit-tested `src/runtime.rs`.

**Licence: MIT** (`gradle.properties:15`, `LICENSE` Copyright (c) 2014
SilentChaos512). This port is **local-only until the maintainer approves
publishing** (ARCHITECTURE.md §8); its textures, models and lang are the
upstream mod's own MIT assets.

Status: **slices 1-3 implemented, `ported-partial`** — the 21 gem items, the 21
storage blocks, the 84 ores (overworld, deepslate, nether and end) with their
worldgen, and the 84 gem lamps (21 gems × off/on and inverted off/on) with the
redstone switching rule.

## Scope and version

The dispatch asked for Silent's Gems **5.1.3** (`SilentChaos512/SilentGems`,
MIT), slice 1: the gem items and their storage blocks. The upstream `master`
is now the **26.2 port, 5.2.0**; ATM10 ships **1.21.1-5.1.3**. Per the brief,
the 26.2 source is used for the content, and the version difference is recorded
explicitly:

- **This port is version 5.2.0, not 5.1.3.** The mixin bundle
  (`mixins/silentgems/horizon-mixin.toml`), the content manifest
  (`content/horizon-mod.toml`) and the `.hnmod`/`.hmod` all carry `5.2.0`,
  because that is the upstream source they were built from. The registry entry
  must name `5.2.0`; relabelling it `5.1.3` would claim a source it was not
  built from, and ATM10's 1.21.1 jar is not that source.
- `CHANGELOG.md:6` (`26.2-5.2.0`) says only "Ported to NeoForge 26.2, Silent
  Lib 12.0+, and Silent Gear 4.3+"; no gems or blocks changed.
- `CHANGELOG.md:38` says the last five gems (`garnet`, `aquamarine`,
  `tanzanite`, `opal`, `pearl`) were added in 1.21.1-5.1.2.
- Both versions therefore declare the same **21 gems**; the 26.2 source is
  preferred because it is the newer loader target, and no behaviour in this
  slice differs between the two. Everything this port registers exists in
  5.1.3 with the same ids, so a pack on 1.21.1-5.1.3 is covered id-for-id;
  only the version string differs.

Repository evidence, the full gem table with file:line and the exact assets are
in [`../_shared/upstream/silentgems/FINDINGS.md`](../_shared/upstream/silentgems/FINDINGS.md).

| # | id | RGB | Rarity |
|---:|---|---|---|
| 1 | `ruby` | `0xE61D1D` | common |
| 2 | `carnelian` | `0xE04D1D` | uncommon |
| 3 | `topaz` | `0xE6711D` | common |
| 4 | `citrine` | `0xC78B03` | uncommon |
| 5 | `heliodor` | `0xE6C51D` | common |
| 6 | `moldavite` | `0xA6D923` | uncommon |
| 7 | `peridot` | `0x29DB18` | common |
| 8 | `turquoise` | `0x3DF4BD` | rare |
| 9 | `kyanite` | `0x41C4F3` | rare |
| 10 | `sapphire` | `0x1D60E5` | common |
| 11 | `iolite` | `0x7543F5` | uncommon |
| 12 | `alexandrite` | `0xAB37E5` | uncommon |
| 13 | `ammolite` | `0xDB2BFF` | rare |
| 14 | `rose_quartz` | `0xFF4EAB` | rare |
| 15 | `black_diamond` | `0x5F524C` | epic |
| 16 | `white_diamond` | `0xD5C1D2` | epic |
| 17 | `garnet` | `0x970000` | uncommon |
| 18 | `aquamarine` | `0x6EE9F4` | uncommon |
| 19 | `tanzanite` | `0x3D00B9` | uncommon |
| 20 | `opal` | `0xDDFFE4` | rare |
| 21 | `pearl` | `0xD3CBBF` | rare |

The enum is `src/main/java/net/silentchaos512/gems/setup/Gems.java:38-165`;
the ids are the lower-cased enum names (`:245-247`), the RGB is the constructor
value (`:249-251`) and the rarity is `:257-259`. The English names are
`src/main/resources/assets/silentgems/lang/en_us.json:29-49`.

## Behaviour

Each gem has:

- `silentgems:<name>` — a plain item with the enum's rarity, no use/tick/
  component behaviour (`item/GemItem.java:8-21`; registration
  `setup/Gems.java:555-561`). Its name is `gem.silentgems.<name>`
  (`en_us.json:29-49`; upstream wraps it in `item.silentgems.gem = %s` at
  `en_us.json:55`).
- `silentgems:<name>_block` — a plain `Block` with upstream's
  `.strength(4, 30).sound(SoundType.METAL)` and **no**
  `requiresCorrectToolForDrops()` (`setup/Gems.java:436-443`,
  `block/GemBlock.java:8-31`). It drops itself
  (`data/GemsLootTableProvider.java:61`) and is named
  `block.silentgems.<name>_block` ("Block of <title>", upstream
  `block.silentgems.gem_block = Block of %s`, `en_us.json:8`).
- a shaped 3×3 recipe of `#c:gems/<name>` → one block, and a shapeless recipe
  of one block → nine gems (`data/recipe/GemsRecipeProvider.java:35-40`; the
  generated `recipe/<name>_block.json` and `<name>_from_block.json`).
- block tags `c:storage_blocks/<name>` and `c:storage_blocks`, and
  `minecraft:mineable/pickaxe`; item tags `c:gems/<name>`, `c:gems`,
  `silentgems:gems` and `minecraft:beacon_payment_items`
  (`data/tags/GemsBlockTagsProvider.java:33,40,51-54`;
  `data/tags/GemsItemTagsProvider.java:44,47-48,51`).
- one creative tab (`setup/GemsCreativeTabs.java:13-21`), title
  `itemGroup.silentgems` (`en_us.json:80`), icon ruby, listing every registered
  item. The slice lists its 42 items (21 gems + 21 blocks).

Assets are the upstream files, byte for byte:
`textures/block/<name>_block.png` and `textures/item/<name>.png`; the generated
`cube_all` model/blockstate and flat item model are carried as 26.3
`blockstates/`, `models/block/`, `models/item/` and `items/` files. Nothing
was invented or recoloured.

## Ores and worldgen (slice 2)

Upstream registers four ore variants for every gem (`Gems.registerBlocks`):

| Variant | id | upstream `strength` | generation (from `Gems`'s `OreConfigDefaults`) |
|---|---|---|---|
| overworld | `silentgems:<gem>_ore` | 3 | 11 gems (ruby, topaz, peridot, sapphire, heliodor, turquoise, iolite, alexandrite, garnet, aquamarine, opal) |
| deepslate | `silentgems:deepslate_<gem>_ore` | 4.5 / 3 | the deepslate target of the overworld feature |
| nether | `silentgems:nether_<gem>_ore` | 4 | 8 gems (carnelian, citrine, moldavite, iolite, alexandrite, black_diamond, tanzanite, pearl); pearl 0..55 |
| end | `silentgems:end_<gem>_ore` | 6 | 9 gems (heliodor, turquoise, kyanite, ammolite, rose_quartz, white_diamond, garnet, opal, pearl) |

- Every ore is `requiresCorrectToolForDrops`; the overworld/deepslate ores are
  in `minecraft:needs_iron_tool`, the nether/end ones in
  `minecraft:needs_diamond_tool` (upstream puts the end ores in NeoForge's
  `needs_netherite_tool`, which 26.3 does not have — see deviations).
- Each ore drops its **gem** (1) with a pickaxe of the right tier; silk-touch
  self-drop and the fortune `ore_drops` formula are upstream's table, but the
  pin's loot engine has no `match_tool`/`apply_bonus`, so the port writes the
  gem drop directly (see deviations).
- Tag sets per gem: `silentgems:ores/<gem>` (block + item), `c:ores/<gem>`
  (block + item, `#silentgems:ores/<gem>`), `c:ores` (both), plus
  `minecraft:mineable/pickaxe`.
- Worldgen: `data/silentgems/worldgen/configured_feature/<dimension>/<gem>_ore.json`
  and `placed_feature/<dimension>/<gem>_ore.json` are upstream's generated
  files, carried over for the dimensions its config enables (28 features:
  11 overworld + 8 nether + 9 end). Three biome modifiers
  (`data/silentgems/neoforge/biome_modifier/{overworld,the_nether,the_end}_features.json`)
  add them to `#minecraft:is_overworld` / `#minecraft:is_nether` /
  `#minecraft:is_end` in `underground_ores`.
  **At pin `0a875da` these files are inert**: the content overlay's
  `worldgen/features.json` is left as a link to the vanilla bundle, so no configured feature a
  package ships reaches the `minecraft:worldgen/feature` registry, and the merged placed features
  are then skipped with `unknown feature reference`. Measured in game, and reproduced by a minimal
  one-feature probe package, on 2026-11-21 —
  [`../_shared/evidence/silentgems-e2e/worldgen-gap.md`](../_shared/evidence/silentgems-e2e/worldgen-gap.md).
  The data is still carried and the e2e asserts the gap (it fails when the pin starts merging).
- The deepslate variant is the second target of each overworld feature (the
  per-dimension file already names both states), so a gem whose overworld
  config is disabled has no deepslate generation either, as upstream.

## Content adaptations to 26.3

1. **Item declarations.** 1.21.1 registered items in code; a 26.3 content mod
   declares each gem with `data/silentgems/horizon/item/<name>.json`
   (template `minecraft:diamond`) and each block item through the block
   declaration's `"item": true`. The rarity moves to the declaration's
   `rarity` field.
2. **Block template.** The storage block copies `minecraft:iron_block` (a
   full metal-sounding cube) with upstream's hardness 4, explosion resistance
   30 and `requires_tool: false`. The 26.3 schema has no `noOcclusion`
   field; the block is a full cube, so none is needed.
3. **Item names.** Upstream formats the shared
   `item.silentgems.gem = %s` / `block.silentgems.gem_block = Block of %s`
   templates at runtime. A content mod resolves `item.<ns>.<path>` and
   `block.<ns>.<path>` per id (`horizon-client/src/mod_content.rs:282`), so the
   port writes the expanded strings for all 21 gems. The rendered text is
   identical.
4. **Recipes** are the 26.3 `"#tag"`/string ingredient and `{id, count}` result
   form; the 1.21 `{"tag":...}` object is not parsed at the pin.
5. **Loot tables** put `{"type":"minecraft:survives_explosion"}` on the pool
   and carry `random_sequence`, the shape the 26.3 loader reads.
6. **Convention tags.** 26.3 ships no `c:` tags, so the port supplies
   `c:gems/<gem>`, `c:gems`, `c:storage_blocks/<gem>` and
   `c:storage_blocks` (block and item as appropriate). The recipes match
   `#c:gems/<gem>`, so the item tag is exercised by the e2e's crafting.
7. **Creative tab.** One content tab replaces upstream's single code tab; it
   lists the slice only. Upstream lists every item in registration order.
8. **Textures/models/item definitions.** The six files per item/block are the
   26.3 layout: `assets/silentgems/blockstates/<block>.json`,
   `models/block/<block>.json`, `models/item/<gem>.json`,
   `items/<gem>.json`, `items/<block>.json`, plus the two PNGs.
9. **Ore declarations.** Each ore copies a vanilla ore template so it gets
   the harvest tier and the ore's own behaviour rows, with upstream's hardness,
   explosion resistance, `requires_tool: true` and a stone-family sound:
   overworld `minecraft:lapis_ore` (3/3, stone), deepslate
   `minecraft:deepslate_lapis_ore` (4.5/3, deepslate), nether
   `minecraft:nether_quartz_ore` (4/4, nether ore), end `minecraft:lapis_ore`
   with 6/6 and stone sound. The template's experience (lapis: 2..5) is the
   port's XP; upstream's `GemOreBlock` used `UniformInt.of(1, 5)` for all four,
   so only the low end differs (recorded).
10. **Ore loot.** Upstream's table is silk-touch → self, otherwise one gem with
    fortune `ore_drops` and `explosion_decay`. The pin reads `condition`
    (singular) and has no `match_tool`, `apply_bonus` or `explosion_decay`, so
    the port writes one gem per ore: the drop is correct, silk touch and
    fortune are not (recorded).
11. **Worldgen file shape.** Two rewrites make the 26.2 datagen readable by the
    pin: (a) block states become 26.3's string form (the 26.2 datagen writes
    `{"Name": "..."}`, which the pin's codec refuses); (b) the data-pack
    `{"type": ..., "config": {...}}` wrapper is flattened to `type` plus the
    config's fields, which is how the pin's feature bundle
    (`worldgen/features.json`) stores vanilla's features — the nested form
    fails at load with `invalid world generation field: targets`.
12. **Nether/end replaceable tags.** Upstream's features target
    `c:netherracks` and `c:end_stones`, NeoForge convention tags 26.3 ships
    neither of; the port supplies both (`minecraft:netherrack`,
    `minecraft:end_stone`), as it already does for `c:gems`.
13. **End ore tier.** Upstream's `neoforge/tags/block/needs_netherite_tool.json`
    has no 26.3 counterpart; the end ores join `minecraft:needs_diamond_tool`
    (the nether ores' tier) instead. A diamond pickaxe mines them.

Slices 1-3 still exclude glowroses/plants, tools/armour, glass,
bricks/tiles/stone variants, teleporters, the silver/chaos material line, food,
GeckoLib-style assets and Silent Gear integration.

The generator is [`tools/gen_content.py`](tools/gen_content.py) for slices 1-2
and [`tools/gen_lamps.py`](tools/gen_lamps.py) for slice 3; both require the
upstream checkout (or the pack jar for a texture) and fail on a missing file.

## Slice 3 — the gem lamps (2026-11-21)

**Gap line this slice comes from** (§ “Gaps and deviations”, item 1): *“Terrain/glowrose plants,
tool/armour sets, **lamps**, glass, bricks/tiles/stone variants, teleporters, … are deferred.”*
Lamps are the only deferred family that is a **logic** rule with a hook the pin expresses
(a block's redstone neighbour → a block write), so they fit the tier without inventing a shape.

### Upstream behaviour (`GemLampBlock`, `setup/Gems.java:563-576`)

One block **per gem and state**, not one block with a `lit` state:

| State | `lit` | `inverted` | block id | item | light |
|---|---|---|---|---|---|
| `OFF` | false | false | `<gem>_lamp` | yes | 0 |
| `ON` | true | false | `<gem>_lamp_on` | no | 15 |
| `INVERTED_ON` | true | true | `<gem>_lamp_inverted_on` | yes | 15 |
| `INVERTED_OFF` | false | true | `<gem>_lamp_inverted` | no | 0 |

`hasItem` is `lit == inverted`, so the two **placed** lamps are `Ruby Lamp`
(`<gem>_lamp`) and `Inverted Ruby Lamp` (`<gem>_lamp_inverted_on`). Strength is
`0.3 / 15`, the sound is stone's default, and the lamp is **not** in
`minecraft:mineable/pickaxe` (`GemsBlockTagsProvider` tags only the ores and the storage blocks);
`requires_tool` is false, so a lamp breaks by hand.

- **The rule** (`block/GemLampBlock.java:44-66`): on place, on a neighbour change and on a scheduled
  tick, read `hasNeighborSignal(pos)` and switch the block to `state.withPower(powered)`
  (`update_clients` only). A lamp already in the target state is not rewritten.
- **Loot** (`GemsLootTableProvider:69-72`): every state drops the **unpowered, item-bearing** form —
  `ruby_lamp_on` drops `ruby_lamp`, `opal_lamp_inverted` drops `opal_lamp_inverted_on`.
- **Recipes** (`GemsRecipeProvider:105-117`): shaped `rgr / gog / rgr` (4 redstone dust, 4 glowstone
  dust, one gem) → `<gem>_lamp`; shapeless `<gem>_lamp` + `minecraft:redstone_torch` →
  `<gem>_lamp_inverted_on`. Both were read back from the pack jar's generated files
  (`silentgems-1.21.1-neoforge-5.1.3.jar`) so the shapes are not a guess.
- **Names** (`en_us.json:12-13`): shared templates `%s Lamp` / `Inverted %s Lamp`, expanded per id for
  the same reason slice 1's gem names are (a content item resolves `item.<ns>.<path>` /
  `block.<ns>.<path>`).

### What the port carries

84 block declarations (`data/silentgems/horizon/block/`), 84 blockstates, 84 cube_all models,
42 `items/` definitions (the two item-bearing states), 84 loot tables, 42 recipes, the 42 upstream
lamp PNGs and 42 lang keys; the creative tab now lists **168** entries. The two convention dust tags
the recipe names (`#c:dusts/redstone`, `#c:dusts/glowstone`) are supplied in `data/c/tags/item/dusts/`,
as the port already does for `c:gems` — 26.3 ships no `c:` tags. No ingredient is a list of
alternatives: each key is one `#tag`, the shape 26.3 accepts.

The template for all four states is `minecraft:stone` (a full cube with no states) with
`hardness 0.3`, `explosion_resistance 15`, `requires_tool: false`, stone sounds and the state's
`light` (0 or 15): using `minecraft:redstone_lamp` would have brought its own `lit` state and its own
vanilla switching behaviour on top of the port's rule.

### The code

`src/runtime.rs` (pure, 16 tests) holds the gem table from slices 1-2 plus the lamp rule:
`LampState { Off, On, InvertedOn, InvertedOff }`, `with_power`, `is_lit`, `is_inverted`, `has_item`,
`light`, `block_name` / `item_name` / `drop_name`, `lamp_of` (id → gem + state) and the 84/42 id
tables. `src/support.rs` is the glue: a `Lamp` `BlockPart` whose `on_place` (any placement: player,
`/setblock`, `/fill`, structure, piston) and `neighbor_changed` both read `has_neighbor_signal`, compute
`with_power` and write the target with `BlockUpdateFlags::CLIENTS` in the call, as upstream's
`UPDATE_CLIENTS` does; a lamp already in the target state is left untouched.

**TDD red → green** (`ports/_shared/evidence/silentgems/sdk-test-red.log`): the lamp tests were written
first against a stub rule and the suite reported **10 passed; 6 failed**, with
`panicked at src/runtime.rs:302: not yet implemented: the lamp id table` and
`panicked at src/runtime.rs:265: the lamp state table`. The red run also caught two transcription
errors in *my own expected values* before any glue existed (the inverted item is
`<gem>_lamp_inverted_on`, not `<gem>_lamp_inverted`, which the jar's `blockstates/` confirms), and one
real bug: `lamp_of` had mapped `_lamp_inverted` to the lit state, which would have made an inverted
lamp flip on every tick. Implementing the table turned the suite green at **16 passed; 0 failed**.

### Coverage (parity rule)

Family keys are the ones `ports/tools/check-coverage.py` inventories from the delivered package
and the pack jar.

| Family | Status | Evidence |
|---|---|---|
| `block` | tested | e2e: 21 storage states + 42 resting lamp states bind; 42 hand-set `_on`/`_inverted` lamps revert to the unpowered form; the block items place |
| `item` | tested | e2e: 21 gems, 84 ores and 42 lamp items give; the two item-less lamp states are refused |
| `loot_table` | tested | e2e breaks: storage block self-drop, ore → gem, `ruby_lamp_on` → `ruby_lamp`, `opal_lamp_inverted_on` → itself, vanilla `redstone_lamp` → itself |
| `recipe:minecraft:crafting_shaped` | tested | `craft_check.py` 9 gems → block; `craft_lamps.py` `rgr/gog/rgr` → lamp (ingredients consumed) |
| `recipe:minecraft:crafting_shapeless` | tested | `craft_check.py` block → 9 gems; `craft_lamps.py` lamp + torch → inverted lamp |
| `recipe:minecraft:smelting` | not ported | the jar's per-gem `ores/<gem>` → gem furnace recipe (200 ticks, 1.0 xp) is not carried yet — the ore slice's remaining gap |
| `recipe:minecraft:blasting` | not ported | the jar's blasting form (100 ticks) of the same recipe; same gap |
| `tag:block` | tested | e2e: `ruby_block` in `#minecraft:mineable/pickaxe`, `#c:storage_blocks`, `#c:storage_blocks/ruby`; `ruby_ore` in `#silentgems:ores/ruby`, `#c:ores/ruby`, `#minecraft:needs_iron_tool` |
| `tag:item` | tested | the recipes match `#c:gems/<gem>`, `#c:dusts/redstone` and `#c:dusts/glowstone` (a wrong ingredient leaves the result empty) |
| `tag:damage_type` | not ported | the jar's `incorrect_for_<gem>_tools` tags belong to the deferred tool/armour layer |
| `tag:entity_type` | not ported | the jar's pet/summoner tags belong to the deferred entity layer |
| `data:silentgear_materials` | not ported | Silent Gear integration is deferred (23 jar files) |
| `data:silentgear_traits` | not ported | Silent Gear integration is deferred (14 jar files) |
| `worldgen:configured_feature` | blocked(configured-feature-not-registered) | measured 2026-11-21: `/place feature silentgems:overworld/ruby_ore` answers `Can't find element … in registry 'minecraft:worldgen/feature'` and the overlay's `worldgen/features.json` is linked unchanged (no mod entry). A minimal one-feature probe package fails the same way, so it is the pin, not this port |
| `worldgen:placed_feature` | blocked(configured-feature-not-registered) | the 28 placed features **are** merged into `worldgen/placed_features.json`, and the worldgen loader then logs `unknown feature reference silentgems:overworld/…` and skips them |
| `data:neoforge` | blocked(configured-feature-not-registered) | the three biome modifiers (`overworld`, `the_nether`, `the_end`) are read and applied, but every feature they add is skipped for the reason above |

The e2e asserts this gap rather than hiding it: the worldgen phase requires the
`Can't find element` reply and that no `silentgems:ruby_ore` was placed, so the phase **fails**
the day the pin merges mod features (the signal to restore the `/place feature` assertions and
move these rows to `tested`). It also prints the loader's own `unknown feature reference` lines
into the log.

Skills: superpowers/using-superpowers,brainstorming,writing-plans,test-driven-development,systematic-debugging,verification-before-completion; BeautifulRust/rust-best-practices

## Verification

Everything below ran on 2026-11-21 against the delivered bytes, through
`ports/tools/build-slot.sh` (`BUILD_LOCAL=1`, because the runner does not `cd` into `--port` and the
remote box has no copy of this port). Evidence: `ports/_shared/evidence/silentgems/` (ladder) and
`ports/_shared/evidence/silentgems-e2e/` (in-game). Every ladder log names the package's size and
SHA-256 in its first line.

### Runtime and glue (SDK crate, `cargo` alone)

| Check | Command | Result |
|---|---|---|
| fmt | `cargo fmt --check` | `FMT_EXIT:0` (`sdk-fmt.log`) |
| clippy | `cargo clippy --all-targets -- -D warnings` | exit 0, **0 warnings** (`sdk-clippy.log`) |
| tests | `cargo test` | **16 passed; 0 failed** (`sdk-test.log`) |
| red first | the same, before the rule existed | **10 passed; 6 failed** (`sdk-test-red.log`) |

### The package (delivered at the port root, not in `target/`)

| Check | Command | Result |
|---|---|---|
| build 1 | `rm -f Cargo.lock && cargo horizon-mod build` | `target/horizon-mod/silentgems.hmod` (`sdk-build.log`) |
| build 2 | the same again, `rm -f Cargo.lock` first | **byte-identical** (`cmp`), same SHA-256 (`sdk-build-second.log`) |
| package check | `cargo horizon-mod check silentgems.hmod` | `silentgems 5.2.1 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)`, exit 0 (`sdk-check.log`) |
| fits the pin | `cargo horizon-mod fits silentgems.hmod --build 0a875da` | `0a875da  made for`, exit 0 (`sdk-fits.log`) |
| fits another build | `cargo horizon-mod fits silentgems.hmod --build 0b14f8a` | `0b14f8a  fits`, exit 0 (`sdk-fits-other.log`) |
| packed `Cargo.lock` | `unzip -l` | **none** (`sdk-entries.log`) |

Delivered [`silentgems.hmod`](silentgems.hmod): **464,271 bytes**, SHA-256
`c500c8212d540cf2642f633b3fcc7a37ddc838ad8389b2a0aae30fcd691e4c42`, **1,446 entries**
(`sdk-entries.log`, `sdk-sha256.txt`): 189 block declarations, 189 loot tables, 189 blockstates,
189 block models, 168 item definitions, 84 recipes, 147 block textures, 21 item textures, 21 item
models, the tags, the worldgen files, the creative tab, `lang/en_us.json`, `LICENSE`,
`Cargo.toml`, `uses.json` and `src/{lib,runtime,support}.rs`. The version inside the manifest, the
`horizon-mod.toml` and `Cargo.toml` is **5.2.1**.

### In-game (launcher-built game at the pin; `tools/e2e.sh`)

The package is native, so the game is built with it by the launcher and the control leg is the same
Nightly **without** it:

```sh
ports/tools/launcher-game.sh build silentgems          # "Compiling silentgems (1 of 1)"
eval "$(ports/tools/launcher-game.sh env silentgems)"
ports/tools/build-slot.sh run --job W1-silentgems-e2e --port ports/silentgems \
    --step e2e -- bash ports/silentgems/tools/e2e.sh
```

| Run | Result |
|---|---|
| default | **PASS + `EXIT:0`** (`e2e-launcher.log`): 21 gems, 21 storage blocks and 84 ores still give; 21/21 storage states bind; the tab lists the gems, blocks and both lamp items of every gem; **42/42 resting lamp states bind** and **42/42 hand-set on/inverted blocks become their unpowered form**; 42/42 lamp items give while `ruby_lamp_on` and `ruby_lamp_inverted` (no items) are refused; `ruby_lamp` (light 0) → `ruby_lamp_on` (light `f`) beside a redstone block → back to `ruby_lamp` (0) when it is removed; `opal_lamp_inverted_on` (light `f`) → `opal_lamp_inverted` (0) while powered → back; a `topaz_lamp` placed beside a signal reads `topaz_lamp_on`; vanilla `minecraft:redstone_lamp` goes `lit=true`/`lit=false` under the same signal; `ruby_lamp_on` drops one `ruby_lamp`, `opal_lamp_inverted_on` drops one of itself, vanilla drops itself; the shaped and shapeless lamp recipes craft and consume their ingredients with both wrong-ingredient controls empty; the 9↔1 compression recipes still craft for ruby/opal/sapphire with their stick controls empty; the block tags match; the carried worldgen is inert at this pin and the gap is asserted; bogus ids and the deferred `silentgems:ruby_teleporter` are refused |
| control (`CONTROL=1`, `GAME=$CONTROL_GAME`) | **FAIL + `EXIT:1`, as required** (`e2e-control.log`): the unmodded Nightly reads `minecraft:air` where the package places `silentgems:ruby_lamp` — the modded run is not vacuous |
| negative (`NEGATIVE=1`) | **FAIL + `EXIT:1`, as required** (`e2e-negative.log`): the run asserts that an unpowered lamp is `_on`; it read `silentgems:ruby_lamp_on` under power, so the assertion failed as intended — the switch check can fail |
| selftest (`SELFTEST=1`) | **FAIL + `EXIT:1`, as required** (`e2e-selftest.log`): the run asserts a lit lamp emits 0; the read-back was `f`, so it failed as intended — the light read-back is live |

`python3 ports/tools/check-e2e-logs.py silentgems-e2e --strict` → exit 0:
`logs=4 ok=4 missing-exit=0 bad-exit=0`.

The e2e's first default run found a **real behaviour** the plan had wrong: a hand-placed `_on` block
with no signal is reverted by the rule (correctly), so the state-bind phase now binds the two resting
states and asserts the other two revert — and the drop phase holds the ruby lamp powered with a
redstone block so the break really happens on `ruby_lamp_on` (the reply's `block_before` is asserted).

### Disk

The staged cargo target stayed in the slot cache and the lease removed the port's cache symlinks. The
port keeps its sources, `assets/`, `data/`, the two ladder logs' evidence (moved to
`ports/_shared/evidence/silentgems/`) and the delivered `silentgems.hmod`; the retired 5.2.0 bundle
(its `mixins/`, `.hnmod`, old `content/` package and tools) is kept under
`ports/_shared/evidence/silentgems/retired-5.2.0/`, outside the package tree, because
`cargo horizon-mod` refuses a `.rs` file anywhere under the port.

## Gaps and deviations

1. **Slices 1-3.** Terrain/glowrose plants, tool/armour sets, **glass**,
   bricks/tiles/stone variants, teleporters, the silver/chaos material line
   (including `silver_ore` and `chaos_ore`), food, GeckoLib-style assets and
   Silent Gear integration are still deferred. The deferred ids are not
   registered, so `silentgems:ruby_teleporter` is refused (the e2e asserts it).
   The lamps left this list in slice 3.
2. **Lamp blocks are 84 registrations, not one block with a `lit` state.**
   That is upstream's own shape (`Gems.registerLamp`) and the reason a lamp
   drop is a *different block id*: the loot table names the unpowered,
   item-bearing form, as upstream's does. Consolidating to a state property
   would have been smaller but would have changed four public block ids per
   gem.
3. **Mod worldgen is inert at `0a875da`.** The 28 configured features a content or native package
   ships are not merged into `worldgen/features.json`; the 28 placed features are, and the loader
   then skips them (`unknown feature reference`). A minimal one-feature probe package fails the
   same way, so the port is not at fault. `/place feature` proved the same files at `1d63e0a5`;
   the e2e now asserts the gap instead of hiding it. Recorded as
   `blocked(configured-feature-not-registered)` in the coverage table.
4. **No furnace recipes for the ores.** The jar ships a `smelting` and a `blasting` recipe per gem
   (`ores/<gem>` → gem, 1.0 xp, 200/100 ticks); slices 1-2 never carried them. They are the next
   content slice (see the coverage table's two `recipe:minecraft:*` rows).
5. **Ore loot subset.** The pin has no `match_tool`, `apply_bonus` or
   `explosion_decay`, so an ore always drops one gem: silk touch returns the
   gem instead of the ore block, and fortune does not multiply. Recorded
   rather than emulated (a hook-side loot rewrite would be a code slice).
6. **Ore experience** comes from the template (lapis: 2-5); upstream's ore used
   a fixed 1-5 provider for every variant.
7. **`%s` name templates expanded.** Rendered names match upstream, but the
   exact translation keys differ (`item.silentgems.ruby` vs the shared
   `item.silentgems.gem`, and `block.silentgems.ruby_lamp` vs
   `block.silentgems.gem_lamp`), because a content block/item resolves a per-id
   key. A resource pack that overrides the shared keys will not rename the
   port's gems or lamps; it must override the per-id keys.
8. **Creative tab placement and contents.** Upstream's code tab lists every
   registered item in registration order; the port's tab lists the 168 items of
   these three slices, with the lamps last. Order is cosmetic.
9. **Rarity is the only gem item effect.** There is no tooltip showing the
   harvest tier or gem colour; upstream's tooltip code is part of the deferred
   gear layer.
10. **No `minecraft:beacon_payment_items` gameplay test.** The tag is written
   (26.3 has the vanilla tag), but the e2e does not build a beacon; the recipe
   test proves the `c:gems` item tag instead. The block tags are checked
   in-game with `execute if block`.
11. **`horizon-mod check` does not parse recipes/loot/worldgen.** It validates the package; the
    recipes were proved by crafting, the loot by mining, and the worldgen is the gap above (it
    was proved by `/place feature` at `1d63e0a5`).
12. **Biome modifiers are not exercised by a chunk-generation test.** They are read and applied,
    but with the configured features inert at this pin (gap 3) there is nothing to generate; a
    natural-generation scan would in any case need a fresh world and a long run. The e2e asserts
    that the feature cannot be placed.
13. **Textures are the upstream MIT files** and the port is local-only until the
    maintainer approves publishing (ARCHITECTURE.md §8). MIT allows derivatives,
    but approval is a separate gate.

## Files

```
horizon-mod.toml        id/version 5.2.1/api 0.2 + [native] pin (0a875da / e59da90)
Cargo.toml              edition 2024, the SDK's horizon-mod-api from .horizon/sdk/
src/lib.rs              the one impl Hooks: register and the feature ticks
src/runtime.rs          the gem table + the lamp state machine, 16 tests, no mod_api
src/support.rs          the Lamp BlockPart (on_place, neighbor_changed)
silentgems.hmod         delivered 1,446-entry package (content + code)
assets/silentgems/      189 blockstates, 189 block models, 21 item models,
                        168 item definitions, 147 + 21 textures, lang/en_us.json
data/silentgems/horizon/189 block declarations, 21 item declarations, 1 tab
data/silentgems/loot_table/ blocks/ 189 tables
data/silentgems/recipe/ 42 compression + 42 lamp recipes
data/minecraft/tags/    mineable/pickaxe, beacon_payment_items
data/c/tags/            gems, storage_blocks, ores, dusts/redstone, dusts/glowstone
data/silentgems/worldgen/, neoforge/biome_modifier/   the 84 ores' generation
tools/gen_content.py    reproduces slices 1-2 from upstream
tools/gen_lamps.py      reproduces slice 3 from upstream (--check for staleness)
tools/craft_check.py    the 9<->1 recipe checks
tools/craft_lamps.py    the lamp recipe checks
tools/e2e.sh            default/control/negative/selftest runs
notes.md, VALIDATION.md
```

The retired bundle's files (`.hnmod`, `mixins/`, old `content/`, `horizon.py`, `package.py`,
`e2e-legs.sh`, `hmod-load.sh`, `compatibility.json`) are under
`ports/_shared/evidence/silentgems/retired-5.2.0/`; they are history, not a package.

## Skills used

- Superpowers: `using-superpowers` (boot), `brainstorming` (choosing the lamp family from the gap
  list: the only deferred family with a logic-tier hook the pin expresses), `writing-plans` (the
  slice's plan), `test-driven-development` (the lamp rule written as failing tests first — the
  6-failure red run is `sdk-test-red.log`), `systematic-debugging` (the first default e2e failed on
  a hand-placed `_on` lamp, which the rule had correctly reverted; the phase now binds the resting
  states and asserts the revert, and the drop phase holds the lamp powered), and
  `verification-before-completion` (every claim re-run and logged under
  `ports/_shared/evidence/silentgems{,-e2e}/`).
- Beautiful Rust: `ports/_shared/skills/BeautifulRust/skills/rust-best-practices/SKILL.md`;
  `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` are clean; no
  `allow`/`expect` was needed. The runtime holds no `mod_api` type, so its tests run without the
  game; the glue keeps one job (queue, parse, write) and the rule stays in `runtime`.
