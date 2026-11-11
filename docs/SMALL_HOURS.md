# Small Hours

One fire, one night, and nobody to keep it but yourself, again.

Pick it from the [main menu](../README.md); `Escape` goes back there.

A turn-based game on a small grid, eight short nights long. Shades come out
of the dark for the hearth. You cannot hurt them. All you can do is walk,
and be the nearer light. When the hearth goes out anyway, the night runs
backward and you live it again beside the keeper you just were.

## The rules

There are three things on the board, and one rule.

**A shade walks toward the nearest light and puts out what it touches.**

- The **hearth** is a light. A shade that reaches it takes an ember and is
  gone. With no embers left the hearth is out.
- A **keeper** (you) is a light. A shade that reaches one puts its lantern
  out and carries on to the next nearest light.
- **Shades** are never spent, hurt or blocked by anything but a tree and
  each other. Dawn gets rid of them.

Everyone moves one square an hour, never diagonally. The keepers move
first, then the shades. A night is five to nine hours. If the hearth is lit
at dawn, the night is kept.

The fine print, all of it:

- Distance is counted in steps. A keeper wins a tie with the hearth, and
  the newest keeper wins a tie with an older one.
- A shade steps along the longer side of the way to its light first (east
  or west on a tie), and along the other side if a tree or another shade
  is in the way. With a tree dead ahead on a straight line it stops, and
  stays stopped until some light gives it a reason to step aside.
- Keepers walk through each other. Shades do not.

### Again

One keeper is rarely enough. When dawn comes on a cold hearth, the night
runs backward to dusk and a second keeper wakes on the next bedroll. The
first one walks exactly the steps you gave it, and you play the second. Up
to four.

A remembered keeper replays its *steps*, not its results. If the shade it
was leading now prefers you, it walks its route alone. If a shade it never
met now comes for it, it walks into trouble without looking up.

A keeper whose lantern is out goes on as a ghost. Nothing sees it and it
touches nothing, but its steps are still recorded. Save it in a later
telling and it walks where its ghost walked.

### Marks

Three, at a kept dawn, each for something different:

1. the hearth is lit;
2. with as few keepers as the night can be kept with;
3. and no lantern went out.

Every night can earn all three at once. Nothing is unlocked by them and
every night is open from the start.

## Reading the screen

There is no text. The ground is painted by whose light reaches it first:
orange for the hearth, and each keeper's own colour. A shade is after
whatever colour it is standing on, and its eyes turn that colour. A line of
dots runs from each shade to its light, with an arrowhead on the step it
means to take if nobody moves; a shade with no arrowhead is stuck. A
remembered keeper shows the step it is about to take (a dot for a wait).

Along the top, a moon crosses a ruler of hours toward the sun, and the
pips on the left are the nights, with the most marks each has earned.
Bottom left are the keepers as lanterns: lit, out, or not yet woken.

| Key | Action |
| --- | --- |
| Arrows, WASD | Step |
| Space | Wait an hour |
| Z, Backspace | Take an hour back. At dusk, this goes back into the keeper before |
| R | Start the night over with one keeper |
| Enter | Sleep through to dawn; at dawn, go on |
| 1–8 | Go to that night |
| M | Mute |

With a pointer: click a square to step toward it, click yourself to wait,
and click the key caps or the night pips.

## The nights

Hand-made, in `src/small_hours/nights.rs`. Each is one thing to notice.

| Night | Shades | Hours | Embers | Par | What it is for |
| --- | --- | --- | --- | --- | --- |
| 1 | 1 | 6 | 1 | 1 | It comes for the nearest light. Be that, somewhere else |
| 2 | 2 | 6 | 1 | 2 | Opposite corners: one keeper is not enough, so, again |
| 3 | 3 | 7 | 1 | 1 | A crowd follows one lantern as well as one shade does |
| 4 | 3 | 7 | 2 | 1 | The hearth can take one. Which one? |
| 5 | 2 | 7 | 1 | 1 | A shade behind a tree is no trouble until it is woken |
| 6 | 4 | 8 | 1 | 1 | The same, with less room to avoid waking it |
| 7 | 5 | 8 | 2 | 2 | Too wide a front for one |
| 8 | 6 | 9 | 1 | 3 | Everything, from everywhere |

Par is proven, not estimated. `src/small_hours/solve.rs` searches every
combination of steps for a given number of keepers, and the tests fail if
a night can be kept under its par, cannot be kept at it without losing a
lantern, or is kept by keepers who stand still.

## Why it is like this

The reasoning leans on Cassidy's own writing at
[wirenook.net](https://wirenook.net/), and these posts in particular:

- **[Exploring the October 2026 Inflection](https://wirenook.net/blog/october-2026-inflection/)**:
  "do not present too much at once" and "the onboarding itself should be
  part of the fun", and the complaint about being bombarded with item
  types. So there are three kinds of thing on the board, one rule, no
  tutorial, and each night adds one idea. The first night is a level, not
  a lesson.
- **The [Sand Nomad](https://wirenook.net/blog/reflections/sand-nomad/)
  reflection**: zero onboarding, too much information at once, a high
  volume of choices that do not matter. Here a turn is one of five
  choices and every piece of information on screen is something the one
  rule uses.
- **The [AmoebaRL](https://wirenook.net/blog/reflections/amoeba/)
  reflection**: "Smaller numbers! I love small numbers", tactical
  decisions that do not matter because the winning strategy is scaling
  in a corner, and overstaying its welcome. Nothing here scales: no
  upgrades, no numbers above nine, and the whole game is eight nights.
- **Reviews.** Into the Breach (10/10) for perfect information and
  telegraphed intent; Quantum League and Lemnis Gate for playing beside
  your own earlier turns, ghost recording included; Opus Magnum for
  puzzles that are easy to finish and scored on more than one thing;
  Unferat for "one core idea executed very well"; Pathologic and Overland
  for a game about not being able to fight back.
- **[Agency vs. Replayability](https://wirenook.net/blog/agency-vs-replayability/)**:
  nothing is gated behind marks, so there is no reason to replay a night
  except wanting to do it better.

## What is not known

Nobody has played this but a solver and a headless browser. The solver
says every night is fair. It cannot say whether one is fun.

Two things to look at first if it is not:

- **The sidestep.** A shade diagonally behind a keeper slides east or west
  before it closes in, so a keeper who shuffles east and west can hold a
  shade in the next row forever. It falls out of the stepping rule above
  and is a legitimate technique, like pillar dancing in a roguelike, but
  it may be too good. Having a shade prefer the side its light did *not*
  just move along is the first thing to try against it; that is untested.
- **Reading a second telling.** A remembered keeper's plan was made in a
  world without you in it. The dots and arrowheads show what each shade
  wants now; they do not show what it wanted last time.

A harsher version, where shades cut corners and a keeper is caught within
a few hours, was tried and thrown out. It made every night cost lanterns
and left almost no room to learn.

## Knobs and tools

- `nights::NIGHTS`: the maps, hours, embers and par.
- `night::MAX_KEEPERS`: bedrolls per night.
- `World::nearest_light` and `World::creep`: the two rules. Change either
  and every par must be re-proven.

```bash
cargo test --lib small_hours                            # rules, nights, sounds
cargo run --release --example small_hours_lab           # each night's par and a way to keep it
FILM=1 cargo run --release --example small_hours_lab    # ...drawn out hour by hour
cargo run --release --example small_hours_lab -- f.txt  # the same for nights in a file
```

The lab also prints *luck*: how often keepers stepping at random keep the
night. About a third for the first night and under one in a hundred for
the last.

## Sound

Synthesised in `src/small_hours/sfx.rs`. No music. Everything pitched is
in C sharp minor pentatonic, the menu's E major pentatonic from its dark
end, so dawn resolves to E. Each keeper steps on its own note, so four
stepping together is a chord.
