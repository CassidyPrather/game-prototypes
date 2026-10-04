# Kitchen Garden

One day of cooking from a garden, for whoever comes to the hatch.

Pick it from the [main menu](../README.md); `Escape` goes back there.

It is the demo of the reusable [cooking module](COOKING.md), and the case for
it. The module only knows foods, processes, recipes and stations. This
prototype adds what a game would put around them: somewhere for raw
ingredients to come from, at a cost; somewhere for dishes to go, for a
price; and never quite enough time, room, wood, coins or hands.

## The day

The day lasts four minutes, and the sun crosses the sky to show it.
Villagers come to the hatch on the right, each wanting one dish. A bubble
shows the dish, its price in coins, and a ring of patience that drains.
Serve them before it runs out. If they are still more than half patient,
they tip a coin. Early customers ask for simple things (a fried egg, tea,
steamed rice). By the afternoon they want bread, cake, bean stew. The jar in
the corner fills with coins, and its three notches are the marks: 30, 55 and
80 coins at dusk.

### Where food comes from

| Source | Gives | Costs |
| --- | --- | --- |
| Six garden plots | carrot, onion, tomato, beans, wheat, rice, tea leaf | one of the crop as seed, and 16–30 s to grow; three back |
| The hen | eggs | waiting; one every 15 s, two at most in the nest |
| The cow | milk | four pulls each; her udder holds three and refills slowly |
| The well | water | three turns of the crank a bucket |
| The market | salt, sugar, spice | coins, which are also your score |

The hearth needs wood: four strokes of the axe a log. A log burns 14 s
whenever anything is on the hearth, cooking or cooked and not taken off. One
fire heats the pot, the pan and the oven together.

The compost heap takes anything (mush, charcoal, the surplus) and enriches
the next planting with one extra crop.

### The kitchen

| Station | Does | Kind |
| --- | --- | --- |
| Board | chop | hold the button: work |
| Quern | grind | work |
| Bowl | knead, mix, churn | work |
| Crock | prove, soak, steep | click to start: it waits for you |
| Pot | boil | click to start: on the fire, and it burns |
| Pan | fry | fire, burns fastest |
| Oven | bake | fire |

Each station has a badge showing its kind: a hand, a flame or an hourglass.
The badge is ringed with progress, and the ring turns red as a finished
dish nears burning. Load a station and a hint appears above it: the
process, then what it will make. If the contents are not a recipe yet, it
shows what they are on the way to and what is missing. If they never will
be one, it shows mush. Rest the pointer on any food (on a shelf, in a
bubble, on a station, at the market) and a card shows how it is made: the
inputs, the process, the result.

### Opportunity costs

This is the reason for the demo. Everything above is cheap on its own; the
game is that you can't do it all at once.

- **One pair of hands.** Physical work (chopping, grinding, cranking,
  milking, splitting wood) needs the button held down. While you hold it,
  the pan can burn.
- **Seed or supper.** Planting uses a crop you could have cooked. Eat the
  last of your rice and there's no more rice today.
- **Which plots.** Six plots, seven crops, different growing times. Slow
  wheat makes the best-paying bread; tea is quick and cheap.
- **Coins are the score.** Sugar for a cake is two coins that won't be in
  the jar at dusk. The cake pays them back several times over, if you
  finish it.
- **Wood.** Batching dishes on one fire saves logs. Leaving one egg on all
  afternoon wastes them. Every log is time at the stump.
- **Eight shelves of four.** The pantry fills up. Harvest everything and
  there's no room for what you cook.
- **Whom to serve.** Three at the hatch, each on a clock. A quick tea now,
  or a stew for the one who pays three times as much?

### Prices are derived, not set

No dish has a hand-set price or patience. `src/kitchen_garden/prices.rs`
computes both from the cooking module's `bill`:

- price = the raw ingredients' value + one coin per process step;
- patience = 40 s + the dish's own cooking and waiting time + half a second
  per stroke of work.

Lengthen a recipe in `src/cooking/recipe.rs` and it reprices itself, and its
customers wait longer for it. That is the tuning hook the module exists to
provide.

### No text

As elsewhere in this repo, there's no writing on screen. Prices are coins,
patience is a ring, the time is the sun, the score is a jar with three
notches, and recipes are cards of pictures. The title card teaches the three
kinds of process in one picture each: a carrot chopped (hand), an egg fried
and then burnt (flame), dough rising (hourglass).

## Controls

| Input | Action |
| --- | --- |
| Drag food | Move it: shelf, station, plot (to plant), customer (to serve), compost |
| Drop food on nothing | Put it in the pantry |
| Click a ripe plot, the nest, the pail, the bucket | Gather it all into the pantry |
| Click a market ware | Buy one |
| Click a loaded pot, pan, oven or crock | Start it |
| Hold the button on a loaded board, quern or bowl | Work it |
| Hold the button on the cow, the well, the stump | Milk, crank, split wood |
| Click a finished station | Put the result in the pantry |
| `M` | Mute |
| `R`, `Enter` or click | Start a new day, once the day is over |
| `Escape` | Back to the menu |

## How it is built

The library side is in `src/cooking/` and `src/kitchen_garden/`; the
macroquad side is in `src/games/kitchen_garden/`.

- `src/cooking/` — the reusable module ([its own doc](COOKING.md)).
- `src/kitchen_garden/day.rs` — the rules, as a deterministic state machine.
  - Commands: `move_food`, `stow_all`, `work`, `start`, `chore`.
  - `tick` at 20 Hz; `Cue`s out.
  - Food is only ever in a `Place`. `move_food` checks both ends before
    touching either, so a refused drag loses nothing.
  - Its `tests.rs` has a monkey that hammers it with nonsense, and a bot.
- `src/kitchen_garden/prices.rs` — prices and patience, from bills.
- `src/kitchen_garden/sfx.rs` — every sound, synthesised in one G major
  pentatonic, with two loops (the garden, the hearth). Tests check every
  sound decodes, starts and ends silently, and that a busy moment cannot
  clip.
- `src/games/kitchen_garden.rs` — the pointer and the cues.
  - Pointer: pressing, holding to work, dragging, dropping, clicking.
  - Cues become sound and motion. `layout` holds every rectangle, shared by
    drawing and hit-testing.
- `src/games/kitchen_garden/pen.rs` — soft vector shapes in frame units, and
  the palette.
- `src/games/kitchen_garden/icons.rs` — a drawing for every food, process and
  kind.
- `src/games/kitchen_garden/draw.rs` — the scene, the hint bubbles, recipe
  cards, title and dusk summary.
- `src/games/kitchen_garden/fx.rs` — particles and flying food (cosmetic).
- `src/games/kitchen_garden/audio.rs` — the sound bank and the loops.

### Knobs

- **The day** (`day.rs`): `DAY`, `LAST_ORDERS`, `PANTRY`, `STACK`, `PLOTS`,
  `SEATS`, `LOG_BURN`, the chore stroke counts, `LAY`, `UDDER_REFILL`,
  `START_COINS`, `MARKS`, `crop()` and the starting pantry in `Day::new`.
- **Prices** (`prices.rs`): raw values, market prices, `TIP`, the patience
  formula.
- **Recipes** (`src/cooking/recipe.rs`).

The balance check is the bot in `day/tests.rs`: a careful cook who works one
order at a time and keeps the fire fed and the garden planted. It must reach
the first mark and not the third, and a cook who does nothing must earn
nothing. `cargo test --lib kitchen_garden::day -- --nocapture` prints its
days: about 35–65 coins over eight seeds.

## Ideas not yet tried

- Spoilage: milk that sours on the shelf, so the pantry is a clock too.
- Regulars who come back for what they liked, and pay more for it.
- A recipe book that fills in as you discover dishes, instead of every card
  being known.
- Weather that changes what grows fast today.
