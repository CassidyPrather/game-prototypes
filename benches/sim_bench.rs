//! Criterion benchmark over the sim and the sequencer.

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};

use game_prototypes::leitmotif::sim::{InputFrame, Sim, TICK_DT, Vec2};
use game_prototypes::leitmotif::song::Sequencer;

/// A sim past the title screen and into the full arrangement, where every
/// layer is sounding and the arena is at its busiest.
fn busy_sim() -> Sim {
    let mut sim = Sim::new(0x5EED);
    sim.advance(
        0.0,
        &InputFrame {
            start: true,
            ..InputFrame::default()
        },
    );
    while sim.music().section < 2 {
        sim.advance(TICK_DT, &InputFrame::default());
    }
    sim
}

fn bench_sim_tick(c: &mut Criterion) {
    let input = InputFrame {
        move_dir: Vec2::new(0.7, -0.7),
        ..InputFrame::default()
    };
    let mut sim = busy_sim();
    // Exactly one tick per iteration: the accumulator lands back on zero
    c.bench_function("sim_tick", |b| {
        b.iter(|| sim.advance(black_box(TICK_DT), black_box(&input)));
    });
}

fn bench_sequencer_tick(c: &mut Criterion) {
    let mut seq = Sequencer::new();
    let mut events = Vec::new();
    c.bench_function("sequencer_tick", |b| {
        b.iter(|| {
            events.clear();
            seq.advance(black_box(TICK_DT), &mut events);
            events.len()
        });
    });
}

criterion_group!(benches, bench_sim_tick, bench_sequencer_tick);
criterion_main!(benches);
