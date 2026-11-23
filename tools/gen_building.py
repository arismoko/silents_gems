#!/usr/bin/env python3
"""Generate Silent's Gems' building blocks from the upstream checkout.

For each of the 21 gems, the seven blocks `Gems.registerBlocks` adds after the storage block:
bricks, tiles, small bricks, polished, smooth and chiseled stone (`.strength(2f, 8f)`, stone
sounds, no tool needed) and glass (`.strength(1f, 5f)`, glass sounds, see-through). Each gets its
block declaration, the upstream blockstate, model, item definition and texture, its self-drop loot
table, its crafting recipe (eight of a vanilla block around one gem makes twelve), its English
name, and a place in the creative tab; the bricks join `minecraft:mineable/pickaxe` as upstream
tags them.

Usage:
    python3 tools/gen_building.py --upstream <SilentGems checkout>
"""

from __future__ import annotations

import argparse
import json
import shutil
from pathlib import Path

from gen_content import GEMS

ROOT = Path(__file__).resolve().parent.parent

# (block id pattern, upstream lang key)
KINDS = [
    ("{}_bricks", "gem_bricks"),
    ("{}_tiles", "gem_tiles"),
    ("{}_small_bricks", "gem_small_bricks"),
    ("polished_{}", "polished_gem_stone"),
    ("smooth_{}", "smooth_gem_stone"),
    ("chiseled_{}", "chiseled_gem_stone"),
    ("{}_glass", "gem_glass"),
]

STONE = {"hardness": 2.0, "explosion_resistance": 8.0, "requires_tool": False, "sounds": "minecraft:stone"}
GLASS = {"hardness": 1.0, "explosion_resistance": 5.0, "requires_tool": False, "sounds": "minecraft:glass"}


def write(path: Path, value) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2) + "\n")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--upstream", type=Path, required=True)
    args = parser.parse_args()
    gen = args.upstream / "src/generated/resources"
    main_assets = args.upstream / "src/main/resources/assets/silentgems"
    upstream_lang = json.loads((main_assets / "lang/en_us.json").read_text())
    assets = ROOT / "assets/silentgems"
    data = ROOT / "data"

    lang_path = assets / "lang/en_us.json"
    lang = json.loads(lang_path.read_text())
    tab_path = data / "silentgems/sunrise/creative_tab/tab.json"
    tab = json.loads(tab_path.read_text())
    pickaxe_path = data / "minecraft/tags/block/mineable/pickaxe.json"
    pickaxe = json.loads(pickaxe_path.read_text())

    for pattern, lang_key in KINDS:
        for name, title, _colour, _rarity in GEMS:
            block = pattern.format(name)
            glass = lang_key == "gem_glass"
            write(data / f"silentgems/sunrise/block/{block}.json", {
                "template": "minecraft:glass" if glass else "minecraft:stone_bricks",
                "properties": GLASS if glass else STONE,
                "item": True,
            })
            for sub in (f"blockstates/{block}.json", f"models/block/{block}.json", f"items/{block}.json"):
                shutil.copyfile(gen / "assets/silentgems" / sub, assets / sub)
            (assets / "textures/block").mkdir(parents=True, exist_ok=True)
            shutil.copyfile(main_assets / f"textures/block/{block}.png", assets / f"textures/block/{block}.png")
            shutil.copyfile(gen / f"data/silentgems/loot_table/blocks/{block}.json",
                            data / f"silentgems/loot_table/blocks/{block}.json")
            recipe = json.loads((gen / f"data/silentgems/recipe/{block}.json").read_text())
            # The colourless-glass convention tag holds vanilla glass; name it directly.
            if recipe["key"].get("#") == "#c:glass_blocks/colorless":
                recipe["key"]["#"] = "minecraft:glass"
            write(data / f"silentgems/recipe/{block}.json", recipe)
            lang[f"block.silentgems.{block}"] = upstream_lang[f"block.silentgems.{lang_key}"].replace("%s", title)
            if f"silentgems:{block}" not in tab["items"]:
                tab["items"].append(f"silentgems:{block}")
            if lang_key == "gem_bricks" and f"silentgems:{block}" not in pickaxe["values"]:
                pickaxe["values"].append(f"silentgems:{block}")

    write(lang_path, lang)
    write(tab_path, tab)
    write(pickaxe_path, pickaxe)


if __name__ == "__main__":
    main()
