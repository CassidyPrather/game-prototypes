//! Windfall's garden lab: each garden's par, its gentlest solution, and how
//! forgiving it is to random play. For designing gardens:
//!
//! ```bash
//! cargo run --release --example windfall_lab          # every garden
//! cargo run --release --example windfall_lab -- 4     # just the fifth
//! ```

// Garden coordinates are small and never negative here.
#![allow(clippy::cast_sign_loss)]

use game_prototypes::windfall::garden::{Cell, Dir, Hue, Level, Piece, Plan, Run};
use game_prototypes::windfall::levels::LEVELS;
use game_prototypes::windfall::solve;

const fn hue(h: Hue) -> char {
    match h {
        Hue::Ember => 'e',
        Hue::Gold => 'g',
        Hue::Rust => 'r',
    }
}

fn show(level: &Level, plan: &Plan) {
    let mut top = vec!['.'; level.width as usize];
    let mut seq = String::new();
    for d in level.drops {
        top[d.x as usize] = '|';
        seq.push(hue(d.hue));
    }
    println!("    {}   drops {seq}", top.iter().collect::<String>());
    for y in 0..level.height {
        let row: String = (0..level.width)
            .map(|x| {
                let at = Cell::new(x, y);
                let fixed = level.fixed.iter().find(|(c, _)| *c == at).map(|(_, p)| *p);
                match (plan.get(at), fixed) {
                    (Some(p), _) => glyph(p, false),
                    (None, Some(p)) => glyph(p, true),
                    (None, None) if level.is_stone(at) => '#',
                    _ => '.',
                }
            })
            .collect();
        println!("    {row}");
    }
    let ground: String = (0..level.width)
        .map(|x| {
            level
                .basket(x)
                .map_or('_', |b| hue(b.hue).to_ascii_uppercase())
        })
        .collect();
    println!("    {ground}");
}

const fn glyph(p: Piece, fixed: bool) -> char {
    match (p, fixed) {
        (Piece::Vane(Dir::Right), false) => '>',
        (Piece::Vane(Dir::Left), false) => '<',
        (Piece::Flip(Dir::Right), false) => ')',
        (Piece::Flip(Dir::Left), false) => '(',
        (Piece::Vane(Dir::Right), true) => '}',
        (Piece::Vane(Dir::Left), true) => '{',
        (Piece::Flip(Dir::Right), true) => ']',
        (Piece::Flip(Dir::Left), true) => '[',
    }
}

fn main() {
    let only: Option<usize> = std::env::args().nth(1).and_then(|a| a.parse().ok());
    let quick = std::env::args().any(|a| a == "--quick");
    for (n, level) in LEVELS.iter().enumerate() {
        if only.is_some_and(|o| o != n) {
            continue;
        }
        println!(
            "garden {n}: {}x{}, {} leaves",
            level.width,
            level.height,
            level.drops.len()
        );
        let started = std::time::Instant::now();
        let Some(plan) = solve::fewest(level, 8) else {
            println!("  no solution within 8 pieces!\n");
            continue;
        };
        let mut run = Run::new(level, &plan);
        run.finish();
        println!(
            "  fewest: {} pieces, {} gusts (par says {}/{})",
            plan.count(),
            run.gusts(),
            level.par_pieces,
            level.par_gusts
        );
        println!("  ({:.2?} to prove)", started.elapsed());
        show(level, &plan);
        if quick {
            continue;
        }
        let limit = plan.count() + usize::from(plan.count() < 6);
        if let Some((gentle, gusts)) = solve::gentlest(level, limit) {
            println!(
                "  gentlest within {limit} pieces: {gusts} gusts, {} pieces",
                gentle.count()
            );
            show(level, &gentle);
        }
        for extra in 0..3 {
            let k = plan.count() + extra;
            println!(
                "  luck with {k} random pieces: {:.4}",
                solve::luck(level, k, 20_000, 7)
            );
        }
        println!();
    }
}
