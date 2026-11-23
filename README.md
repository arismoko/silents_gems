# Silent's Gems for Sunrise

Twenty-one gems with their ores, storage blocks and redstone gem lamps.

A port of [Silent's Gems](https://github.com/SilentChaos512/SilentGems) by SilentChaos512 to [Sunrise](https://playsunrise.net) (Minecraft Java 26.3). The gems, ores and blocks are content; the lamps' redstone rule is a small native part.

## What it does

- 21 gems (ruby, sapphire, opal, alexandrite, black and white diamond and more), each with a storage block and its 9-to-1 recipes.
- 84 ores in stone, deepslate, the Nether and the End, generating in each dimension as the original's defaults set, with silk touch and fortune drops.
- Smelting and blasting a gem ore gives its gem.
- 84 gem lamps: a lamp lights while powered by redstone, and an inverted lamp lights while unpowered.
- A building palette for every gem: bricks, small bricks, tiles, polished, smooth and chiseled stone, and coloured glass (147 blocks), each crafted from eight vanilla blocks around one gem.

## Differences from the original

- The teleporters, glowroses, food, pet summoners, chaos and silver lines and Silent Gear integration are not ported yet.
- A lamp switches at the end of the tick its signal changes, not within the same block update.

## Building

With the Sunrise mod tool (`cargo sunrise-mod`, from the Sunrise SDK) and Rust:

```sh
cargo sunrise-mod build    # compiles against the pinned game, packs target/sunrise-mod/silentgems.smod
cargo sunrise-mod run      # builds it and starts the game with it
cargo test
```

`sunrise-mod.toml` pins the Sunrise Dev build `dev-2026-11-22-7cfdf81` (game commit `7cfdf81463`). Sunrise builds native mods from source on the player's computer, against the exact game build.

## Licence and credits

Silent's Gems is by SilentChaos512 (https://github.com/SilentChaos512/SilentGems). Its textures, models, recipes, loot tables, worldgen and strings are SilentChaos512's and are used here under the **MIT licence**, the original's; `LICENSE` holds the text and its copyright notice, and `LICENSE-NOTICE.md` says what this port carries.

Ported to Minecraft 26.3 by AdLf0208 ([AdLf0208/silents_gems](https://github.com/AdLf0208/silents_gems)), and to Sunrise by arismoko. This port is distributed under the same licence.
