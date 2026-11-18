# Windfall sound sources

Every sound here is a real recording from opengameart.org. Each source page lists **CC0** as its only license (checked 2026-11-18). No CC-BY or OGA-BY material is used, and no attribution is required. Authors are still credited below.

Common processing for every file:
- decoded and downmixed to mono at 44100 Hz
- silence trimmed
- band-limited with high-pass and low-pass filters, listed per file
- short fade-in and fade-out against clicks
- peak-normalised to -3 dBFS unless noted
- encoded with ffmpeg libvorbis at `-q:a 2` (the ambience is the exception, see below)
- metadata stripped

"Pitch shift" means tape-style resampling (`asetrate` + `aresample`), so pitch and duration change together.

| Output | Source page | Original file | Author | License | Processing |
|---|---|---|---|---|---|
| leaf_release_1.ogg | https://opengameart.org/content/20-rustles-dry-leaves | `qubodup-rustle.7z` → `rustle/rustle17.flac` | qubodup | CC0 | HP 120 Hz, LP 9 kHz, trimmed to 0.42 s, 4 ms fade-in, 160 ms fade-out, -3 dBFS |
| leaf_release_2.ogg | same page | `rustle/rustle09.flac` | qubodup | CC0 | same as leaf_release_1 |
| gust_1.ogg | https://opengameart.org/content/swish-bamboo-stick-weapon-swhoshes | `swoshes.7z` → `swosh-39.flac` | qubodup | CC0 | HP 200 Hz, LP 4.5 kHz (removes the sharp "weapon" edge), trimmed to 0.40 s, 50 ms fade-in, 180 ms fade-out, -3 dBFS |
| gust_2.ogg | same page | `swosh-21.flac` | qubodup | CC0 | same as gust_1, trimmed to 0.30 s |
| catch.ogg | https://opengameart.org/content/10-impactshield-blocks (a cardboard hit) **+** https://opengameart.org/content/20-rustles-dry-leaves | `impact_-_starninjas.zip` → `impact.6.ogg` + `rustle/rustle16.flac` | StarNinjas; qubodup | CC0; CC0 | Cardboard hit: HP 90 Hz, LP 3.5 kHz. Leaf rustle layered under it at -10 dB, 5 ms later. Cut to 0.33 s with fades, -3 dBFS |
| miss.ogg | https://opengameart.org/content/10-impactshield-blocks | `impact.4.ogg` | StarNinjas | CC0 | HP 60 Hz, LP 700 Hz twice (muffled), pitched down 3 semitones, 0.30 s, 100 ms fade-out, -3 dBFS |
| place.ogg | https://opengameart.org/content/100-cc0-metal-and-wood-sfx | `100-CC0-wood-metal-SFX.zip` → `wood_hit_03.ogg` | rubberduck | CC0 | HP 150 Hz, LP 8 kHz, 0.18 s, 60 ms fade-out, -3 dBFS |
| remove.ogg | https://opengameart.org/content/pop-sounds | `pop3.ogg` | cogitollc | CC0 | HP 120 Hz, LP 4 kHz, pitched down 1 semitone, about 0.08 s, normalised to **-6 dBFS** (quieter than place) |
| flip.ogg | https://opengameart.org/content/100-cc0-sfx | `100-CC0-SFX_0.zip` → `wooded_box_open.ogg` (wooden box latch, double clack) | rubberduck | CC0 | HP 180 Hz, LP 8 kHz, 0.29 s, 60 ms fade-out, -3 dBFS |
| mark.ogg | https://opengameart.org/content/bell-dingschimes | `bell_ding2.wav` | PWL | CC0 | HP 150 Hz, trimmed to 1.2 s, 500 ms fade-out, **-4 dBFS** |
| solved.ogg | https://opengameart.org/content/kalimba-two-notes | `kalimba.wav` | railkill | CC0 | Builds a four-note phrase G4 → A4 → D5 → G5 from the two recorded notes. The original G4→A4 plays first. The A4 note, pitch-shifted +5 and +10 semitones, is overlaid at 0.40 s and 0.58 s. HP 100 Hz, about 0.87 s, -3 dBFS |
| ambience.ogg | https://opengameart.org/content/park-ambiences | `park_ambience_birds.wav` (16 s to 42 s) + `park_ambience_wind.wav` (200 s to 226 s) | Thimras | CC0 | Birds bed: HP 120 Hz, LP 8 kHz. Wind layer: HP 250 Hz (removes mic buffeting), LP 6 kHz. Both RMS-matched, then mixed with the wind at 0.6x. 24 s loop: the 2 s past the loop end is equal-power crossfaded into the first 2 s, so the file loops seamlessly. Peak -14 dBFS. Encoded libvorbis managed bitrate 40 kb/s (min 32k, max 48k), because `-q:a -1` gave about 185 KB |

Raw downloads were kept outside the repo, in the session scratchpad (`sfx-src/`), along with the processing script (`tools/build.py`).
