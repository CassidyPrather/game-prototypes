//! Space Trucking's frontend: input in, the ship console out.
//!
//! Everything that decides what happens lives in the library's
//! `space_trucking::sim`. This file only translates between the window and
//! the sim's logical world: it gathers an [`InputFrame`] each frame (folding
//! the console's icon buttons into the pause/warp/mute toggles), advances
//! the sim, and hands the result to [`render`] and [`audio`]. It also owns
//! the save slot — load-and-catch-up on startup, cue-driven autosave
//! forever after — the flight recorder's black box, the opt-in telemetry
//! buffer behind the web shell's consent card
//! (`docs/space-trucking/TELEMETRY.md`), and the game's one and only piece
//! of text, the version string.
//!
//! Run natively with `--replay <file>` (the shell's own command line, which
//! skips the menu) to watch a recorded session play itself back, and with
//! `--dev` to unlock fast-forward.
//!
//! The ship keeps flying while the player is at the menu: nothing here
//! updates then, but the wall clock does not stop, and the first frame back
//! replays the difference the same way a backgrounded tab does.

mod audio;
mod emblem;
mod juice;
mod palette;
mod render;
mod storage;
mod tutor;

use audio::Audio;
use game_prototypes::VERSION;
use game_prototypes::space_trucking::replay::Recording;
use game_prototypes::space_trucking::sim::{
    Cue, InputFrame, Sim, TICK_DT, Vec2, WARP_FACTOR, WORLD_H, WORLD_W, layout,
};
use game_prototypes::space_trucking::telemetry::{Aggregate, BUFFER_KEY, CONSENT_KEY, Snapshot};
use juice::Juice;
use macroquad::color::Color;
use macroquad::input::{
    KeyCode, MouseButton, is_key_down, is_key_pressed, is_mouse_button_down,
    is_mouse_button_pressed, is_mouse_button_released, mouse_position,
};
use macroquad::text::{draw_text, measure_text};
use macroquad::texture::{FilterMode, RenderTarget, RenderTargetParams, render_target_ex};
use macroquad::time::get_frame_time;
use macroquad::window::{next_frame, screen_height, screen_width};
use tutor::Tutor;

pub use emblem::emblem;

use crate::ui::Frame;

const TEXT_SIZE: u16 = 16;
const TEXT_MARGIN: f32 = 8.0;

/// Pixel crunch: world units per rendered pixel.
///
/// The fiction draws into a target this many times smaller than the logical
/// world and upscales nearest-neighbour — hard pixel edges everywhere, per
/// the art doc. Set to 1.0 and everything still draws, just uncrunched.
pub const CRUNCH: f32 = 2.0;

/// Seconds between wall-clock autosaves; cue-driven saves come sooner.
const SAVE_EVERY: f64 = 10.0;

/// Storage key for the flight recorder's black box, persisted on the same
/// cadence as the save.
const REPLAY_KEY: &str = "space-trucking/replay";

/// Storage key the web shell mirrors `prefers-reduced-motion` into: `"1"`
/// while reduced, absent otherwise (see `web/index.html`). Read when the
/// game loads — a mid-session OS toggle applies the next time it does.
/// Native builds have no shell writing it, so the key stays absent and
/// motion stays full.
const REDUCED_MOTION_KEY: &str = "space-trucking/reduced-motion";

/// Storage key for developer mode, which is the only thing that unlocks
/// fast-forward. The web shell sets it after the pretty-please dialogue
/// (see `web/index.html`); natively, `--dev` on the command line sets it
/// for good. Players never need it: the game is meant to run at 1x.
const DEV_KEY: &str = "space-trucking/dev";

/// Storage key the web shell mirrors the local wall clock's deep-night
/// window into (23:30–06:00): `"1"` inside it, `"0"` outside. Native
/// builds fall back to the same window in UTC — close enough for a
/// mystery.
const NIGHT_KEY: &str = "space-trucking/night";

/// Storage key this game raises (`"1"`) for the web shell when the player
/// has walked into it and no telemetry choice is recorded yet: the cue to
/// show the consent card. The page cannot know when the game was chosen
/// from the menu, and the consent contract is "asked once, before first
/// play" — so the game says so. The shell clears it again on load and on
/// an answer, so a stale one never asks early.
const CONSENT_WANTED_KEY: &str = "space-trucking/consent-wanted";

/// A frame gap larger than this means the tab was backgrounded, the machine
/// slept, or the player was at the menu: real time kept passing, so the
/// missing ticks are replayed through `fast_forward` instead of being
/// clamped away.
const STALL_SECONDS: f64 = 1.0;

/// Whether the player's OS asked for reduced motion, per the mirrored key.
fn reduced_motion() -> bool {
    storage::get(REDUCED_MOTION_KEY).as_deref() == Some("1")
}

/// Whether developer mode is unlocked (the pretty-please was said).
fn dev_mode() -> bool {
    storage::get(DEV_KEY).as_deref() == Some("1")
}

/// Whether it is deep night (23:30–06:00) on the player's clock: the web
/// shell's mirrored local answer when present, otherwise the platform
/// fallback below. Deliberately a frontend concern — the sim only ever
/// sees the resulting `InputFrame` bit, so timezones, travel, and
/// multiplayer clocks are all somebody else's problem out here.
fn night_now() -> bool {
    match storage::get(NIGHT_KEY).as_deref() {
        Some("1") => true,
        Some(_) => false,
        None => local_night(),
    }
}

/// Native: the OS clock and timezone, read directly.
#[cfg(not(target_arch = "wasm32"))]
fn local_night() -> bool {
    use chrono::Timelike;
    let now = chrono::Local::now();
    let minutes = now.hour() * 60 + now.minute();
    !(360..1410).contains(&minutes)
}

/// Web without the shell's mirror (storage refused): UTC, the honest
/// approximation of last resort.
#[cfg(target_arch = "wasm32")]
fn local_night() -> bool {
    let day = macroquad::miniquad::date::now().rem_euclid(86_400.0);
    let hour = day / 3600.0;
    !(6.0..23.5).contains(&hour)
}

/// Longest frame the replay pacer banks, matching the sim's own clamp so a
/// backgrounded playback does not spiral catching up.
const REPLAY_FRAME_CLAMP: f32 = 0.25;

/// Longest absence the startup catch-up replays, in seconds (six hours).
const MAX_CATCH_UP: f64 = 6.0 * 3600.0;

/// Sim ticks per wall-clock second of absence.
const CATCH_UP_RATE: f64 = 60.0;

/// A run of Space Trucking: the sim, and everything around it that is
/// about this machine rather than the ship.
// Independent switches the shell and the player flip separately; a state
// machine over them would only rename the bools.
#[allow(clippy::struct_excessive_bools)]
pub struct Game {
    sim: Sim,
    recording: Recording,
    /// The opt-in aggregator; `None` unless consent is recorded as `"yes"`.
    telemetry: Option<Aggregate>,
    /// Whether the consent card's answer has yet to arrive (see
    /// [`Game::load`]).
    consent_pending: bool,
    /// Seconds of absence the startup catch-up replayed, for the telemetry
    /// row of the same name.
    caught_up_seconds: f64,
    audio: Audio,
    juice: Juice,
    tutor: Tutor,
    reduced_motion: bool,
    dev: bool,
    night: bool,
    /// Wall-clock idle for the onboarding ghost: seconds since the player
    /// last pressed, keyed, or toggled anything. Pointer motion deliberately
    /// does not count — watching must not hold the tutor at bay.
    idle_seconds: f32,
    target: RenderTarget,
    /// Wall-clock moments (miniquad's `date::now`) of the last save and the
    /// last frame.
    last_save: f64,
    last_frame: f64,
    /// Where the pointer was this frame, in world coordinates, for drawing.
    pointer: Vec2,
}

impl Game {
    /// Restore the save (replaying the absence), tune the telemetry, and
    /// bake the sound bank.
    pub async fn load() -> Self {
        if std::env::args().any(|arg| arg == "--dev") {
            // The native pretty-please. Saying it once is saying it forever.
            storage::set(DEV_KEY, "1");
        }

        let (sim, arrived_while_away, caught_up_seconds) = restore();
        let recording = Recording::new(sim.save_string());
        // On a first visit the consent card's answer necessarily arrives AFTER
        // boot — the card floats over the already-running game — so while no
        // choice is recorded yet, keep asking at the persist cadence and let a
        // late "yes" start counting mid-session.
        let consent_pending = storage::get(CONSENT_KEY).is_none();
        if consent_pending && cfg!(target_arch = "wasm32") {
            storage::set(CONSENT_WANTED_KEY, "1");
        }
        let telemetry = boot_telemetry(caught_up_seconds);
        let audio = Audio::load().await;
        let mut juice = Juice::default();
        if arrived_while_away {
            juice.catch_up_arrival();
        }
        let now = macroquad::miniquad::date::now();
        Self {
            sim,
            recording,
            telemetry,
            consent_pending,
            caught_up_seconds,
            audio,
            juice,
            tutor: Tutor::default(),
            reduced_motion: reduced_motion(),
            dev: dev_mode(),
            night: night_now(),
            idle_seconds: 0.0,
            target: pixel_target(),
            last_save: now,
            last_frame: now,
            pointer: Vec2::default(),
        }
    }

    /// Write the save, roll the black box, and do the other work that rides
    /// the save cadence.
    fn persist(&mut self, now: f64) {
        storage::store(&self.sim.save_string(), now);
        if self.recording.is_full() && self.sim.held(0).is_none() {
            // Roll the tape. Saves drop drags, so only cut between them.
            self.recording
                .rebase(self.sim.save_string(), self.sim.tick());
        }
        self.recording.seal(self.sim.tick());
        storage::set(REPLAY_KEY, &self.recording.serialize());
        if self.consent_pending {
            match storage::get(CONSENT_KEY).as_deref() {
                Some("yes") => {
                    self.telemetry = Some(consented_aggregate(self.caught_up_seconds));
                    self.consent_pending = false;
                }
                Some(_) => self.consent_pending = false,
                None => {}
            }
        }
        if let Some(aggregate) = &self.telemetry {
            // Boot merged this aggregate with its stored predecessor,
            // so the write is the running total across sessions.
            storage::set(BUFFER_KEY, &aggregate.serialize());
        }
        // Cheap clock work rides the save cadence: the night window
        // creeps, and the shell may have heard a pretty-please.
        self.night = night_now();
        self.dev = dev_mode();
        self.last_save = now;
    }
}

impl crate::games::Game for Game {
    fn update(&mut self, dt: f32) {
        let view = View::fit(screen_width(), screen_height());
        let input = gather_input(&view, self.dev, self.night);
        let toggle_mute =
            is_key_pressed(KeyCode::M) || (input.press && layout::SPEAKER.contains(input.pointer));
        self.pointer = input.pointer;

        self.last_frame = stall_catch_up(&mut self.sim, &mut self.juice, self.last_frame, dt);

        self.recording.record_frame(self.sim.tick(), &input);
        self.sim.advance(dt, &input);
        self.juice.update(dt, &self.sim, input.pointer, input.press);
        self.audio.update(dt, &self.sim, toggle_mute);

        // Any real input resets the idle clock, which is also how the tutor
        // learns to snuff its ghost mid-demonstration.
        let interacted = input.press
            || input.held
            || input.release
            || input.toggle_pause
            || input.toggle_warp
            || input.reseed.is_some()
            || toggle_mute;
        self.idle_seconds = if interacted {
            0.0
        } else {
            self.idle_seconds + dt
        };
        self.tutor.update(dt, self.idle_seconds, &self.sim);

        if self.sim.cues().iter().any(|cue| matches!(cue, Cue::Reseed)) {
            // The black box tells one run's story: a new world, a new tape.
            self.recording = Recording::new(self.sim.save_string());
        }

        if let Some(aggregate) = &mut self.telemetry {
            // Cues live until the next advance, so the aggregator sees this
            // frame's — the reseed just counted included — with the coarse
            // state as the advance left it.
            aggregate.observe(self.sim.cues(), dt, Snapshot::of(&self.sim));
        }

        let now = macroquad::miniquad::date::now();
        if save_worthy(&self.sim) || now - self.last_save >= SAVE_EVERY {
            self.persist(now);
        }
    }

    fn draw(&self, frame: &Frame) {
        render::draw(
            &View::of(frame),
            &self.target,
            &render::Scene {
                sim: &self.sim,
                juice: &self.juice,
                pointer: self.pointer,
                ghost: self.tutor.ghost(),
                audio_waiting: self.audio.needs_gesture(),
                audio_muted: self.audio.muted(),
                reduced_motion: self.reduced_motion,
                dev: self.dev,
            },
        );
        draw_version(palette::VERSION_TEXT);
    }

    /// The ship flies on without anyone watching, and the loops that were
    /// sounding for it stop.
    fn leave(&mut self) {
        self.audio.hush();
        // A save on the way out, so the absence is measured from now and
        // closing the tab at the menu loses nothing.
        self.persist(macroquad::miniquad::date::now());
    }
}

/// Play a recording back at real time: one sim tick per elapsed [`TICK_DT`]
/// (times [`WARP_FACTOR`] while the recorded session warped), rendering
/// normally with the recorded pointer as a ghost cursor. Local input is
/// ignored — close the window to leave. The version string turns amber as
/// the "this is a replay" tell.
// macroquad's future runs on the main thread; Send is not on the menu.
#[allow(clippy::future_not_send)]
pub async fn replay_session(path: Option<String>) {
    let Some(path) = path else {
        eprintln!("usage: game-prototypes --replay <file>");
        return;
    };
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(err) => {
            eprintln!("cannot read {path}: {err}");
            return;
        }
    };
    let recording = match Recording::parse(&text) {
        Ok(recording) => recording,
        Err(err) => {
            eprintln!("cannot replay {path}: {err}");
            return;
        }
    };
    let Ok(mut sim) = recording.base_sim() else {
        // Unreachable in practice: parse already validated the base.
        eprintln!("cannot replay {path}: base save refused");
        return;
    };
    let mut cursor = recording.cursor();
    let mut audio = Audio::load().await;
    let mut juice = Juice::default();
    // The viewer's own preference, not the recorded session's: reduced
    // motion is presentation, and the tape carries only inputs.
    let reduced_motion = reduced_motion();
    let target = pixel_target();
    let mut accumulator: f32 = 0.0;
    let mut rolling = true;

    loop {
        let view = View::fit(screen_width(), screen_height());
        let dt = get_frame_time();
        if rolling {
            let scale = if sim.is_warp() { WARP_FACTOR } else { 1.0 };
            accumulator += (dt * scale).clamp(0.0, REPLAY_FRAME_CLAMP * scale);
            // The float condition is the fixed-timestep idiom; the clamp
            // above bounds the loop.
            #[allow(clippy::while_float)]
            while rolling && accumulator >= TICK_DT {
                accumulator -= TICK_DT;
                // A refused tick (corrupt tape) simply freezes the frame:
                // the box shows as much of the story as it holds.
                rolling = cursor.next_tick(&mut sim).unwrap_or(false);
            }
        }
        let pointer = cursor.pointer();
        juice.update(dt, &sim, pointer, false);
        audio.update(dt, &sim, false);
        render::draw(
            &view,
            &target,
            &render::Scene {
                sim: &sim,
                juice: &juice,
                pointer,
                // Replays show the recorded hand, never the tutor's.
                ghost: None,
                audio_waiting: audio.needs_gesture(),
                audio_muted: audio.muted(),
                reduced_motion,
                dev: true,
            },
        );
        draw_version(palette::fade(palette::AMBER, 0.75));
        next_frame().await;
    }
}

/// The low-res target the whole fiction renders into. Nearest filtering is
/// what makes the upscale pixels instead of blur. `sample_count` must be 0:
/// even 1 makes macroquad allocate an MSAA-resolve pass whose blit needs
/// WebGL2, and the vendored `gl.js` context is WebGL 1.
#[allow(clippy::cast_sign_loss)] // Both operands are positive constants.
fn pixel_target() -> RenderTarget {
    let target = render_target_ex(
        (WORLD_W / CRUNCH) as u32,
        (WORLD_H / CRUNCH) as u32,
        RenderTargetParams {
            sample_count: 0,
            depth: false,
        },
    );
    target.texture.set_filter(FilterMode::Nearest);
    target
}

/// A backgrounded tab or a sleeping laptop does not pause the world: when
/// the wall clock says far more passed than the frame did, replay the
/// difference silently, exactly like the boot catch-up. Returns the new
/// frame timestamp.
fn stall_catch_up(sim: &mut Sim, juice: &mut Juice, last_frame: f64, dt: f32) -> f64 {
    let wall_now = macroquad::miniquad::date::now();
    let wall_gap = wall_now - last_frame;
    if wall_gap > STALL_SECONDS {
        let missed = (wall_gap - f64::from(dt)).clamp(0.0, MAX_CATCH_UP);
        let ticks = u64::try_from((missed * CATCH_UP_RATE) as i64).unwrap_or(0);
        if sim.fast_forward(ticks).arrived {
            juice.catch_up_arrival();
        }
    }
    wall_now
}

/// Load the save and replay the absence, or start fresh. The second value
/// reports whether the ship docked somewhere while the player was away, so
/// the renderer can pulse the dock ring about it; the third is how many
/// seconds of absence were actually replayed, for the telemetry catch-up
/// row.
fn restore() -> (Sim, bool, f64) {
    let Some((save, saved_at)) = storage::load() else {
        return (Sim::new(fresh_seed()), false, 0.0);
    };
    let Ok(mut sim) = Sim::from_save(&save) else {
        return (Sim::new(fresh_seed()), false, 0.0);
    };
    let elapsed = (macroquad::miniquad::date::now() - saved_at).clamp(0.0, MAX_CATCH_UP);
    let ticks = u64::try_from((elapsed * CATCH_UP_RATE) as i64).unwrap_or(0);
    let caught_up = sim.fast_forward(ticks);
    (
        sim,
        caught_up.arrived,
        caught_up.ticks as f64 / CATCH_UP_RATE,
    )
}

/// Construct the telemetry aggregator iff consent is recorded as `"yes"`
/// (docs/space-trucking/TELEMETRY.md): load the stored buffer if it parses —
/// the merge across sessions — count the session, and count the replayed
/// absence. Anything else — declined, or no consent recorded, which is
/// every native build since only the web shell's consent card writes the
/// key — means opt-in never happened: nothing is constructed, nothing will
/// be written, and any previously stored buffer is deleted. Declining
/// cleans up.
fn boot_telemetry(catch_up_seconds: f64) -> Option<Aggregate> {
    if storage::get(CONSENT_KEY).as_deref() == Some("yes") {
        Some(consented_aggregate(catch_up_seconds))
    } else {
        storage::remove(BUFFER_KEY);
        None
    }
}

/// The consented aggregator: the stored buffer if it parses — the merge
/// across sessions — with this session and its replayed absence counted.
/// Shared by boot and by a first visit's late "yes" from the consent card.
fn consented_aggregate(catch_up_seconds: f64) -> Aggregate {
    let mut aggregate = storage::get(BUFFER_KEY)
        .and_then(|text| Aggregate::parse(&text).ok())
        .unwrap_or_default();
    aggregate.note_session();
    aggregate.note_catch_up(catch_up_seconds);
    aggregate
}

/// Whether this frame produced a cue worth writing the save for.
fn save_worthy(sim: &Sim) -> bool {
    sim.cues().iter().any(|cue| {
        matches!(
            cue,
            Cue::Arrive
                | Cue::Depart
                | Cue::Accept { .. }
                | Cue::Place
                | Cue::Pause { .. }
                | Cue::Reseed
        )
    })
}

/// Wall-clock seed for fresh runs; determinism starts once the sim owns it.
fn fresh_seed() -> u64 {
    macroquad::miniquad::date::now().to_bits()
}

/// Letterboxed mapping between the sim's fixed logical world and the window.
///
/// The world is exactly the shell's [`Frame`] in size, so this is the frame
/// again in the sim's own vector type.
pub struct View {
    scale: f32,
    origin: Vec2,
}

impl View {
    /// The mapping for a window of this size.
    #[must_use]
    fn fit(screen_w: f32, screen_h: f32) -> Self {
        Self::of(&Frame::fit(screen_w, screen_h))
    }

    /// The mapping the shell's frame already is.
    #[must_use]
    fn of(frame: &Frame) -> Self {
        let origin = frame.at(macroquad::math::Vec2::ZERO);
        Self {
            scale: frame.scale(),
            origin: Vec2::new(origin.x, origin.y),
        }
    }

    #[must_use]
    pub fn to_screen(&self, world: Vec2) -> Vec2 {
        world * self.scale + self.origin
    }

    #[must_use]
    fn to_world(&self, screen: Vec2) -> Vec2 {
        (screen - self.origin) * self.scale.recip()
    }

    /// World-to-screen length factor.
    #[must_use]
    pub const fn scale(&self) -> f32 {
        self.scale
    }
}

fn gather_input(view: &View, dev_mode: bool, night: bool) -> InputFrame {
    let (mouse_x, mouse_y) = mouse_position();
    let pointer = view.to_world(Vec2::new(mouse_x, mouse_y));
    let press = is_mouse_button_pressed(MouseButton::Left);
    InputFrame {
        pointer,
        press,
        held: is_mouse_button_down(MouseButton::Left),
        release: is_mouse_button_released(MouseButton::Left),
        // The icon buttons fold into the same toggles as the keys; the sim
        // deliberately ignores presses on those rects.
        toggle_pause: is_key_pressed(KeyCode::Space)
            || (press && layout::PAUSE_BTN.contains(pointer)),
        toggle_warp: dev_mode
            && (is_key_pressed(KeyCode::F) || (press && layout::WARP_BTN.contains(pointer))),
        shift: is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift),
        night,
        reseed: is_key_pressed(KeyCode::R).then(fresh_seed),
    }
}

/// The one permitted piece of text: the version, bottom-right. The colour is
/// a palette role — live play uses [`palette::VERSION_TEXT`], replay tints
/// it amber as its only tell.
fn draw_version(color: Color) {
    let version = format!("space-trucking {VERSION}");
    let size = measure_text(&version, None, TEXT_SIZE, 1.0);
    draw_text(
        &version,
        screen_width() - size.width - TEXT_MARGIN,
        screen_height() - TEXT_MARGIN,
        f32::from(TEXT_SIZE),
        color,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::{FRAME_H, FRAME_W};

    /// The frontend leans on the world and the shell's frame being one box.
    #[test]
    fn the_world_is_the_shells_frame() {
        assert!((WORLD_W - FRAME_W).abs() < f32::EPSILON);
        assert!((WORLD_H - FRAME_H).abs() < f32::EPSILON);
    }
}
