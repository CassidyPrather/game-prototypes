//! Small Hours' workbench: read nights from a text file and say how few
//! keepers each needs, with one way of doing it.
//!
//! ```text
//! cargo run --release --example small_hours_lab -- nights.txt
//! ```
//!
//! The file is blocks separated by blank lines. A block's first line is
//! `hours embers`, and the rest is the map as `small_hours::nights` writes
//! it. With no file, the game's own nights are checked instead.

use game_prototypes::small_hours::night::{Level, MAX_KEEPERS, Step};
use game_prototypes::small_hours::{nights, solve};

fn main() {
    let levels: Vec<Level> = std::env::args().nth(1).map_or_else(
        || nights::NIGHTS.to_vec(),
        |path| {
            let text = std::fs::read_to_string(path).expect("a readable file");
            text.split("\n\n")
                .filter(|block| !block.trim().is_empty())
                .map(parse)
                .collect()
        },
    );
    if std::env::var_os("SUMMARY").is_some() {
        // One line per night: index, par, gentle par, luck at par.
        for (i, level) in levels.iter().enumerate() {
            let par = solve::fewest(level, 3);
            let gentle = (1..=3).find(|&k| solve::solve_as(level, k, true).is_some());
            let luck = par.map_or(0.0, |k| solve::luck(level, k, 4000, 7) * 100.0);
            println!(
                "{i} par={} gentle={} luck={luck:.1}",
                par.unwrap_or(0),
                gentle.unwrap_or(0)
            );
        }
        return;
    }
    for (i, level) in levels.iter().enumerate() {
        println!(
            "night {} — {} hours, {} embers",
            i + 1,
            level.hours,
            level.embers
        );
        for row in level.rows {
            println!("  {row}");
        }
        if game_prototypes::small_hours::night::kept(level, &[Vec::new()]) {
            println!("  keeps itself: one keeper standing still is enough");
        }
        let mut found = false;
        for keepers in 1..MAX_KEEPERS {
            let started = std::time::Instant::now();
            let gentle = solve::solve_as(level, keepers, true);
            let solved = gentle.clone().or_else(|| solve::solve(level, keepers));
            let took = started.elapsed().as_secs_f32();
            match solved {
                Some(tapes) => {
                    let written: Vec<String> = tapes
                        .iter()
                        .map(|tape| tape.iter().map(|s| Step::letter(*s)).collect())
                        .collect();
                    let luck = solve::luck(level, keepers, 20_000, 7) * 100.0;
                    let how = if gentle.is_some() {
                        "gently"
                    } else {
                        "at a cost"
                    };
                    println!(
                        "  {keepers}: {} — {how}, luck {luck:.1}% ({took:.2}s)",
                        written.join(" / ")
                    );
                    if std::env::var_os("FILM").is_some() {
                        film(level, &tapes);
                    }
                    found = true;
                    break;
                }
                None => println!("  {keepers}: no ({took:.2}s)"),
            }
        }
        if !found {
            println!("  needs every bedroll, or cannot be kept");
        }
        println!();
    }
}

fn parse(block: &str) -> Level {
    let mut lines = block.lines().filter(|l| !l.trim().is_empty());
    let mut head = lines.next().expect("a header").split_whitespace();
    let hours = head.next().expect("hours").parse().expect("a number");
    let embers = head.next().expect("embers").parse().expect("a number");
    let rows: Vec<&'static str> = lines
        .map(|line| &*Box::leak(line.trim().to_owned().into_boxed_str()))
        .collect();
    Level {
        rows: Box::leak(rows.into_boxed_slice()),
        hours,
        embers,
        par: 0,
    }
}

/// Print the night hour by hour: `F` hearth, `#` tree, digits lit keepers,
/// `x` a snuffed one, `s` shades.
fn film(level: &Level, tapes: &[Vec<Step>]) {
    use game_prototypes::small_hours::night::{Cell, Ground, World};
    let ground = Ground::parse(level.rows);
    let mut world = World::dusk(&ground, level, tapes.len());
    let mut frames: Vec<Vec<String>> = Vec::new();
    for hour in 0..=usize::from(level.hours) {
        let mut rows = Vec::new();
        for y in 0..ground.height {
            let mut row = String::new();
            for x in 0..ground.width {
                let at = Cell::new(x, y);
                let keeper = world.keepers.iter().rposition(|k| k.at == at);
                row.push(if world.shades.contains(&Some(at)) {
                    's'
                } else if at == ground.hearth {
                    if world.lit() { 'F' } else { 'f' }
                } else if let Some(k) = keeper {
                    if world.keepers[k].lit {
                        char::from(b'1' + u8::try_from(k).expect("few keepers"))
                    } else {
                        'x'
                    }
                } else if ground.tree(at) {
                    '#'
                } else {
                    '.'
                });
            }
            rows.push(row);
        }
        frames.push(rows);
        if hour < usize::from(level.hours) {
            let steps: Vec<Step> = tapes
                .iter()
                .map(|t| t.get(hour).copied().unwrap_or(Step::Wait))
                .collect();
            world.advance(&ground, &steps, &mut Vec::new());
        }
    }
    for y in 0..frames[0].len() {
        let line: Vec<&str> = frames.iter().map(|f| f[y].as_str()).collect();
        println!("    {}", line.join("  "));
    }
}
