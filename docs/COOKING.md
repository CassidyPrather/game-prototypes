# The cooking module

A small cooking system you can copy into another game: foods, processes,
recipes, a station that cooks one into the other, a stock to count them in,
and a bill that says what anything costs from the ground up. It is meant as
a starting point for a resource pipeline that wants tuning, not a finished
design. [Kitchen Garden](KITCHEN_GARDEN.md) is the demo built on it.

It lives in `src/cooking.rs` and `src/cooking/`. It uses nothing but `std`,
and its files only refer to each other through `super::`, so installing it
elsewhere is a copy:

```bash
cp src/cooking.rs   ../my-game/src/cooking.rs
cp -r src/cooking   ../my-game/src/cooking
# then `pub mod cooking;` (or `mod cooking;`) in your crate root
```

`the_module_can_be_copied_out_whole` (in `cooking.rs`) fails the build if any
file in it starts reaching outside, so it stays copyable. The tests come
along with it and run in the new home.

## The pieces

| File | What it is |
| --- | --- |
| `food.rs` | `Food`: one flat enum of every food. Each has a `Stage` (raw, prepared, dish, waste) and some `Group`s (fluid base, animal produce, crop, grain, vegetable, seasoning, mineral, sweet, drink, baked). |
| `process.rs` | `Process`: what can be done to food, each of one `Kind`. |
| `recipe.rs` | `RECIPES`: the table. Inputs (a multiset), process, output, and how much work. Plus lookups: `find`, `making`, `using`, `toward`, `missing`. |
| `station.rs` | `Station`: the state machine. |
| `stock.rs` | `Stock`: a count of every food. |
| `bill.rs` | `bill(food)`: raw ingredients, strokes, heat, waiting and steps, all the way down. |

### Foods

| Stage | Foods |
| --- | --- |
| Raw | water, milk, egg, carrot, onion, tomato, wheat, rice, beans, sugar, tea leaf, spice, salt |
| Prepared | flour, dough, risen dough, batter, butter, hot water, chopped carrot, chopped onion, chopped tomato, soaked beans |
| Dish | tea, milk tea, salad, boiled egg, steamed rice, soup, bean stew, custard, rice pudding, fried egg, omelette, pancake, fried rice, flatbread, bread, cake |
| Waste | mush (what any process makes of inputs no recipe wants), charcoal (what heat makes of anything left on it) |

Milk is both a fluid base and animal produce; salt is a mineral seasoning;
wheat is a grain crop. A raw food is one no recipe makes, so it is the
game's job to supply it.

### Processes, in three kinds

| Kind | Processes | Driven by | Once done |
| --- | --- | --- | --- |
| Physical | chop, grind, churn, knead, mix | `Station::work`, one stroke each | waits |
| Heat | boil, fry, bake | `Station::tick`, one tick each | keeps cooking: smokes, then burns to charcoal |
| Time | prove, soak, steep | `Station::tick` | waits |

That table is the whole design tension. Physical work costs the cook's
attention; heat costs nothing until it costs everything; time is free but
slow.

### Recipes

```text
Chop   carrot                          -> chopped carrot      4 strokes
Grind  wheat                           -> flour               8 strokes
Knead  flour + water                   -> dough               8 strokes
Mix    flour + egg + milk              -> batter              6 strokes
Prove  dough                           -> risen dough        20 s
Steep  hot water + tea leaf            -> tea                 6 s
Boil   water                           -> hot water           5 s
Boil   water + chopped carrot
       + chopped onion + salt          -> soup               15 s
Bake   risen dough                     -> bread              14 s
...
```

The full table is `RECIPES` in `src/cooking/recipe.rs`, 26 rows. Inputs
are unordered and duplicates count. Each recipe makes one of its output,
so bills are always whole numbers. Its tests keep it well-formed:

- every prepared food and dish has a recipe, and nothing else does;
- every raw and prepared food is used by something;
- no two recipes of one process take the same inputs;
- nothing needs itself or waste.

Time is in integer ticks, so a fixed-step game stays deterministic.
`TICKS_PER_SEC` (20) is only the scale the table is written in. Change it
and every duration rescales.

### The station

```text
            put / take                 work or tick            take
  ┌──────┐ ─────────► ┌──────┐  start  ┌─────────┐  done  ┌───────┐ ──► Idle
  │ Idle │            │ Idle │ ──────► │ Working │ ─────► │ Ready │
  └──────┘ ◄───────── └──────┘         └─────────┘        └───────┘
  (empty)               (loaded)        abort ──► Idle      heat: burns
```

```rust
use cooking::{Food, Process, Station};

let mut pot = Station::new(&[Process::Boil]);
pot.put(Food::Water)?;
pot.put(Food::Egg)?;
pot.start()?;                 // a clock station needs starting
for _ in 0..cooking::secs(8) {
    pot.tick();               // gate this on fuel, power, pause...
}
assert_eq!(pot.take()?, Food::BoiledEgg);

let mut board = Station::new(&[Process::Chop]);
board.put(Food::Onion)?;
for _ in 0..4 {
    board.work()?;            // the first stroke starts it
}
assert_eq!(board.take()?, Food::ChoppedOnion);
```

How it behaves:

- **More than one process.** A station can run several processes that share
  a drive (a bowl that kneads, mixes and churns). When it starts, it picks the
  first process whose recipe matches its contents. If none matches, the
  first process makes mush. `Station::plan` says in advance which it will
  be. `Station::toward` lists the recipes a half-filled station is on the
  way to, and `recipe::missing` says what each still needs: that is enough
  to show the player a hint.
- **Capacity** is derived: the largest recipe any of its processes takes.
- **Idle** stations give back what was put in, last first. A working station
  is locked. `abort` gives the contents back unchanged.
- **Heat** dishes have a grace period per process. `Event::Smoking` comes
  halfway through it, then the dish becomes charcoal and `Event::Burnt`
  fires.
- **Clock stations only move when ticked.** The station has no notion of
  fuel or power. Whatever gates it is the caller's: don't tick it.
- **Events.** Everything that happens is queued as an `Event` until
  `drain`. Use them for sound and motion. They are not part of the logic.

### The bill

```rust
let b = cooking::bill(Food::Bread).unwrap();
// b.raw    == one wheat, one water
// b.steps  == 4   (grind, knead, prove, bake)
// b.effort == 16 strokes, b.time == 20 s, b.heat == 14 s
let worth = b.raw_value(|f| my_prices[f.index()]) + b.steps;
```

`bill` follows the first recipe for each food back to raw ingredients. Use
it wherever a number should follow from the recipe rather than be set by
hand: a price, a customer's patience, an XP reward, a crafting tier. Kitchen
Garden prices every dish this way (`src/kitchen_garden/prices.rs`), so
making a recipe longer reprices it automatically.

## Changing it

- **A new food:** add the variant to `Food` and to `Food::ALL` (in order),
  then give it a `stage` and `groups`. If it is made, add a recipe; if it is
  raw, use it in one. The tests say what you missed.
- **A new process:** add it to `Process` and `Process::ALL`, give it a
  `kind`, and a `grace` if it is heat.
- **Retuning:** edit the `work` column of `RECIPES`, `Process::grace`, or
  `Process::mush_work`.
- **Quality, spoilage, multiple outputs:** deliberately left out. Quality
  and spoilage can live beside a `Food` in your own item type, and the
  station never needs to know. For multiple outputs, give the recipe an
  output count and divide the bill.
