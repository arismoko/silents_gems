#!/usr/bin/env python3
"""Generate the silver and chaos lines, glowroses, food, pet summoners and teleporter content.

Everything here is declarations, models, recipes, loot, tags and worldgen taken from the
original's own data (`src/generated/resources` and `src/main/resources` at the audited commit),
rewritten into the 26.3 forms the port already uses. Behaviour (teleporting, linking, summoning,
rabbit coffee, bone meal) lives in `src/`.

Usage:
    python3 tools/gen_extras.py --upstream <SilentGems checkout>
"""

from __future__ import annotations

import argparse
import json
import re
import shutil
from pathlib import Path

from gen_content import GEMS

ROOT = Path(__file__).resolve().parent.parent
METAL = {"hardness": 4.0, "explosion_resistance": 30.0, "requires_tool": False, "sounds": "minecraft:iron_block"}

# Convention tags 26.3 does not ship, named by the vanilla item the original's tag held.
DIRECT = {
    "#c:ender_pearls": "minecraft:ender_pearl",
    "#c:ingots/gold": "minecraft:gold_ingot",
    "#c:ingots/iron": "minecraft:iron_ingot",
    "#c:dusts/redstone": "minecraft:redstone",
    "#c:strings": "minecraft:string",
    "#c:storage_blocks/iron": "minecraft:iron_block",
    "#c:rods/wooden": "minecraft:stick",
    "#c:leathers": "minecraft:leather",
}

EFFECT = lambda eid, ticks, amp=0: {"id": f"minecraft:{eid}", "duration": ticks, "amplifier": amp}


def food(nutrition, saturation, always=False):
    value = {"nutrition": nutrition, "saturation": saturation}
    if always:
        value["can_always_eat"] = True
    return value


def consumable(animation, effects):
    return {
        "animation": animation,
        "sound": "minecraft:entity.generic.drink" if animation == "drink" else "minecraft:entity.generic.eat",
        "has_consume_particles": animation != "drink",
        "on_consume_effects": [{"type": "minecraft:apply_effects", "effects": effects, "probability": 1.0}],
    }


def lore(key):
    return [{"translate": key, "color": "gray"}]


# name -> (English, template, components)
ITEMS = {
    "raw_silver": ("Raw Silver", "minecraft:raw_iron", {}),
    "silver_ingot": ("Silver Ingot", "minecraft:iron_ingot", {}),
    "silver_nugget": ("Silver Nugget", "minecraft:iron_nugget", {}),
    "silver_rod": ("Silver Rod", "minecraft:stick", {}),
    "chaos_essence": ("Chaos Essence", "minecraft:diamond", {}),
    "reinforced_gold_rod": ("Reinforced Gold Rod", "minecraft:stick", {}),
    "reinforced_silver_rod": ("Reinforced Silver Rod", "minecraft:stick", {}),
    "teleporter_linker": ("Teleporter Linker", "minecraft:diamond", {"minecraft:max_stack_size": 1}),
    "sparkling_bone_meal": ("Sparkling Bone Meal", "minecraft:diamond", {
        "minecraft:lore": lore("item.silentgems.sparkling_bone_meal.desc")}),
    "summon_kitty": ("Summon Kitty", "minecraft:diamond", {
        "minecraft:enchantment_glint_override": True, "minecraft:lore": lore("item.silentgems.summon_kitty.desc")}),
    "summon_puppy": ("Summon Puppy", "minecraft:diamond", {
        "minecraft:enchantment_glint_override": True, "minecraft:lore": lore("item.silentgems.summon_puppy.desc")}),
    "potato_on_a_stick": ("Potato on a Stick", "minecraft:bread", {
        "minecraft:food": food(6, 0.7), "minecraft:use_remainder": {"id": "minecraft:stick", "count": 1},
        "minecraft:lore": lore("item.silentgems.potato_on_a_stick.desc")}),
    "sugar_cookie": ("Sugar Cookie", "minecraft:bread", {
        "minecraft:food": food(2, 0.4, True),
        "minecraft:consumable": consumable("eat", [EFFECT("speed", 600), EFFECT("haste", 400)])}),
    "cup_of_coffee": ("Cup of Coffee", "minecraft:bread", {
        "minecraft:food": food(1, 0.2, True),
        "minecraft:consumable": consumable("drink", [EFFECT("speed", 3600), EFFECT("haste", 1800)])}),
    "uncooked_meaty_stew": ("Uncooked Meaty Stew", "minecraft:bread", {
        "minecraft:food": food(4, 0.6), "minecraft:use_remainder": {"id": "minecraft:bowl", "count": 1}}),
    "meaty_stew": ("Meaty Stew", "minecraft:bread", {
        "minecraft:food": food(12, 1.6), "minecraft:use_remainder": {"id": "minecraft:bowl", "count": 1}}),
    "uncooked_fishy_stew": ("Uncooked Fishy Stew", "minecraft:bread", {
        "minecraft:food": food(4, 0.5), "minecraft:use_remainder": {"id": "minecraft:bowl", "count": 1}}),
    "fishy_stew": ("Fishy Stew", "minecraft:bread", {
        "minecraft:food": food(10, 1.2), "minecraft:use_remainder": {"id": "minecraft:bowl", "count": 1}}),
    "iron_potato": ("Iron Potato", "minecraft:bread", {
        "minecraft:food": food(9, 0.9, True),
        "minecraft:lore": lore("item.silentgems.iron_potato.desc"),
        "minecraft:consumable": consumable("eat", [
            EFFECT("absorption", 12000, 4), EFFECT("resistance", 6000, 0), EFFECT("strength", 6000, 1)])}),
}

ORES = {
    # name: (English, template for xp/behaviour, hardness, resistance, sound)
    "chaos_ore": ("Chaos Ore", "minecraft:diamond_ore", 5.0, 3.0, "minecraft:stone"),
    "deepslate_chaos_ore": ("Deepslate Chaos Ore", "minecraft:deepslate_diamond_ore", 6.0, 3.0, "minecraft:deepslate"),
    "silver_ore": ("Silver Ore", "minecraft:iron_ore", 3.0, 3.0, "minecraft:stone"),
    "deepslate_silver_ore": ("Deepslate Silver Ore", "minecraft:deepslate_iron_ore", 4.5, 3.0, "minecraft:deepslate"),
}


def write(path: Path, value) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2) + "\n")


def read(path: Path):
    return json.loads(path.read_text())


def flatten_state(node):
    """The 26.2 datagen writes block states as {"Name": id}; 26.3's form is the id."""
    if isinstance(node, dict):
        if "Name" in node and len(node) <= 2:
            return {"id": node["Name"], **({"properties": node["Properties"]} if "Properties" in node else {})}
        return {k: flatten_state(v) for k, v in node.items()}
    if isinstance(node, list):
        return [flatten_state(v) for v in node]
    return node


def fix_ingredient(value):
    if isinstance(value, str):
        return DIRECT.get(value, value)
    if isinstance(value, list):
        return [fix_ingredient(v) for v in value]
    return value


def merge_tag(path: Path, values):
    existing = read(path)["values"] if path.exists() else []
    write(path, {"values": existing + [v for v in values if v not in existing]})


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--upstream", type=Path, required=True)
    args = parser.parse_args()
    up = args.upstream.resolve()
    gen = up / "src/generated/resources"
    main_assets = up / "src/main/resources/assets/silentgems"
    assets = ROOT / "assets/silentgems"
    data = ROOT / "data"
    sg = data / "silentgems"
    lang_path = assets / "lang/en_us.json"
    lang = read(lang_path)
    ids = {"items": [], "blocks": []}

    def declare_block(name, template, properties, title, item=True, extra=None):
        body = {"template": template, "properties": properties, "item": item}
        body.update(extra or {})
        write(sg / f"sunrise/block/{name}.json", body)
        lang[f"block.silentgems.{name}"] = title
        if item:
            ids["blocks"].append(f"silentgems:{name}")

    def copy_asset(sub):
        target = assets / sub
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(gen / "assets/silentgems" / sub, target)

    def copy_texture(kind, name):
        target = assets / f"textures/{kind}/{name}.png"
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(main_assets / f"textures/{kind}/{name}.png", target)

    def copy_loot(name):
        write(sg / f"loot_table/blocks/{name}.json", loot_26_3(read(gen / f"data/silentgems/loot_table/blocks/{name}.json")))

    def copy_recipe(path):
        recipe = read(gen / f"data/silentgems/recipe/{path}.json")
        recipe.pop("category", None)
        if "key" in recipe:
            recipe["key"] = {k: fix_ingredient(v) for k, v in recipe["key"].items()}
        if "ingredients" in recipe:
            recipe["ingredients"] = fix_ingredient(recipe["ingredients"])
        if "ingredient" in recipe:
            recipe["ingredient"] = fix_ingredient(recipe["ingredient"])
        result = recipe["result"]
        result.setdefault("count", 1)
        write(sg / f"recipe/{path}.json", recipe)

    # ---- silver and chaos: ores, storage blocks
    for name, (title, template, hardness, resistance, sound) in ORES.items():
        declare_block(name, template, {"hardness": hardness, "explosion_resistance": resistance,
                                       "requires_tool": True, "sounds": sound}, title)
        for sub in (f"blockstates/{name}.json", f"models/block/{name}.json", f"items/{name}.json"):
            copy_asset(sub)
        copy_texture("block", name)
        write(sg / f"loot_table/blocks/{name}.json", ore_loot(name, "chaos_essence" if "chaos" in name else "raw_silver"))
    for name, title in (("silver_block", "Block of Silver"), ("chaos_essence_block", "Block of Chaos Essence")):
        declare_block(name, "minecraft:iron_block", METAL, title)
        for sub in (f"blockstates/{name}.json", f"models/block/{name}.json", f"items/{name}.json"):
            copy_asset(sub)
        copy_texture("block", name)
        copy_loot(name)

    # ---- items
    for name, (title, template, components) in ITEMS.items():
        body = {"template": template}
        components = dict(components)
        if "minecraft:max_stack_size" in components:
            body["max_stack_size"] = components.pop("minecraft:max_stack_size")
        if components:
            body["components"] = components
        write(sg / f"sunrise/item/{name}.json", body)
        lang[f"item.silentgems.{name}"] = title
        ids["items"].append(f"silentgems:{name}")
        copy_asset(f"items/{name}.json")
        copy_asset(f"models/item/{name}.json")
        copy_texture("item", name)
    lang.update({
        "item.silentgems.iron_potato.desc": "The potato that does not compromise",
        "item.silentgems.potato_on_a_stick.desc": "The legend lives on",
        "item.silentgems.sparkling_bone_meal.desc": "Grows glowroses when used on grass",
        "item.silentgems.summon_kitty.desc": "A much less annoying way to get cats.",
        "item.silentgems.summon_puppy.desc": "Trouble finding those pesky wolves?",
    })

    # ---- glowroses and their pots; teleporters
    anchor_title = "Teleporter Anchor"
    for gem, title, *_ in GEMS:
        rose = f"{gem}_glowrose"
        declare_block(rose, "minecraft:poppy", {"hardness": 0.0, "explosion_resistance": 0.0, "sounds": "minecraft:grass",
                                         "light": 10, "dynamic_light": True},
                      f"{title} Glowrose", extra={"suspicious_stew_effects": [["minecraft:glowing", 160]]})
        declare_block(f"potted_{rose}", "minecraft:potted_poppy", {"hardness": 0.0, "explosion_resistance": 0.0, "light": 15, "dynamic_light": True},
                      f"Potted {title} Glowrose", item=False)
        for sub in (f"blockstates/{rose}.json", f"blockstates/potted_{rose}.json", f"models/item/{rose}.json",
                    f"items/{rose}.json"):
            copy_asset(sub)
        for sub, parent in ((rose, "minecraft:block/cross"), (f"potted_{rose}", "minecraft:block/flower_pot_cross")):
            model = read(gen / f"assets/silentgems/models/block/{sub}.json")
            model["parent"] = parent
            model["render_type"] = "minecraft:cutout"
            write(assets / f"models/block/{sub}.json", model)
        copy_texture("block", rose)
        copy_loot(rose)
        copy_loot(f"potted_{rose}")
        for tag in ("block", "item"):
            write(sg / f"tags/{tag}/glowroses/{gem}.json", {"values": [f"silentgems:{rose}"]})
        copy_recipe_glowrose = [p.stem for p in (gen / "data/silentgems/recipe").glob(f"*_dye_from_{rose}.json")]
        for stem in copy_recipe_glowrose:
            copy_recipe(stem)

        for kind, label in (("teleporter", "Teleporter"), ("redstone_teleporter", "Redstone Teleporter")):
            name = f"{gem}_{kind}"
            props = {"hardness": 5.0, "explosion_resistance": 5.0, "requires_tool": False, "sounds": "minecraft:iron_block"}
            extra = {"state_properties": {"triggered": {"type": "boolean"}}} if kind == "redstone_teleporter" else None
            declare_block(name, "minecraft:iron_block", props, f"{title} {label}", extra=extra)
            model = read(gen / f"assets/silentgems/models/block/{name}.json")
            write(assets / f"models/block/{name}.json", model)
            copy_asset(f"items/{name}.json")
            state = {"model": f"silentgems:block/{name}"}
            write(assets / f"blockstates/{name}.json",
                  {"variants": {"triggered=false": state, "triggered=true": state}} if extra else {"variants": {"": state}})
            copy_loot(name)
            copy_recipe(name)
    declare_block("teleporter_anchor", "minecraft:iron_block",
                  {"hardness": 5.0, "explosion_resistance": 3.0, "requires_tool": True, "sounds": "minecraft:iron_block"},
                  anchor_title)
    write(assets / "blockstates/teleporter_anchor.json", {"variants": {"": {"model": "silentgems:block/teleporter_anchor"}}})
    for sub in ("models/block/teleporter_anchor.json", "items/teleporter_anchor.json"):
        copy_asset(sub)
    for sub in ("cube_overlay", "cube_overlay_all", "teleporter"):
        shutil.copyfile(main_assets / f"models/block/{sub}.json", assets / f"models/block/{sub}.json")
    for tex in ("redstone_teleporter_frame", "teleporter_anchor_frame", "teleporter_anchor_frame_side",
                "teleporter_frame", "teleporter_frame_side"):
        copy_texture("block", tex)
    copy_loot("teleporter_anchor")
    copy_recipe("teleporter_anchor")

    lang.update({
        "silentgems.configuration.glowrose": "Glowrose",
        "silentgems.configuration.rabbitsProduceCoffee": "Rabbits Produce Coffee",
    })

    # ---- recipes with no block counterpart
    for path in ("silver_block", "silver_ingot_from_block", "silver_ingot_from_nugget", "silver_nugget", "silver_rod",
                 "chaos_essence_block", "chaos_essence_from_block", "reinforced_gold_rod", "reinforced_silver_rod",
                 "teleporter_linker", "sparkling_bone_meal", "summon_kitty", "summon_puppy", "potato_on_a_stick",
                 "sugar_cookie", "iron_potato", "uncooked_meaty_stew", "uncooked_fishy_stew",
                 "smelting/silver_ingot", "blasting/silver_ingot", "smelting/meaty_stew", "smoking/meaty_stew",
                 "campfire_cooking/meaty_stew", "smelting/fishy_stew", "smoking/fishy_stew", "campfire_cooking/fishy_stew"):
        copy_recipe(path)
    # The silver and chaos block recipes' upstream names are swapped (`silver_block` unpacks); keep ids, fix nothing.

    # ---- tags
    for sub in ("tags/entity_type/coffee_producers", "tags/item/stew_fish", "tags/item/stew_meat"):
        write(sg / f"{sub}.json", read(gen / f"data/silentgems/{sub}.json"))
    glow_tags = [f"#silentgems:glowroses/{g[0]}" for g in GEMS]
    for tag in ("block", "item"):
        write(sg / f"tags/{tag}/glowroses.json", {"values": glow_tags})
    write(sg / "tags/item/gems.json", {"values": [f"silentgems:{g[0]}" for g in GEMS]})
    potted = [f"silentgems:potted_{g[0]}_glowrose" for g in GEMS]
    merge_tag(data / "minecraft/tags/block/flowers.json", glow_tags)
    merge_tag(data / "minecraft/tags/block/flower_pots.json", potted)
    merge_tag(data / "c/tags/item/flowers/small.json", glow_tags)
    merge_tag(data / "c/tags/item/ingots/silver.json", ["silentgems:silver_ingot"])
    merge_tag(data / "c/tags/item/nuggets/silver.json", ["silentgems:silver_nugget"])
    merge_tag(data / "c/tags/item/rods/silver.json", ["silentgems:silver_rod"])
    for kind, ores, block in (("chaos", ("chaos_ore", "deepslate_chaos_ore"), "chaos_essence_block"),
                              ("silver", ("silver_ore", "deepslate_silver_ore"), "silver_block")):
        stone = [f"silentgems:{o}" for o in ores]
        for tag in ("block", "item"):
            merge_tag(data / f"c/tags/{tag}/ores/{kind}.json", stone)
            merge_tag(data / f"c/tags/{tag}/storage_blocks/{kind}.json", [f"silentgems:{block}"])
            merge_tag(data / f"c/tags/{tag}/ores.json", [f"#c:ores/{kind}"])
            merge_tag(data / f"c/tags/{tag}/storage_blocks.json", [f"silentgems:{block}"])
    pick = ["silentgems:chaos_ore", "silentgems:deepslate_chaos_ore", "silentgems:silver_ore",
            "silentgems:deepslate_silver_ore", "silentgems:chaos_essence_block", "silentgems:silver_block",
            "silentgems:teleporter_anchor"]
    pick += [f"silentgems:{g[0]}_{k}" for g in GEMS for k in ("teleporter", "redstone_teleporter")]
    merge_tag(data / "minecraft/tags/block/mineable/pickaxe.json", pick)
    merge_tag(data / "minecraft/tags/block/needs_iron_tool.json",
              ["silentgems:silver_ore", "silentgems:deepslate_silver_ore"])
    merge_tag(data / "minecraft/tags/block/needs_diamond_tool.json",
              ["silentgems:chaos_ore", "silentgems:deepslate_chaos_ore"])

    # ---- worldgen: silver, chaos and the 21 glowroses in each dimension
    dims = (("overworld", "#minecraft:is_overworld"), ("the_nether", "#minecraft:is_nether"),
            ("the_end", "#minecraft:is_end"))
    wg = gen / "data/silentgems/worldgen"
    for dim, biomes in dims:
        stems = [f"{g[0]}_glowrose" for g in GEMS]
        if dim == "overworld":
            stems = ["silver_ore", "chaos_ore"] + stems
        for stem in stems:
            spec = flatten_state(read(wg / f"configured_feature/{dim}/{stem}.json"))
            config = spec.pop("config", None)
            if isinstance(config, dict):
                spec.update(config)
            write(sg / f"worldgen/feature/{dim}/{stem}.json", spec)
            write(sg / f"worldgen/placed_feature/{dim}/{stem}.json", read(wg / f"placed_feature/{dim}/{stem}.json"))
        modifier = sg / f"neoforge/biome_modifier/{dim}_glowroses.json"
        roses = [f"silentgems:{dim}/{s}" for s in stems if s.endswith("glowrose")]
        write(modifier, {"type": "neoforge:add_features", "biomes": biomes, "features": roses,
                         "step": "underground_ores"})
    write(sg / "neoforge/biome_modifier/overworld_materials.json",
          {"type": "neoforge:add_features", "biomes": "#minecraft:is_overworld",
           "features": ["silentgems:overworld/silver_ore", "silentgems:overworld/chaos_ore"],
           "step": "underground_ores"})

    # ---- creative tab and lang
    tab_path = sg / "sunrise/creative_tab/tab.json"
    tab = read(tab_path)
    tab["items"] = list(dict.fromkeys(tab["items"] + ids["items"] + ids["blocks"]))
    write(tab_path, tab)
    write(lang_path, dict(sorted(lang.items())))
    print(f"{len(ids['items'])} items and {len(ids['blocks'])} block items written")


def ore_loot(name, drop):
    """An ore's table in the shape of vanilla 26.3's `lapis_ore`/`redstone_ore`: itself under silk touch, else its drop."""
    if drop == "chaos_essence":
        modifier = [
            {"type": "minecraft:set_count", "count": {"type": "minecraft:uniform", "min": 1, "max": 3}},
            {"type": "minecraft:apply_bonus", "enchantment": "minecraft:fortune",
             "formula": "minecraft:uniform_bonus_count", "parameters": {"bonusMultiplier": 1}},
            {"type": "minecraft:explosion_decay"},
        ]
    else:
        modifier = [
            {"type": "minecraft:apply_bonus", "enchantment": "minecraft:fortune", "formula": "minecraft:ore_drops"},
            {"type": "minecraft:explosion_decay"},
        ]
    return {"type": "minecraft:block", "pools": [{"rolls": 1, "entries": [{"type": "minecraft:alternatives", "children": [
        {"type": "minecraft:item", "condition": "minecraft:tool/can_silk_touch", "name": f"silentgems:{name}"},
        {"type": "minecraft:item", "modifier": modifier, "name": f"silentgems:{drop}"}]}]}],
        "random_sequence": f"silentgems:blocks/{name}"}


def loot_26_3(table):
    """The 26.2 datagen's loot shape to the one the port's other tables use."""
    def fix(node):
        if isinstance(node, dict):
            out = {}
            for key, value in node.items():
                if key == "conditions":
                    if isinstance(value, list) and len(value) == 1:
                        out["condition"] = fix(value[0])
                    else:
                        out["condition"] = {"type": "minecraft:all_of", "terms": fix(value)}
                    continue
                if key == "condition" and isinstance(value, str):
                    out["type"] = value
                    continue
                if key == "functions":
                    out["modifier"] = fix(value)
                    continue
                if key == "function":
                    out["type"] = value
                    continue
                out[key] = fix(value)
            return out
        if isinstance(node, list):
            return [fix(v) for v in node]
        return node
    return fix(table)


if __name__ == "__main__":
    main()
