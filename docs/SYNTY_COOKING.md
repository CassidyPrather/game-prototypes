# Synty meshes for the cooking module

A lookup from every food in [the cooking module](COOKING.md) to a mesh from the Synty packs, and
from the things around it in [Kitchen Garden](KITCHEN_GARDEN.md) (stations, well, wood, coins) to
one as well. Nothing here uses any of them yet; this is the list to hand when it should.

The same data, as something to read from code, is [`art/synty-cooking.toml`](../art/synty-cooking.toml).
It carries the digest and the path inside the pack for each primary pick. This file is for people.

## Reading it

**The id** is `<pack>/<mesh>`, for example `polygon_fantasy_village/sm_prop_food_carrot_01`. It is the key
Space Trucking's art catalogue (`C:/Source/space-trucking/art/dex/`) files that mesh under. The mesh name
alone is not unique: `SM_Prop_Barrel_01` exists in dozens of packs, and `SM_Prop_Well_01` is in three of
the ones used here. The `sha256` in the TOML pins the exact file.

**Fit** says how good the match is:

| Fit | Meaning |
| --- | --- |
| exact | It is the food. |
| close | The right thing in another form or state: a shaker for salt, a slice for chopped. |
| stand-in | A vessel or proxy. Tint it, fill it, or let the game's own icon do the work. |
| gap | Nothing fits. The nearest is named and the rest has to be composed. |

## How well it covers

Of the 41 foods: **14 exact, 9 close, 15 stand-in, 3 gap.**

The packs, by how many primaries come from each:

| Pack | Primaries | Store directory |
| --- | --- | --- |
| `polygon_fantasy_village` | 28 | `POLYGON - Fantasy Village` |
| `polygon_shops_pack` | 6 | `POLYGON - Shops Pack` |
| `polygon_farm_pack` | 3 | `POLYGON - Farm Pack` |
| `polygon_fantasy_kingdom_pack` | 1 | `POLYGON - Fantasy Kingdom Pack` |
| `polygon_xmax_pack` | 1 | `POLYGON - Xmax Pack` |
| `polygon_viking_realm` | 1 | `POLYGON - Viking Realm` |
| `polygon_ancient_egypt` | 1 | `POLYGON - Ancient Egypt` |

**POLYGON Fantasy Village does most of it.** It is a village-kitchen pack and has the sack of flour,
the butter, the pancakes, the tea set, the stew, the loaf and the cake, and it is the one set whose
pieces look as if they belong together. Farm Pack fills the crops it lacks (onion, beans, grain).
Shops Pack fills the rest, mainly the cut and burnt states. Anything outside those three is one-off.

Packs that were searched and have almost nothing: Town Pack (a few items), Coffee Shop (no food),
Gingerbread (no useful ingredients), Icons Pack (flat food icons, useful only for a UI).

## Things to know before using it

- **Shops Pack colours in the catalogue are wrong.** The catalogue rendered its food with the building
  atlas, so every Shops mesh is brown in the preview and every description says "muted brown". Trust
  the shape and the triangle count; the real atlas is one of the `PolygonShops_Texture_0N_*` files.
  The same wrong-atlas problem makes Viking Realm's flatbread and Samurai's rice plant look black.
- **`SM_Prop_Food_Sugar_01` and `_02` (Shops Pack) are wooden planks**, not sugar.
  Fantasy Village's `sm_prop_sugar_jar_01` is shaped like a strawberry.
- **Rice is a real gap.** No pack searched has grains of rice. Steamed rice and rice pudding are stand-ins
  and fried rice is a plate of yellow chunks that reads as rice.
- **No live animals.** There is no hen or cow in any POLYGON pack searched (the cow in Dark Fantasy is
  dead). The nest and the chicken coop exist.
- **Cups, bowls and the pail are empty.** Tea, milk tea, water and batter are shown by their vessel;
  the liquid is something the game would add.
- Descriptions in the catalogue were written by a vision model from a picture. I chose the picks above
  after looking at the preview renders of the uncertain ones, not from the descriptions alone.

## Raw ingredients

| Food | Fit | Primary | Tris | Alternatives | Note |
| --- | --- | --- | --- | --- | --- |
| `Food::Water` | stand-in | `polygon_fantasy_village/sm_prop_bucket_01` | 585 | `polygon_fantasy_village/sm_prop_jug_02`: white pitcher with blue bands<br>`polygon_farm_pack/sm_prop_tool_bucket_01`: plain metal pail<br>`polygon_fantasy_village/sm_prop_well_01`: the well itself | Water has no shape of its own, so show the vessel: a wooden pail. The game fetches it from a well with a crank, which suits. |
| `Food::Milk` | stand-in | `polygon_fantasy_village/sm_prop_jug_02` | 346 | `polygon_shops_pack/sm_prop_cafe_milk_jug_01`: small jug<br>`polygon_shops_pack/sm_prop_product_milk_01`: milk carton<br>`polygon_office_pack/sm_prop_milk_01`: office fridge milk<br>`polygon_icons_pack/sm_icon_food_milk_01`: flat icon-style | A white pitcher reads as milk. Same trick as water: the container is the icon. |
| `Food::Egg` | exact | `polygon_fantasy_village/sm_prop_food_egg_01` | 80 | `polygon_shops_pack/sm_prop_food_egg_whole_01`: whole egg<br>`polygon_fantasy_village/sm_prop_nest_01`: the nest, holding three pastel eggs<br>`polygon_easter_pack/sm_easter_egg_1`: painted egg (there are 14: sm_easter_egg_1 to _14) | A plain white whole egg. |
| `Food::Carrot` | exact | `polygon_fantasy_village/sm_prop_food_carrot_01` | 182 | `polygon_farm_pack/sm_prop_carrot_01`: Farm Pack carrot, 116 tris<br>`polygon_farm_pack/sm_prop_carrot_01_group`: three carrots<br>`polygon_farm_pack/sm_prop_box_carrot_01`: crate of carrots<br>`polygon_fantasy_village/sm_prop_food_carrot_giant_01`: oversized, pale | Orange root with a leafy top. |
| `Food::Onion` | exact | `polygon_farm_pack/sm_prop_onion_01` | 100 | `polygon_shops_pack/sm_prop_food_onion_whole_01`: whole onion<br>`polygon_farm_pack/sm_prop_onion_01_group`: three onions<br>`polygon_fantasy_kingdom_pack/sm_prop_onion_rope_01`: string of onions<br>`polygon_town_pack/sm_item_onion_01`: green onion with bulb | Fantasy Village has no whole onion; Farm Pack has the best one. |
| `Food::Tomato` | exact | `polygon_fantasy_village/sm_prop_food_tomato_01` | 206 | `polygon_farm_pack/sm_prop_tomato_01`: Farm Pack tomato<br>`polygon_town_pack/sm_item_tomato_01`: Town Pack tomato<br>`polygon_shops_pack/sm_prop_food_tomato_whole_01`: whole tomato<br>`polygon_farm_pack/sm_prop_tomato_01_group`: three tomatoes | Red, with a green calyx. |
| `Food::Wheat` | exact | `polygon_fantasy_kingdom_pack/sm_prop_wheat_bunch_01` | 778 | `polygon_farm_pack/sm_chr_attach_wheat_01`: single stalk (a character attachment)<br>`polygon_farm_pack/sm_prop_plant_wheat_02`: standing cluster, for the plot<br>`polygon_farm_pack/sm_prop_plant_wheat_optimised_02`: cheaper standing cluster, 105 tris<br>`polygon_ancient_egypt/sm_prop_wheat_bunch_01`: sheaf, tied with pale rope<br>`simple_fantasy/sf_prop_crops_wheatstack_01`: SIMPLE Fantasy sheaf | A tied sheaf, which reads better as a held item than a single stalk. |
| `Food::Rice` | gap | `polygon_farm_pack/sm_prop_grainbag_open_01` | 158 | `polygon_samurai/sm_env_riceplant_01`: rice plant, for the plot<br>`polygon_samurai/sm_prop_basket_rice_01`: potted rice plant<br>`polygon_fantasy_village/sm_prop_sack_02`: open sack<br>`polygon_icons_pack/sm_icon_crafting_grain_01`: flat grain icon | No pack here has grains of rice. An open sack of grain is the honest stand-in; the one rice plant (Samurai) renders as a flat black tuft and is a weak match. |
| `Food::Beans` | exact | `polygon_farm_pack/sm_prop_bean_01_group` | 380 | `polygon_farm_pack/sm_prop_bean_01_l`: one pod, large<br>`polygon_farm_pack/sm_prop_bean_01_m`: one pod, medium<br>`polygon_farm_pack/sm_prop_bean_01_s`: one pod, small<br>`polygon_farm_pack/sm_prop_plant_bush_01_bean`: bean plant, for the plot | A pile of green bean pods. |
| `Food::Sugar` | stand-in | `polygon_shops_pack/sm_prop_cafe_sugar_box_01` | 318 | `polygon_office_pack/sm_prop_mints_01`: bowl of white blocks<br>`polygon_pirate_pack/sm_prop_sugarcane_pile_01`: sugarcane stalks<br>`polygon_fantasy_village/sm_prop_sugar_jar_01`: strawberry-shaped jar | A box of sugar cubes. WARNING: SM_Prop_Food_Sugar_01 and _02 in the Shops Pack are wooden planks, not sugar; the name is misleading. Fantasy Village's sugar jar is shaped like a strawberry and reads as jam. |
| `Food::TeaLeaf` | stand-in | `polygon_fantasy_village/sm_prop_food_leaf_01` | 32 | `polygon_viking_realm/sm_prop_herb_01`: herb bundle<br>`polygon_fantasy_kingdom_pack/sm_prop_herb_bunch_01`: lavender bunch<br>`polygon_icons_pack/sm_icon_crafting_leaf_01`: flat leaf icon | One flat lime-green leaf, 32 tris. |
| `Food::Spice` | close | `polygon_fantasy_village/sm_prop_spice_jar_01` | 260 | `polygon_fantasy_kingdom_pack/sm_prop_pot_spice_01`: pot of yellow ground spice<br>`polygon_fantasy_kingdom_pack/sm_prop_pot_spice_02`: pot of orange spice<br>`polygon_fantasy_kingdom_pack/sm_prop_pot_spice_03`: pot of crimson spice<br>`polygon_farm_pack/sm_prop_pepper_01`: a pepper<br>`polygon_farm_pack/sm_prop_plant_bush_03_chilli`: chilli plant, for the plot | A shaker with a red band. |
| `Food::Salt` | close | `polygon_shops_pack/sm_prop_cafe_shaker_salt_01` | 100 | `polygon_shops_pack/sm_prop_cafe_grinder_salt_01`: grinder<br>`polygon_shops_pack/sm_prop_cafe_salt_grinder_01`: wooden grinder<br>`polygon_fantasy_kingdom_pack/sm_item_salt_pepper_grinder_01`: tall wooden grinder<br>`polygon_apocalypse_wasteland/sm_env_salt_round_01`: salt flat, an environment piece | A shaker. It is a different shape from the spice jar, which is the point. |

## Prepared

| Food | Fit | Primary | Tris | Alternatives | Note |
| --- | --- | --- | --- | --- | --- |
| `Food::Flour` | exact | `polygon_fantasy_village/sm_prop_food_flour_01` | 362 | `polygon_farm_pack/sm_prop_grainbag_01`: plain grain sack | A sack stamped FLOUR with a mound on top. |
| `Food::Dough` | close | `polygon_shops_pack/sm_prop_food_pizza_dough_01` | 90 | `polygon_fantasy_village/sm_prop_food_steambun_02`: pale creased bun | A flat unbaked disc. |
| `Food::RisenDough` | stand-in | `polygon_fantasy_village/sm_prop_food_steambun_02` | 144 | `polygon_shops_pack/sm_prop_food_pizza_dough_01`: the dough disc, scaled | A domed pale bun. Or reuse the dough mesh scaled up: the game's own icon is dough rising, so growth is the cue. |
| `Food::Batter` | stand-in | `polygon_fantasy_village/sm_prop_food_sauce_01` | 112 | `polygon_fantasy_village/sm_prop_bowl_02`: empty clay bowl<br>`polygon_fantasy_village/sm_prop_bowl_01`: empty wooden bowl | A white bowl already holding a pool of glossy sauce; tint the pool pale yellow. |
| `Food::Butter` | exact | `polygon_fantasy_village/sm_prop_food_butter_01` | 112 |  | A wrapped block of butter. |
| `Food::HotWater` | stand-in | `polygon_fantasy_village/sm_prop_teapot_03` | 810 | `polygon_fantasy_kingdom_pack/sm_item_kettle_01`: copper kettle<br>`polygon_fantasy_village/sm_prop_cooking_pot_01`: lidded pot<br>`polygon_fantasy_village/sm_prop_teapot_02`: teapot | A red kettle. Add steam with a particle. |
| `Food::ChoppedCarrot` | exact | `polygon_fantasy_village/sm_prop_food_carrot_slice_01` | 26 |  | An orange octagonal coin. |
| `Food::ChoppedOnion` | close | `polygon_fantasy_village/sm_prop_food_spring_onion_01` | 48 | `polygon_shops_pack/sm_prop_food_onion_sliced_01`: scatter of cut pieces<br>`polygon_shops_pack/sm_prop_food_onion_slice_01`: one ring<br>`polygon_shops_pack/sm_prop_food_onion_ring_01`: torus ring | A green ring with a yellow core, which is a spring onion slice. |
| `Food::ChoppedTomato` | exact | `polygon_fantasy_village/sm_prop_food_tomato_slice_01` | 82 | `polygon_shops_pack/sm_prop_food_tomato_sliced_01`: two slices<br>`polygon_shops_pack/sm_prop_food_tomato_cut_01`: halved<br>`polygon_shops_pack/sm_prop_food_tomato_chunk_01`: chunk | A red disc with a pale star. |
| `Food::SoakedBeans` | gap | `polygon_fantasy_village/sm_prop_bowl_01` | 228 | `polygon_farm_pack/sm_prop_bean_01_group`: pods to put in the bowl | No soaked beans. Compose: a bowl with the bean pods from the Farm Pack in it. |

## Dishes

| Food | Fit | Primary | Tris | Alternatives | Note |
| --- | --- | --- | --- | --- | --- |
| `Food::Tea` | stand-in | `polygon_fantasy_village/sm_prop_tea_cup_02` | 220 | `polygon_fantasy_village/sm_prop_tea_cup_01`: pink cup<br>`polygon_fantasy_village/sm_prop_tea_saucer_01`: saucer<br>`polygon_fantasy_village/sm_prop_teapot_02`: teapot<br>`polygon_kids_pack/sm_prop_tea_cup_01`: Kids Pack cup | A white cup with a red rim. The cups are empty, so tint a disc inside. |
| `Food::MilkTea` | stand-in | `polygon_fantasy_village/sm_prop_tea_cup_01` | 674 | `polygon_kids_pack/sm_prop_tea_cup_02`: lavender Kids Pack cup | The pink cup, so it reads apart from tea. |
| `Food::Salad` | stand-in | `polygon_fantasy_village/sm_prop_food_lettuce_01` | 562 | `polygon_fantasy_village/sm_prop_bowl_01`: bowl to compose in<br>`polygon_fantasy_kingdom_pack/sm_item_food_bowl_01`: dark wooden bowl | A green head of lettuce. Or compose a bowl from the three chopped-vegetable meshes. |
| `Food::BoiledEgg` | stand-in | `polygon_shops_pack/sm_prop_food_egg_whole_01` | 80 | `polygon_fantasy_village/sm_prop_food_egg_01`: whole egg<br>`polygon_easter_pack/sm_easter_egg_1`: painted egg | Nothing shows a boiled egg; a second whole egg is as close as it gets. |
| `Food::SteamedRice` | gap | `polygon_fantasy_village/sm_prop_steam_bowl_01` | 240 | `polygon_icons_pack/sm_icon_food_rice_bowl_01`: flat rice-bowl icon<br>`polygon_fantasy_village/sm_prop_bowl_02`: clay bowl | An empty wooden bowl; add a white mound. |
| `Food::Soup` | close | `polygon_fantasy_village/sm_prop_food_sauce_02` | 353 | `polygon_casino/sm_prop_sushi_bowl_01`: bowl of pale broth<br>`polygon_fantasy_kingdom_pack/sm_item_pot_02_stew`: pot of stew<br>`polygon_military_pack/sm_prop_pot_large_soup_01`: big pot of soup | A white bowl of red soup with a spoon in it. |
| `Food::BeanStew` | close | `polygon_fantasy_village/sm_prop_food_bowl_insert_06` | 879 | `polygon_fantasy_kingdom_pack/sm_item_pan_01_stew`: pan of stew<br>`polygon_fantasy_kingdom_pack/sm_item_pot_02_stew`: pot of stew | A hearty dish of stew with vegetables and green and brown pieces. |
| `Food::Custard` | close | `polygon_shops_pack/sm_prop_food_custard_square_01` | 70 | `polygon_xmax_pack/sm_xmas_pudding`: Christmas pudding<br>`polygon_fantasy_village/sm_prop_food_sauce_01`: bowl of sauce | A square of custard slice. The name matches. |
| `Food::RicePudding` | stand-in | `polygon_xmax_pack/sm_xmas_pudding` | 382 | `polygon_casino/sm_prop_sushi_bowl_01`: bowl of pale broth<br>`polygon_fantasy_village/sm_prop_food_sauce_01`: bowl of sauce | A pudding on a saucer. No rice pudding exists; tint it pale. |
| `Food::FriedEgg` | exact | `polygon_fantasy_village/sm_prop_food_egg_02` | 110 | `polygon_shops_pack/sm_prop_food_egg_cooked_01`: cooked egg disc<br>`polygon_shops_pack/sm_prop_food_egg_burnt_01`: the same, burnt | A fried egg with a yolk. |
| `Food::Omelette` | stand-in | `polygon_shops_pack/sm_prop_food_egg_cooked_01` | 74 | `polygon_shops_pack/sm_prop_food_quiche_01`: quiche<br>`polygon_fantasy_village/sm_prop_food_egg_02`: fried egg | A flat cooked disc; tint it yellow. There is no folded omelette. |
| `Food::Pancake` | exact | `polygon_fantasy_village/sm_prop_food_pancakes_03` | 456 | `polygon_fantasy_village/sm_prop_food_pancakes_01`: with syrup and strawberries<br>`polygon_fantasy_village/sm_prop_food_pancakes_02`: with syrup and orange<br>`polygon_fantasy_village/sm_prop_food_pancakes_04`: plain stack<br>`polygon_shops_pack/sm_prop_food_pancake_01`: Shops Pack stack | A stack of three with a pat of butter. |
| `Food::FriedRice` | stand-in | `polygon_fantasy_village/sm_prop_food_bowl_insert_01` | 1073 | `polygon_fantasy_village/sm_prop_food_bowl_insert_05`: topped flatbread | Yellow chunks flecked with green and red on an orange-rimmed dish. |
| `Food::Flatbread` | close | `polygon_viking_realm/sm_prop_flatbread_01` | 614 | `polygon_shops_pack/sm_prop_food_pizza_cooked_03`: plain baked disc<br>`polygon_fantasy_village/sm_prop_food_bowl_insert_05`: flatbread with toppings | A named flatbread. Its preview is black, which is a wrong-atlas guess and not the mesh. |
| `Food::Bread` | exact | `polygon_fantasy_village/sm_prop_food_bread_loaf_01` | 240 | `polygon_fantasy_village/sm_prop_food_bread_loaf_02`: loaf<br>`polygon_fantasy_village/sm_prop_food_bread_loaf_03`: round loaf<br>`polygon_fantasy_village/sm_prop_food_bread_loaf_04`: baguette<br>`polygon_fantasy_village/sm_prop_food_bread_slice_01`: slice<br>`polygon_fantasy_village/sm_prop_food_bread_toast_01`: toast<br>`polygon_fantasy_kingdom_pack/sm_item_bread_01`: Fantasy Kingdom loaf | A scored golden loaf. |
| `Food::Cake` | exact | `polygon_fantasy_village/sm_prop_food_cake_05` | 516 | `polygon_fantasy_village/sm_prop_food_cake_01`: cream and berries<br>`polygon_fantasy_village/sm_prop_food_cake_04`: pink icing<br>`polygon_fantasy_village/sm_prop_food_cake_slice_01`: a slice<br>`polygon_fantasy_village/sm_prop_food_cupcake_01`: cupcake<br>`polygon_kids_pack/sm_prop_cake_02`: birthday cake<br>`polygon_shops_pack/sm_prop_food_cake_01`: Shops Pack cake | A layered sponge with jam. The others are fancier. |

## Waste

| Food | Fit | Primary | Tris | Alternatives | Note |
| --- | --- | --- | --- | --- | --- |
| `Food::Mush` | stand-in | `polygon_fantasy_village/sm_prop_food_bowl_insert_04` | 1077 | `polygon_dark_fantasy/sm_prop_sludge_rotten_01`: grey sludge<br>`polygon_dark_fantasy/sm_prop_bread_rotten_01`: mouldy bread | A heap of discarded food scraps. |
| `Food::Charcoal` | close | `polygon_ancient_egypt/sm_prop_brazier_04_insert_01` | 148 | `polygon_ancient_egypt/sm_prop_brazier_02_insert_01`: charred logs<br>`polygon_dark_fortress/sm_prop_brazier_04_insert_01`: Dark Fortress version<br>`polygon_shops_pack/sm_prop_food_egg_burnt_01`: burnt egg<br>`polygon_shops_pack/sm_prop_food_onion_burnt_01`: burnt onion | A mound of ash with charred logs. |

## Around the cooking

Not foods, but the Kitchen Garden demo draws them. Same format.

| Thing | Fit | Primary | Tris | Alternatives | Note |
| --- | --- | --- | --- | --- | --- |
| Chopping board | close | `polygon_fantasy_village/sm_prop_cutting_board_02` | 80 | `polygon_fantasy_village/sm_prop_chopping_block_01`: stump block<br>`polygon_fantasy_village/sm_prop_cutting_board_01`: paddle board | Flat board. |
| Quern (grind) | stand-in | `polygon_shops_pack/sm_prop_cafe_grinder_01` | 858 | `polygon_fantasy_kingdom_pack/sm_item_mortar_pestle_01`: mortar and pestle<br>`polygon_fantasy_kingdom_pack/sm_prop_mortar_and_pestle_01`: mortar and pestle, prop<br>`polygon_fantasy_village/sm_prop_waterwheel_01`: waterwheel, if it is a mill | A hand-cranked grinder; no quern or millstone exists. |
| Bowl (knead, mix, churn) | exact | `polygon_fantasy_village/sm_prop_bowl_02` | 286 | `polygon_fantasy_village/sm_prop_bowl_01`: wooden bowl<br>`polygon_fantasy_village/sm_prop_rolling_pin_01`: rolling pin<br>`polygon_fantasy_village/sm_prop_wooden_spoon_01`: wooden spoon |  |
| Crock (prove, soak, steep) | close | `polygon_fantasy_village/sm_prop_pot_01` | 540 | `polygon_fantasy_village/sm_prop_jar_04`: clay jar<br>`polygon_fantasy_village/sm_prop_jar_03`: terracotta jar | Clay pot with a lid. |
| Pot (boil) | exact | `polygon_fantasy_village/sm_prop_cooking_pot_01` | 520 | `polygon_fantasy_kingdom_pack/sm_item_pot_02`: iron pot<br>`polygon_fantasy_village/sm_prop_pot_hanging_01`: hanging pot |  |
| Pan (fry) | exact | `polygon_fantasy_village/sm_prop_frying_pan_01` | 270 | `polygon_fantasy_kingdom_pack/sm_item_pan_01_stew`: pan with stew |  |
| Oven (bake) | exact | `polygon_fantasy_village/sm_prop_oven_02` | 1526 | `polygon_fantasy_village/sm_prop_oven_03`: green domed oven<br>`polygon_fantasy_village/sm_prop_woodburner_01`: pot-bellied stove<br>`polygon_fantasy_village/sm_prop_oven_01`: stove | Clay oven. |
| Wood log / wood pile | close | `polygon_fantasy_village/sm_prop_wood_pile_02` | 996 | `polygon_fantasy_village/sm_prop_firewood_rack_01`: log rack<br>`polygon_farm_pack/sm_prop_wood_stack_01`: Farm Pack pile |  |
| Axe | exact | `polygon_fantasy_village/sm_prop_wood_axe_01` | 242 | `polygon_fantasy_village/sm_prop_axe_02`: red-wrapped axe<br>`polygon_farm_pack/sm_prop_tool_axe_01`: Farm Pack axe |  |
| Stump (split wood) | exact | `polygon_fantasy_village/sm_prop_chopping_block_01` | 150 |  |  |
| Well and crank | exact | `polygon_fantasy_village/sm_prop_well_01` | 4442 | `polygon_farm_pack/sm_prop_well_01`: Farm Pack well<br>`polygon_fantasy_kingdom_pack/sm_prop_well_01`: Fantasy Kingdom well<br>`polygon_fantasy_kingdom_pack/sm_prop_well_01_handle`: crank handle on its own |  |
| Compost heap | close | `polygon_town_pack/sm_prop_compost_01` | 316 |  | A black bin. |
| Hen's nest | exact | `polygon_fantasy_village/sm_prop_nest_01` | 504 | `polygon_farm_pack/sm_prop_chicken_coop_01`: chicken coop | Nest holding three eggs. There is no hen mesh. |
| Garden plot | close | `polygon_farm_pack/sm_env_vege_rows_01` | 499 | `polygon_fantasy_village/sm_prop_planter_01`: planter<br>`polygon_farm_pack/sm_prop_plantbox_large_01`: large plant box<br>`polygon_farm_pack/sm_prop_seedpacket_01`: seed packet | Rows of crop shoots. |
| Coin | close | `polygon_pirate_realm/sm_prop_coin_01` | 24 | `polygon_icons_pack/sm_icon_coin_01`: flat icon coin<br>`polygon_dungeon_pack/sm_item_coins_01`: coin pile | Gold coin. |
| Pantry basket / crate | close | `polygon_fantasy_village/sm_prop_basket_02` | 348 | `polygon_fantasy_village/sm_prop_crate_01`: crate<br>`polygon_farm_pack/sm_prop_crate_01`: Farm Pack crate |  |

Missing entirely: a live hen and a cow.

## Using one

Space Trucking's pipeline is the thing that turns these into something a game can load. In outline:

1. Add each pack under `[pack.<id>]` in that repo's `art/manifest.toml`, with `dir` set to the store
   directory in the table above. `$SYNTY_STORE` is `C:\Users\prath\OneDrive\synty` on this machine.
2. Add an `[asset.<name>]` table with `pack`, and `source` and `sha256` copied from the matching table in
   `art/synty-cooking.toml`.
3. `cargo xtask art resolve` converts it to a glb in `art/cache/`.

Synty's licence lets the meshes ship in a built game and forbids committing them as source, which is why
this repository holds only the ids. Keep it that way: no mesh, texture or preview image goes in here.

To look at one first, the catalogue keeps a four-view preview of each mesh at
`C:/Source/space-trucking/art/cache/dex/<sha256>/preview.png`, and `cargo xtask art dex <text>` searches
what the catalogue says about them.

## Method

The catalogue holds 48,790 meshes across 115 packs. I searched it by name and by description for each of
the 41 foods, looked at the best candidates in each of the packs that turned up, and recorded the best
single mesh plus the other usable ones. Every id was then checked to exist in the catalogue, and the 41
`Food` variants were checked against `Food::ALL`. For Fantasy Village, Farm, Shops, Town, Fantasy
Kingdom, Kids and Icons I listed and read the food, kitchen and farm items. Every other pack was searched
by name and description only, so a lone good mesh in one of them could have been missed.
