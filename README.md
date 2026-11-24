# Silent's Gems for Sunrise

Twenty-one gems with their ores, building blocks, lamps, glowroses, teleporters, coffee and more.

A port of [Silent's Gems](https://github.com/SilentChaos512/SilentGems) by SilentChaos512 to [Sunrise](https://playsunrise.net) (Minecraft Java 26.3). The gems, ores and blocks are content; the lamps, teleporters, glowroses, summoners and rabbit coffee are native parts.

## What it does

- 21 gems (ruby, sapphire, opal, alexandrite, black and white diamond and more), each with a storage block and its 9-to-1 recipes.
- 84 ores in stone, deepslate, the Nether and the End, generating in each dimension as the original's defaults set, with silk touch and fortune drops.
- Smelting and blasting a gem ore gives its gem.
- 84 gem lamps: a lamp lights while powered by redstone, and an inverted lamp lights while unpowered.
- A building palette for every gem: bricks, small bricks, tiles, polished, smooth and chiseled stone, and coloured glass (147 blocks), each crafted from eight vanilla blocks around one gem.
- Silver and chaos: ores in stone and deepslate, raw silver, ingots, nuggets, rods, chaos essence, storage blocks and the reinforced rods, with the original's recipes.
- 21 glowroses (free-standing and potted) that grow in all three dimensions and glow, with light levels in the mod's settings; sparkling bone meal scatters them over grass.
- Food: potato on a stick, sugar cookie, cup of coffee, meaty and fishy stews (raw and cooked) and the iron potato, with their effects. Rabbits drop a cup of coffee on a timer (settings).
- Summon Kitty and Summon Puppy call a tamed cat or wolf.
- Teleporters, redstone teleporters and the anchor: link two with the teleporter linker (across dimensions too) and click to travel; a redstone teleporter sends everything within five blocks when powered. Gem glass tints a beacon's beam.

## Differences from the original

- Silent Gear integration is not ported.
- A teleporter placed in a mirrored structure keeps its facing: a block can't answer the mirror step yet ([sunrise-org/sunrise#5314](https://github.com/sunrise-org/sunrise/issues/5314)). Placing and rotating turn it as in the original.
- Glowroses and the rabbit coffee read their light and timer from the mod's settings rather than a config file.
- A lamp just placed where it reads a signal switches at the end of that tick, not within the placement; a signal change switches it at once ([sunrise-org/sunrise#5302](https://github.com/sunrise-org/sunrise/issues/5302)).
- A redstone teleporter sends the entities around it at the end of the tick its signal arrives, not within the redstone update ([sunrise-org/sunrise#5303](https://github.com/sunrise-org/sunrise/issues/5303)).

## Building

With the Sunrise mod tool (`cargo sunrise-mod`, from the Sunrise SDK) and Rust:

```sh
cargo sunrise-mod build    # compiles against the pinned game, packs target/sunrise-mod/silentgems.smod
cargo sunrise-mod run      # builds it and starts the game with it
cargo test
```

`sunrise-mod.toml` pins the Sunrise Dev build `dev-2026-11-23-ebd2d22` (game commit `ebd2d226fb`). Sunrise builds native mods from source on the player's computer, against the exact game build.

## Licence and credits

Silent's Gems is by SilentChaos512 (https://github.com/SilentChaos512/SilentGems). Its textures, models, recipes, loot tables, worldgen and strings are SilentChaos512's and are used here under the **MIT licence**, the original's; `LICENSE` holds the text and its copyright notice, and `LICENSE-NOTICE.md` says what this port carries.

Ported to Minecraft 26.3 by AdLf0208 ([AdLf0208/silents_gems](https://github.com/AdLf0208/silents_gems)), and to Sunrise by arismoko. This port is distributed under the same licence.
