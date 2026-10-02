# Credits

Third-party and vendored assets. The project's own code is AGPL-3.0-or-later;
see [LICENSE](LICENSE). This file tracks everything that came from somewhere
else.

**The convention:** every asset gets a line here at intake — name, source URL,
author, license, date added. Art, audio, fonts, shaders, vendored js, all of
it. Thirty seconds now versus archaeology later.

| Asset | Source | Author | License | Added |
| --- | --- | --- | --- | --- |
| `web/gl.js` | miniquad's wasm loader, vendored from the miniquad 0.4.10 crate — <https://github.com/not-fl3/miniquad> | not-fl3 and contributors | MIT OR Apache-2.0 | 2026-07 |
| `assets/fonts/AtkinsonHyperlegible-Bold.ttf` | Atkinson Hyperlegible Bold 1.006, as served by the wirenook.net style guide — <https://www.brailleinstitute.org/freefont/>; licence text in `assets/fonts/OFL.txt` | Braille Institute of America (Elliott Scott, Megan Eiswerth, Linus Boman, Theodore Petrosky) | OFL-1.1 | 2026-09 |
| Sand Nomad's motive marks and emotes (`src/sand_nomad/art.rs`) | Redrawn as pixel art from Cassidy's motif package: the six motive suits (chalice, club, heart, spike, diamond, spade), their hexagon, the hex emotes and the anima ring | Cassidy Prather | The author's own designs, used in this project with permission | 2026-09 |
| `web/audio.js` | quad-snd's miniquad audio plugin, vendored unmodified from the quad-snd 0.2.8 crate (`js/audio.js`) — <https://github.com/not-fl3/quad-snd> | not-fl3 and contributors | MIT OR Apache-2.0 | 2026-07 |
| `web/sapp_jsutils.js` | sapp-jsutils' miniquad plugin, vendored unmodified from the sapp-jsutils 0.1.7 crate (`js/sapp_jsutils.js`) — <https://github.com/not-fl3/sapp-jsutils> | not-fl3 and contributors | MIT OR Apache-2.0 | 2026-08 |
| `web/quad-storage.js` | quad-storage's miniquad plugin, vendored unmodified from the upstream repo's `js/quad-storage.js` at the quad-storage 0.1.3 release commit (`3760b95`; the published crate ships no js) — <https://github.com/optozorax/quad-storage> | Ilya Sheprut (optozorax) | MIT OR Apache-2.0 | 2026-08 |

The js files are copied verbatim so refreshing them after a `cargo update` is
a straight `cp` out of `~/.cargo/registry/src/*/<crate>/` — except
`web/quad-storage.js`, which the published crate omits, so it comes from the
upstream git repo at the tag-equivalent commit recorded in the crate's
`.cargo_vcs_info.json`. Upstream `audio.js` logs `"fix"` to the console on the
first click and builds one throwaway `AudioContext` while it is at it; that is
theirs, left alone deliberately. Upstream `quad-storage.js` registers plugin
version `"0.1.2"` while the quad-storage-sys 0.1.0 crate reports its version
as a packed integer (65536), so gl.js logs a harmless version-mismatch
`console.error` at startup; also theirs, also left alone. (`sapp_jsutils.js`
and `quad-storage.js` are Space Trucking's; the other prototypes do not use
them.)

The sound effects themselves need no credit line — `src/leitmotif/synth.rs`,
`src/shell/sfx.rs`, `src/sand_nomad/sfx.rs` and `src/space_trucking/synth.rs`
generate them from arithmetic at startup, so there are no audio assets to
track. Sand Nomad's pixel art is
drawn by hand as data in `src/sand_nomad/art.rs`; the only borrowed part is
the motif designs credited above.
