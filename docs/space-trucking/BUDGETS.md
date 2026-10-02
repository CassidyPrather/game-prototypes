# Performance Budgets

[DESIGN.md](DESIGN.md) asks for profiling metrics to enforce, not merely observe. These
are them. Every budget is a hard gate somewhere in CI; a change that blows
one either shrinks or retunes the number **in this file, in the same
change**, with a sentence of why. Ceilings carry ~50–100× headroom over
measured values — they catch regressions in *kind* (an accidental O(n²), a
busy-wait, an asset creep), not percentage drift; `cargo bench` remains the
tool for watching drift.

| Budget | Ceiling | Measured | Enforced by |
| --- | --- | --- | --- |
| wasm payload (post-`wasm-opt`) | 1,500,000 bytes | ~1.19 MB (the whole menu bundle; Space Trucking alone was ~638 KB) | `ci-cd.yml` size step |
| Offline catch-up, 1 sim-hour | 250 ms | ~2 ms | `tests/space_trucking_perf.rs` (release) |
| 100k crew ticks × 6 players | 250 ms | ~13 ms | `tests/space_trucking_perf.rs` (release) |
| Convoy delivery voyage (6 replicas, hostile links) | 5,000 ms | ~100 ms | `tests/space_trucking_perf.rs` (release) |
| 1000 save round-trips | 500 ms | ~40 ms | `tests/space_trucking_perf.rs` (release) |
| Black box at cap: serialize / parse / replay 50k entries | 500 / 500 / 1000 ms | ~21 / 16 / 3 ms | `tests/space_trucking_perf.rs` (release) |
| Debug test suite wall time (`cargo test --lib space_trucking`) | keep under ~30 s | ~15 s (the earlier ~0.5 s figure no longer held when this was moved into game-prototypes; the standalone crate measured the same) | courtesy, not a gate |

Not yet enforced, deliberately: frame time (needs a headless GPU story —
revisit with the live-multiplayer slice) and load-to-first-frame (dominated
by browser wasm compile; the size budget is its proxy).

Run locally: `cargo test --release --test space_trucking_perf -- --ignored`
