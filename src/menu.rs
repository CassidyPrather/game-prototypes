//! The main menu: a carousel of prototypes, dressed as wirenook.net.
//!
//! The look is borrowed from the site's style guide (wirenook.net/style):
//! a sky gradient, a field of four-point sparkles down the left edge, a
//! sigil triangle, technical-drawing tick rulers along the top and right
//! that wake up where you point, and striped windows with a double border
//! and chips in their top line. Each prototype is one window, and inside
//! it a night-sky screen shows the toy's own emblem.
//!
//! Almost no text, like the prototypes themselves: the name in each
//! window's title chip, because a shelf of unnamed pictures stops being a
//! menu, and the build version. What to press is drawn as key caps.
//!
//! The part that can be wrong — where the pointer is — lives in the
//! library's [`game_prototypes::shell`] and is unit-tested there, as is
//! what the menu sounds like. [`Screen`] here holds only presentation:
//! springs and timers that make the menu move, eased every frame toward
//! whatever the shell's state says.

use std::f32::consts::PI;

use game_prototypes::VERSION;
use game_prototypes::shell::sfx::Sfx;
use game_prototypes::shell::{GameId, Menu};
use macroquad::color::Color;
use macroquad::input::{
    KeyCode, MouseButton, get_last_key_pressed, is_key_pressed, is_mouse_button_pressed,
    mouse_position, mouse_wheel,
};
use macroquad::text::Font;

use crate::games;
use crate::sounds::Sounds;
use crate::ui::{self, Frame, MqVec2, pulse, with_alpha};

/// wirenook's palette, from its press kit.
mod brand {
    use macroquad::color::Color;

    /// A `#rrggbb` colour at an alpha.
    const fn hex(rgb: u32, alpha: f32) -> Color {
        Color::new(
            ((rgb >> 16) & 0xff) as f32 / 255.0,
            ((rgb >> 8) & 0xff) as f32 / 255.0,
            (rgb & 0xff) as f32 / 255.0,
            alpha,
        )
    }

    // The page behind everything: a sky gradient in three stops.
    pub const SKY_HIGH: Color = hex(0x65_b8_e6, 1.0);
    pub const SKY_MID: Color = hex(0x51_9e_d5, 1.0);
    pub const SKY_LOW: Color = hex(0x8a_bb_e9, 1.0);

    pub const ICE: Color = hex(0xd7_fb_ff, 1.0);
    pub const CYAN: Color = hex(0x73_f4_ff, 1.0);
    pub const VIOLET: Color = hex(0x6d_2c_95, 1.0);
    pub const MAGENTA: Color = hex(0xff_00_e6, 1.0);
    pub const INK: Color = hex(0x27_14_52, 1.0);
    pub const BANANA: Color = hex(0xff_f9_b1, 1.0);
    /// "Orange is now officially a part of it."
    pub const ORANGE: Color = hex(0xff_66_00, 1.0);

    /// Chips in a window's top line.
    pub const PANEL: Color = hex(0xe7_fc_ff, 1.0);
    /// Every border.
    pub const LINE: Color = Color::new(231.0 / 255.0, 252.0 / 255.0, 1.0, 0.9);
    pub const SHADOW: Color = Color::new(65.0 / 255.0, 24.0 / 255.0, 127.0 / 255.0, 0.28);

    // A window's fill, and the stripes laid over it.
    pub const WINDOW_A: Color = Color::new(78.0 / 255.0, 171.0 / 255.0, 216.0 / 255.0, 0.72);
    pub const WINDOW_B: Color = Color::new(191.0 / 255.0, 171.0 / 255.0, 1.0, 0.74);
    pub const STRIPE_A: Color = Color::new(84.0 / 255.0, 80.0 / 255.0, 188.0 / 255.0, 0.2);
    pub const STRIPE_B: Color = Color::new(111.0 / 255.0, 244.0 / 255.0, 1.0, 0.24);

    // The wave field's two lattices of sparkles.
    pub const SPARK_VIOLET: Color = Color::new(139.0 / 255.0, 19.0 / 255.0, 214.0 / 255.0, 0.75);
    pub const SPARK_LILAC: Color = Color::new(221.0 / 255.0, 200.0 / 255.0, 235.0 / 255.0, 0.34);

    // Nightmode wallpapers: each prototype's screen, and the dive into one.
    pub const NIGHT: Color = hex(0x0b_0c_25, 1.0);
    pub const NIGHT_HIGH: Color = hex(0x13_15_3f, 1.0);
    pub const NIGHT_VIOLET: Color = hex(0x15_08_6a, 1.0);
}

/// One prototype's window, in frame units, at full size.
const CARD: MqVec2 = MqVec2::new(440.0, 270.0);
/// Where the chosen window sits.
const CARD_Y: f32 = 318.0;
/// Window centres are this far apart along the carousel. Wider than the
/// frame's half, so neighbours peek in at the edges rather than crowding.
const PITCH: f32 = 480.0;
/// A window's inner margin.
const PAD: f32 = 14.0;
/// Height of the chips in a window's top line.
const CHIP_H: f32 = 32.0;
/// The night-sky screen inside a window, relative to the window's centre.
const SCREEN_CENTRE: MqVec2 = MqVec2::new(0.0, 22.0);
const SCREEN: MqVec2 = MqVec2::new(CARD.x - PAD * 2.0, 196.0);

/// The pointer counts as resting this long after it last moved; after that
/// the rulers go back to marking the chosen window.
const POINTER_REST: f32 = 1.5;
/// From the press to the screen being covered and the prototype loading.
const LAUNCH_SECS: f32 = 0.5;
/// The iris opening back onto the menu after leaving a prototype.
const RETURN_SECS: f32 = 0.45;
/// The iris opening onto a prototype once it has loaded.
pub const REVEAL_SECS: f32 = 0.45;
/// What a "seconds since" timer starts at: long enough ago that nothing
/// it drives is still moving.
const LONG_AGO: f32 = 1000.0;

// Tick rulers, as on the site: minor, fifth and tenth tick.
const TICK_SPACING: f32 = 8.0;
const TICK_LEN: [f32; 3] = [6.0, 10.0, 16.0];
const TICK_ALPHA: [f32; 3] = [0.34, 0.46, 0.6];
/// Extra tick length at the peak of the bump under the cursor.
const TICK_BUMP: f32 = 22.0;
/// Bump half-width while following the pointer.
const POINTER_SIGMA: f32 = 60.0;

/// The menu's motion: springs and timers. Everything that decides *what*
/// is chosen is the library's [`Menu`]; this only decides how it looks
/// getting there.
pub struct Screen {
    font: Font,
    /// Seconds since the program started, for idle motion.
    clock: f32,
    /// Seconds since the menu last came on screen, for its entrance.
    shown: f32,
    /// Whether it came on screen out of a prototype, which plays the dive
    /// backwards.
    from_game: bool,
    /// The carousel's position, in entries; eases toward the pointer.
    scroll: f32,
    /// How far the chosen window has risen, `0..=1`.
    lift: f32,
    /// Whether the mouse is over the chosen window, and that eased.
    hover: bool,
    hover_ease: f32,
    /// Seconds since a move that had nowhere to go, and which way it went.
    shake: f32,
    shake_dir: f32,
    /// Seconds since the header mark was last startled into a hop.
    hop: f32,
    /// A prototype on its way in, and seconds since it was chosen.
    launch: Option<(GameId, f32)>,
    /// The mouse, in frame units, and how long it has been still.
    mouse: MqVec2,
    mouse_idle: f32,
    /// Seconds since each drawn key was pressed, to press the cap too.
    arrows_pressed: f32,
    enter_pressed: f32,
    /// Seconds since the wheel last moved the carousel. A trackpad scrolls
    /// in a stream of small events, one per frame, and each must not be a
    /// step of its own.
    wheel_turned: f32,
    ruler: Ruler,
}

/// The tick rulers' drawn state, eased toward where they should point.
#[derive(Clone, Copy)]
struct Ruler {
    /// Where the bump is, in frame units.
    at: MqVec2,
    /// How pronounced the bump is, `0..=1`.
    amp: f32,
    /// `0` following the pointer, `1` hugging the chosen window with
    /// brackets on its extent.
    focus: f32,
}

/// Where one window is drawn this frame.
#[derive(Clone, Copy)]
struct Place {
    centre: MqVec2,
    scale: f32,
    /// `1` for the window at the carousel's centre, falling to `0` a whole
    /// pitch away.
    focus: f32,
}

impl Screen {
    pub fn new(font: Font) -> Self {
        Self {
            font,
            clock: 0.0,
            shown: 0.0,
            from_game: false,
            scroll: 0.0,
            lift: 0.0,
            hover: false,
            hover_ease: 0.0,
            shake: LONG_AGO,
            shake_dir: 0.0,
            hop: LONG_AGO,
            launch: None,
            mouse: MqVec2::new(ui::FRAME_W * 0.5, ui::FRAME_H * 0.5),
            mouse_idle: POINTER_REST,
            arrows_pressed: LONG_AGO,
            enter_pressed: LONG_AGO,
            wheel_turned: LONG_AGO,
            ruler: Ruler {
                at: MqVec2::new(ui::FRAME_W * 0.5, CARD_Y),
                amp: 0.0,
                focus: 1.0,
            },
        }
    }

    /// The menu is back on screen after a prototype: replay the entrance,
    /// out of the dark the prototype was in.
    pub const fn show_after_game(&mut self) {
        self.shown = 0.0;
        self.from_game = true;
        self.launch = None;
        self.lift = 0.0;
        self.hop = 0.0;
        self.mouse_idle = POINTER_REST;
    }

    /// Read the keyboard and mouse for one frame and move everything along.
    /// Returns a prototype once its launch has covered the screen, which is
    /// the moment to load it.
    ///
    /// Enter and a click start; Space does not, so that holding it to launch
    /// Leitmotif cannot also spend its fermata on the first frame of play.
    pub fn update(
        &mut self,
        menu: &mut Menu,
        frame: &Frame,
        dt: f32,
        sounds: &mut Sounds,
    ) -> Option<GameId> {
        self.clock += dt;
        self.shown += dt;
        for timer in [
            &mut self.shake,
            &mut self.hop,
            &mut self.arrows_pressed,
            &mut self.enter_pressed,
            &mut self.wheel_turned,
        ] {
            *timer = (*timer + dt).min(LONG_AGO);
        }

        if get_last_key_pressed().is_some() || is_mouse_button_pressed(MouseButton::Left) {
            sounds.wake();
        }

        let (mouse_x, mouse_y) = mouse_position();
        let mouse = frame.frame_pos(MqVec2::new(mouse_x, mouse_y));
        if mouse.distance_squared(self.mouse) > 0.25 {
            self.mouse_idle = 0.0;
        } else {
            self.mouse_idle += dt;
        }
        self.mouse = mouse;

        if let Some((id, age)) = self.launch.as_mut() {
            *age += dt;
            let (id, age) = (*id, *age);
            self.ease(*menu, dt);
            return (age >= LAUNCH_SECS).then_some(id);
        }

        self.read_keys(menu, sounds);

        let under = self.card_under(*menu, mouse);
        let over_chosen = under == Some(menu.index());
        if over_chosen && !self.hover && self.mouse_idle == 0.0 {
            sounds.play(Sfx::Hover);
        }
        self.hover = over_chosen;

        let mut go = is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::KpEnter);
        if go {
            self.enter_pressed = 0.0;
            self.mouse_idle = POINTER_REST;
        }
        if is_mouse_button_pressed(MouseButton::Left) {
            match under {
                Some(index) if index == menu.index() => go = true,
                // A click on a neighbour peeking in brings it to the centre.
                Some(index) => {
                    let sfx = if index > menu.index() {
                        Sfx::Next
                    } else {
                        Sfx::Prev
                    };
                    menu.point_at(index);
                    self.moved(sounds, sfx);
                }
                None => {}
            }
        }
        if go {
            self.launch = Some((menu.selected(), 0.0));
            self.hop = 0.0;
            sounds.play(Sfx::Launch);
        }

        self.ease(*menu, dt);
        None
    }

    /// Arrows, WASD and the wheel all walk the carousel.
    fn read_keys(&mut self, menu: &mut Menu, sounds: &Sounds) {
        let pressed = |keys: [KeyCode; 4]| keys.iter().any(|&key| is_key_pressed(key));
        let mut step = 0;
        if pressed([KeyCode::Right, KeyCode::Down, KeyCode::D, KeyCode::S]) {
            step += 1;
        }
        if pressed([KeyCode::Left, KeyCode::Up, KeyCode::A, KeyCode::W]) {
            step -= 1;
        }
        if step != 0 {
            self.arrows_pressed = 0.0;
            self.mouse_idle = POINTER_REST;
        }
        let (_, wheel) = mouse_wheel();
        if step == 0 && wheel != 0.0 && self.wheel_turned > 0.2 {
            step = if wheel < 0.0 { 1 } else { -1 };
            self.wheel_turned = 0.0;
        }
        if step == 0 {
            return;
        }

        if Menu::len() < 2 {
            // Nowhere to go: say so with a shake and a knock rather than
            // silently doing nothing.
            self.shake = 0.0;
            self.shake_dir = step as f32;
            sounds.play(Sfx::Bump);
            return;
        }
        menu.move_by(step);
        self.moved(sounds, if step > 0 { Sfx::Next } else { Sfx::Prev });
    }

    /// The pointer landed on a different window.
    fn moved(&mut self, sounds: &Sounds, sfx: Sfx) {
        self.lift = 0.0;
        self.hop = 0.0;
        sounds.play(sfx);
    }

    /// Ease every spring one frame toward its target.
    fn ease(&mut self, menu: Menu, dt: f32) {
        self.scroll = approach(self.scroll, menu.index() as f32, 12.0, dt);
        self.lift = approach(self.lift, 1.0, 7.0, dt);
        self.hover_ease = approach(self.hover_ease, f32::from(u8::from(self.hover)), 10.0, dt);

        let pointing = self.mouse_idle < POINTER_REST
            && (0.0..ui::FRAME_W).contains(&self.mouse.x)
            && (0.0..ui::FRAME_H).contains(&self.mouse.y);
        let (target, focus) = if pointing {
            (self.mouse, 0.0)
        } else {
            (self.place(menu.index(), menu.index()).centre, 1.0)
        };
        let ruler = &mut self.ruler;
        ruler.at.x = approach(ruler.at.x, target.x, 14.0, dt);
        ruler.at.y = approach(ruler.at.y, target.y, 14.0, dt);
        ruler.amp = approach(ruler.amp, 1.0, 3.0, dt);
        ruler.focus = approach(ruler.focus, focus, 8.0, dt);
    }

    /// Where window `index` is drawn this frame, given the chosen one.
    fn place(&self, index: usize, chosen: usize) -> Place {
        let offset = index as f32 - self.scroll;
        let focus = (1.0 - offset.abs()).clamp(0.0, 1.0);
        let mut scale = focus.mul_add(0.2, 0.8);
        let mut centre = MqVec2::new(offset.mul_add(PITCH, ui::FRAME_W * 0.5), CARD_Y);

        // The entrance: every window rises into place, neighbours a beat
        // behind the centre.
        let rise = ease_out_back(unit((self.shown - offset.abs().mul_add(0.08, 0.1)) / 0.55));
        centre.y = (1.0 - rise).mul_add(90.0, centre.y);

        if index == chosen {
            let lift = ease_out_cubic(self.lift);
            let float = (self.clock * 2.2).sin() * 2.0 * self.hover_ease;
            scale *= lift.mul_add(0.025, self.hover_ease.mul_add(0.015, 1.0));
            centre.y -= lift.mul_add(6.0, self.hover_ease * 4.0) + float;
            if self.shake < 0.6 {
                let wobble = (self.shake * 48.0).sin() * (-self.shake * 9.0).exp();
                centre.x = (wobble * 12.0).mul_add(self.shake_dir, centre.x);
            }
            if let Some((_, age)) = self.launch {
                // Squash on the press, then spring back past full size.
                scale *= ((age * 26.0).sin() * (-age * 7.0).exp()).mul_add(-0.05, 1.0);
            }
        }
        Place {
            centre,
            scale,
            focus,
        }
    }

    /// The window under a frame-unit point, if any. Windows too far along
    /// the carousel to be seen do not count.
    fn card_under(&self, menu: Menu, at: MqVec2) -> Option<usize> {
        // The chosen window is drawn on top, so it wins where they overlap.
        let chosen = menu.index();
        std::iter::once(chosen)
            .chain((0..Menu::len()).filter(|&index| index != chosen))
            .find(|&index| {
                let place = self.place(index, chosen);
                let offset = at - place.centre;
                offset.x.abs() < CARD.x * 0.5 * place.scale
                    && offset.y.abs() < CARD.y * 0.5 * place.scale
                    && (0.0..ui::FRAME_W).contains(&at.x)
            })
    }

    /// Paint the menu. `loaded` is the prototype already in memory, if any,
    /// whose window says so: choosing it again resumes rather than loads.
    pub fn draw(&self, menu: Menu, frame: &Frame, loaded: Option<GameId>) {
        sky(frame);
        self.clouds(frame);
        self.wave_field(frame);
        self.sigil(frame);
        self.rulers(frame, menu);
        self.header(frame);

        // Neighbours first, so the chosen window sits over them.
        let chosen = menu.index();
        for (index, &id) in GameId::ALL.iter().enumerate() {
            if index != chosen {
                self.card(
                    frame,
                    id,
                    self.place(index, chosen),
                    false,
                    loaded == Some(id),
                );
            }
        }
        let id = menu.selected();
        self.card(
            frame,
            id,
            self.place(chosen, chosen),
            true,
            loaded == Some(id),
        );

        self.hints(frame);
        self.version(frame);
        self.dive(frame, menu);
    }

    /// Soft clouds drifting right across the sky, behind everything.
    fn clouds(&self, frame: &Frame) {
        /// Height, speed in frame units a second, size and head start.
        const CLOUDS: [(f32, f32, f32, f32); 3] = [
            (150.0, 9.0, 1.0, 0.0),
            (455.0, 6.0, 1.4, 420.0),
            (250.0, 13.0, 0.75, 760.0),
        ];
        /// One cloud's puffs: offset and radius.
        const PUFFS: [(f32, f32, f32); 5] = [
            (0.0, 0.0, 34.0),
            (32.0, -14.0, 28.0),
            (60.0, 4.0, 30.0),
            (-30.0, 8.0, 24.0),
            (88.0, 12.0, 20.0),
        ];
        let span = ui::FRAME_W + 360.0;
        for (y, speed, size, start) in CLOUDS {
            let x = self.clock.mul_add(speed, start).rem_euclid(span) - 180.0;
            // Solid, in the sky's own colour lifted toward white, rather than
            // translucent white: overlapping translucent puffs show every
            // circle's edge, and a cloud should read as one shape.
            let colour = lerp_colour(
                sky_at(MqVec2::new(x, y)),
                Color::new(1.0, 1.0, 1.0, 1.0),
                0.14,
            );
            for (dx, dy, radius) in PUFFS {
                frame.poly(
                    frame.at(MqVec2::new(dx.mul_add(size, x), dy.mul_add(size, y))),
                    24,
                    radius * size,
                    0.0,
                    colour,
                );
            }
        }
    }

    /// Two offset lattices of sparkles down the left edge, fading to the
    /// right, with a slow wave rolling through them.
    // Counts of ticks and bands come from positive layout sizes, so the
    // casts cannot lose a sign.
    #[allow(clippy::cast_sign_loss)]
    fn wave_field(&self, frame: &Frame) {
        const WIDTH: f32 = 126.0;
        const STEP: f32 = 14.0;
        let appear = unit(self.shown * 2.0);
        let columns = (WIDTH / STEP) as usize;
        let rows = (ui::FRAME_H / STEP) as usize + 1;
        for (shift, colour) in [(0.0, brand::SPARK_LILAC), (7.0, brand::SPARK_VIOLET)] {
            for column in 0..=columns {
                let x = (column as f32).mul_add(STEP, shift) + 4.0;
                // Solid for the first 40%, then fading out, like the site's
                // mask on the same field.
                let mask = 1.0 - unit(WIDTH.mul_add(-0.4, x) / (WIDTH * 0.6));
                for row in 0..=rows {
                    let y = (row as f32).mul_add(STEP, shift);
                    let wave = x
                        .mul_add(0.05, y.mul_add(-0.022, self.clock * 1.8))
                        .sin()
                        .mul_add(0.5, 0.5);
                    frame.sparkle(
                        frame.at(MqVec2::new(x, y)),
                        4.2 * wave.mul_add(0.5, 0.5),
                        with_alpha(colour, colour.a * mask * appear),
                    );
                }
            }
        }
    }

    /// The big faint triangle in the bottom-left corner, swaying slowly.
    fn sigil(&self, frame: &Frame) {
        const SIZE: f32 = 300.0;
        const CENTRE: MqVec2 = MqVec2::new(198.0, 514.0);
        const POINTS: [MqVec2; 3] = [
            MqVec2::new(0.5, 0.0),
            MqVec2::new(0.82, 0.86),
            MqVec2::new(0.2, 0.72),
        ];
        let turn = (self.clock * 0.35).sin().mul_add(4.0, 12.0).to_radians();
        let rotation = MqVec2::from_angle(turn);
        let corners = POINTS.map(|point| {
            let local = (point - MqVec2::splat(0.5)) * SIZE;
            frame.at(CENTRE + rotation.rotate(local))
        });
        frame.outline(
            &corners,
            2.5,
            with_alpha(brand::ICE, 0.3 * unit(self.shown * 1.5)),
        );
    }

    /// Tick rulers along the top and right edges. A bump follows the
    /// pointer; when it rests, or the keyboard is in use, the bump glides to
    /// the chosen window and brackets mark its extent.
    // Counts of ticks and bands come from positive layout sizes, so the
    // casts cannot lose a sign.
    #[allow(clippy::cast_sign_loss)]
    fn rulers(&self, frame: &Frame, menu: Menu) {
        let ruler = self.ruler;
        let chosen = self.place(menu.index(), menu.index());
        let half = CARD * (0.5 * chosen.scale);
        let sigma = MqVec2::new(
            lerp(POINTER_SIGMA, half.x * 0.8, ruler.focus),
            lerp(POINTER_SIGMA, half.y * 0.8, ruler.focus),
        );
        // The ticks grow in along each ruler as the menu arrives.
        let grown = |along: f32| unit(self.shown.mul_add(2.2, -along * 0.8));

        let top = (ui::FRAME_W / TICK_SPACING) as usize;
        for i in 0..=top {
            let x = i as f32 * TICK_SPACING;
            let bump = gauss(x - ruler.at.x, sigma.x) * ruler.amp;
            let (length, colour) = tick(i, bump);
            let length = length * grown(x / ui::FRAME_W);
            frame.line(
                frame.at(MqVec2::new(x, 0.0)),
                frame.at(MqVec2::new(x, length)),
                1.2,
                colour,
            );
        }
        let right = (ui::FRAME_H / TICK_SPACING) as usize;
        for i in 0..=right {
            let y = i as f32 * TICK_SPACING;
            let bump = gauss(y - ruler.at.y, sigma.y) * ruler.amp;
            let (length, colour) = tick(i, bump);
            let length = length * grown(y / ui::FRAME_H);
            frame.line(
                frame.at(MqVec2::new(ui::FRAME_W, y)),
                frame.at(MqVec2::new(ui::FRAME_W - length, y)),
                1.2,
                colour,
            );
        }

        // A cursor on each ruler, at the peak of its bump.
        let cursor = with_alpha(brand::MAGENTA, 0.9 * ruler.amp * (1.0 - ruler.focus));
        let reach = TICK_LEN[2] + TICK_BUMP + 5.0;
        let tip = frame.at(MqVec2::new(ruler.at.x, reach));
        let s = frame.scale() * 5.0;
        Frame::triangle(
            tip,
            tip + MqVec2::new(-s, s * 1.4),
            tip + MqVec2::new(s, s * 1.4),
            cursor,
        );
        let tip = frame.at(MqVec2::new(ui::FRAME_W - reach, ruler.at.y));
        Frame::triangle(
            tip,
            tip + MqVec2::new(-s * 1.4, -s),
            tip + MqVec2::new(-s * 1.4, s),
            cursor,
        );

        // Dimension brackets: the chosen window's extent on each ruler.
        let bracket = with_alpha(brand::BANANA, 0.95 * ruler.focus * ruler.amp);
        let from = chosen.centre - half;
        let to = chosen.centre + half;
        let y = reach + 2.0;
        let x = ui::FRAME_W - reach - 2.0;
        let end = 5.0;
        frame.line(
            frame.at(MqVec2::new(from.x, y)),
            frame.at(MqVec2::new(to.x, y)),
            1.5,
            bracket,
        );
        frame.line(
            frame.at(MqVec2::new(x, from.y)),
            frame.at(MqVec2::new(x, to.y)),
            1.5,
            bracket,
        );
        for along in [from.x, to.x] {
            frame.line(
                frame.at(MqVec2::new(along, y - end)),
                frame.at(MqVec2::new(along, y + end)),
                1.5,
                bracket,
            );
        }
        for along in [from.y, to.y] {
            frame.line(
                frame.at(MqVec2::new(x - end, along)),
                frame.at(MqVec2::new(x + end, along)),
                1.5,
                bracket,
            );
        }
    }

    /// The collection's mark: a circle, a square and a triangle, the three
    /// shapes every prototype here is built out of (and the three on the
    /// wirenook business wallpaper). Stickers with a hard violet shadow,
    /// bobbing in a wave, that hop when anything happens.
    fn header(&self, frame: &Frame) {
        const Y: f32 = 78.0;
        const SPACING: f32 = 58.0;
        let fills = [brand::CYAN, brand::BANANA, brand::MAGENTA];
        for (i, fill) in fills.into_iter().enumerate() {
            let n = i as f32;
            let drop = ease_out_back(unit(n.mul_add(-0.07, self.shown) / 0.45));
            let bob = n.mul_add(-0.8, self.clock * 2.0).sin() * 3.0;
            let hop = (PI * unit(n.mul_add(-0.05, self.hop) / 0.3)).sin() * 10.0;
            let tilt = self.clock.mul_add(1.3, n).sin() * 7.0;
            let centre = MqVec2::new(
                (n - 1.0).mul_add(SPACING, ui::FRAME_W * 0.5),
                (1.0 - drop).mul_add(-70.0, Y + bob - hop),
            );
            let (sides, radius, turn) = match i {
                0 => (32, 17.0, 0.0),
                1 => (4, 21.0, 45.0 + tilt),
                _ => (3, 21.0, -90.0 + tilt),
            };
            let at = frame.at(centre);
            let shadow = frame.at(centre + MqVec2::new(4.0, 4.0));
            frame.poly(shadow, sides, radius, turn, with_alpha(brand::VIOLET, 0.55));
            frame.poly(at, sides, radius, turn, fill);
            frame.poly_lines(at, sides, radius, turn, 2.5, brand::INK);
        }
    }

    /// One prototype's window: shadow, striped fill, double border, a top
    /// line of chips, and a night-sky screen with the toy's emblem on it.
    fn card(&self, frame: &Frame, id: GameId, place: Place, chosen: bool, loaded: bool) {
        let Place {
            centre,
            scale,
            focus,
        } = place;
        // Everything in the window is laid out at full size about its centre
        // and scaled with it.
        let at = |local: MqVec2| frame.at(centre + local * scale);
        let size = CARD * scale;
        let corner = at(CARD * -0.5);
        if corner.x > frame.at(MqVec2::new(ui::FRAME_W, 0.0)).x
            || at(CARD * 0.5).x < frame.at(MqVec2::ZERO).x
        {
            return;
        }
        let lift = if chosen {
            ease_out_cubic(self.lift)
        } else {
            0.0
        };

        // A soft shadow in three layers, deeper the higher the window rises.
        for layer in 0..3 {
            let grow = (layer as f32).mul_add(6.0, 2.0);
            let drop = (layer as f32).mul_add(2.0, lift.mul_add(8.0, 10.0));
            frame.rect_centred(
                at(MqVec2::new(0.0, drop)),
                size + MqVec2::splat(grow * scale),
                with_alpha(brand::SHADOW, brand::SHADOW.a / (layer as f32 + 1.5)),
            );
        }

        frame.gradient(
            corner,
            size,
            [
                brand::WINDOW_A,
                lerp_colour(brand::WINDOW_A, brand::WINDOW_B, 0.45),
                brand::WINDOW_B,
                lerp_colour(brand::WINDOW_A, brand::WINDOW_B, 0.55),
            ],
        );
        self.stripes(frame, corner, size, scale, chosen);

        // The double border, 5px double on the site.
        frame.rect_lines_centred(at(MqVec2::ZERO), size, 1.6 * scale, brand::LINE);
        frame.rect_lines_centred(
            at(MqVec2::ZERO),
            size - MqVec2::splat(7.0 * scale),
            1.6 * scale,
            brand::LINE,
        );

        self.top_line(frame, id, &at, scale, chosen, loaded);
        self.screen(frame, id, &at, scale, chosen, lift);

        // Windows away from the centre sink back into the sky.
        let veil = (1.0 - focus) * 0.5;
        if veil > 0.0 {
            frame.rect(corner, size, with_alpha(brand::SKY_MID, veil));
        }
    }

    /// The window's stripes, drifting slowly downward; faster on the chosen
    /// one, which is the only thing on screen being offered.
    // Counts of ticks and bands come from positive layout sizes, so the
    // casts cannot lose a sign.
    #[allow(clippy::cast_sign_loss)]
    fn stripes(&self, frame: &Frame, corner: MqVec2, size: MqVec2, scale: f32, chosen: bool) {
        const PERIOD: f32 = 13.0;
        const WIDE: f32 = 7.0;
        let speed = if chosen { 9.0 } else { 3.0 };
        let drift = (self.clock * speed).rem_euclid(PERIOD);
        let height = size.y / scale;
        let bands = (height / PERIOD) as usize + 2;
        for band in 0..bands {
            let top = (band as f32).mul_add(PERIOD, drift - PERIOD);
            for (from, to, colour) in [
                (top, top + WIDE, brand::STRIPE_A),
                (top + WIDE, top + PERIOD, brand::STRIPE_B),
            ] {
                let (from, to) = (from.clamp(0.0, height), to.clamp(0.0, height));
                if to > from {
                    frame.rect(
                        corner + MqVec2::new(0.0, from * scale * frame.scale()),
                        MqVec2::new(size.x, (to - from) * scale),
                        colour,
                    );
                }
            }
        }
    }

    /// The window's top line: the title chip on the left, a status light on
    /// the right.
    fn top_line(
        &self,
        frame: &Frame,
        id: GameId,
        at: &impl Fn(MqVec2) -> MqVec2,
        scale: f32,
        chosen: bool,
        loaded: bool,
    ) {
        let left = (-CARD.x).mul_add(0.5, PAD);
        let right = CARD.x.mul_add(0.5, -PAD);
        let y = CHIP_H.mul_add(0.5, (-CARD.y).mul_add(0.5, PAD));
        let font = Some(&self.font);

        let text = 20.0;
        let title = frame.label_width(font, id.name(), text * scale) / scale + 22.0;
        chip(
            frame,
            at(MqVec2::new(title.mul_add(0.5, left), y)),
            MqVec2::new(title, CHIP_H) * scale,
            scale,
        );
        frame.label(
            font,
            id.name(),
            at(MqVec2::new(title.mul_add(0.5, left), y + 1.0)),
            text * scale,
            brand::INK,
        );

        // The status light: a dot, and a play mark. The dot beats while the
        // prototype waits in memory, blinks on the chosen window, and is an
        // empty ring otherwise.
        let width = 58.0;
        chip(
            frame,
            at(MqVec2::new(f32::mul_add(width, -0.5, right), y)),
            MqVec2::new(width, CHIP_H) * scale,
            scale,
        );
        let dot = at(MqVec2::new(right - width + 17.0, y));
        if loaded {
            frame.circle(dot, heartbeat(self.clock) * 5.5 * scale, brand::MAGENTA);
        } else if chosen && (self.clock * 0.9).fract() < 0.58 {
            frame.circle(dot, 5.0 * scale, brand::ORANGE);
        } else {
            frame.ring(dot, 4.5 * scale, 1.5 * scale, with_alpha(brand::INK, 0.5));
        }
        let play = at(MqVec2::new(right - 18.0, y));
        let s = 7.0 * scale * frame.scale();
        Frame::triangle(
            play + MqVec2::new(s, 0.0),
            play + MqVec2::new(-s * 0.6, -s),
            play + MqVec2::new(-s * 0.6, s),
            brand::INK,
        );
    }

    /// The night-sky screen inside a window, with twinkling stars, the
    /// prototype's emblem, and focus brackets that snap in on the chosen one.
    fn screen(
        &self,
        frame: &Frame,
        id: GameId,
        at: &impl Fn(MqVec2) -> MqVec2,
        scale: f32,
        chosen: bool,
        lift: f32,
    ) {
        let size = SCREEN * scale;
        frame.gradient(
            at(SCREEN_CENTRE - SCREEN * 0.5),
            size,
            [
                brand::NIGHT_HIGH,
                brand::NIGHT_VIOLET,
                brand::NIGHT,
                brand::NIGHT,
            ],
        );

        // A few stars, placed by a fixed scatter and twinkling out of step.
        for i in 0..18_u16 {
            let n = f32::from(i);
            let spot = MqVec2::new(
                (n * 0.618_034).fract() - 0.5,
                n.mul_add(0.414_214, 0.13).fract() - 0.5,
            ) * (SCREEN - MqVec2::splat(16.0));
            let twinkle = n.mul_add(1.7, self.clock * 2.4).sin().mul_add(0.5, 0.5);
            frame.circle(
                at(SCREEN_CENTRE + spot),
                f32::mul_add(twinkle, 0.6, 1.0) * scale,
                with_alpha(brand::ICE, twinkle.mul_add(0.5, 0.15)),
            );
        }

        games::emblem(
            id,
            frame,
            at(SCREEN_CENTRE - MqVec2::new(0.0, 6.0)),
            SCREEN.x * 0.62 * scale,
            self.clock,
        );
        frame.rect_lines_centred(
            at(SCREEN_CENTRE),
            size,
            1.0 * scale,
            with_alpha(brand::ICE, 0.6),
        );

        if chosen {
            // Corner brackets, closing in from outside as the window rises.
            let colour = with_alpha(brand::BANANA, lift);
            let inset = (1.0 - lift).mul_add(-14.0, 7.0);
            let arm = 16.0;
            for (sx, sy) in [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
                let sign = MqVec2::new(sx, sy);
                let point = SCREEN_CENTRE + (SCREEN * 0.5 - MqVec2::splat(inset)) * sign;
                frame.line(
                    at(point),
                    at(point - MqVec2::new(arm * sx, 0.0)),
                    2.5 * scale,
                    colour,
                );
                frame.line(
                    at(point),
                    at(point - MqVec2::new(0.0, arm * sy)),
                    2.5 * scale,
                    colour,
                );
            }
        }
    }

    /// What to press, as chips: the arrows once there is somewhere to move
    /// to, and the return key, which is the one thing being asked for.
    /// Pressing a key presses its cap.
    fn hints(&self, frame: &Frame) {
        let appear = ease_out_cubic(unit((self.shown - 0.3) / 0.4));
        let y = (1.0 - appear).mul_add(24.0, ui::FRAME_H - 44.0);
        let centre = ui::FRAME_W * 0.5;

        if Menu::len() > 1 {
            let pressed = self.arrows_pressed < 0.14;
            for (dx, dir) in [(-122.0, -1.0), (-86.0, 1.0)] {
                let at = MqVec2::new(centre + dx, y);
                let ink = key_chip(frame, at, MqVec2::splat(28.0), pressed, appear);
                frame.arrow_cap(
                    MqVec2::new(dir, 0.0),
                    frame.at(at + MqVec2::new(0.0, press_depth(pressed))),
                    24.0,
                    ink,
                );
            }
        }

        let pressed = self.enter_pressed < 0.14 || self.launch.is_some();
        let at = MqVec2::new(centre, y);
        let size = MqVec2::new(96.0, 28.0);
        let glow = pulse(self.clock);
        frame.rect_lines_centred(
            frame.at(at),
            size + MqVec2::splat(glow.mul_add(4.0, 8.0)),
            2.0,
            with_alpha(brand::ORANGE, glow.mul_add(0.4, 0.3) * appear),
        );
        let ink = key_chip(frame, at, size, pressed, appear);
        frame.enter_cap(
            frame.at(at + MqVec2::new(0.0, press_depth(pressed))),
            size - MqVec2::splat(6.0),
            ink,
        );
    }

    /// Which build this is, bottom left. A launcher is a developer's front
    /// door.
    fn version(&self, frame: &Frame) {
        let font = Some(&self.font);
        let size = 12.0;
        let appear = ease_out_cubic(unit((self.shown - 0.4) / 0.4));
        let width = frame.label_width(font, VERSION, size) + 16.0;
        let at = MqVec2::new(
            (1.0 - appear).mul_add(-width, width.mul_add(0.5, 14.0)),
            ui::FRAME_H - 20.0,
        );
        chip(frame, frame.at(at), MqVec2::new(width, 20.0), 1.0);
        frame.label(
            font,
            VERSION,
            frame.at(at + MqVec2::new(0.0, 1.0)),
            size,
            with_alpha(brand::INK, 0.8),
        );
    }

    /// The dive: on launch, ripples and an iris of night closing on the
    /// chosen screen; back out of a prototype, the same iris opening.
    fn dive(&self, frame: &Frame, menu: Menu) {
        let place = self.place(menu.index(), menu.index());
        let screen = place.centre + SCREEN_CENTRE * place.scale;

        if let Some((_, age)) = self.launch {
            for (delay, colour) in [(0.0, brand::ICE), (0.07, brand::MAGENTA)] {
                let t = unit((age - delay) / 0.4);
                if t > 0.0 && t < 1.0 {
                    frame.arc(
                        frame.at(screen),
                        ease_out_cubic(t) * 420.0,
                        3.0,
                        0.0,
                        360.0,
                        with_alpha(colour, 1.0 - t),
                    );
                }
            }
            let close = ease_in_cubic(unit((age - 0.12) / (LAUNCH_SECS - 0.14)));
            frame.poly(
                frame.at(screen),
                64,
                close * cover(screen),
                0.0,
                brand::NIGHT,
            );
        } else if self.from_game && self.shown < RETURN_SECS {
            iris(
                frame,
                screen,
                ease_out_cubic(unit(self.shown / RETURN_SECS)),
            );
        }
    }
}

/// The iris opening onto a prototype that has just loaded, laid over its
/// first frames. `age` is seconds since it started.
pub fn reveal(frame: &Frame, age: f32) {
    iris(
        frame,
        MqVec2::new(ui::FRAME_W * 0.5, ui::FRAME_H * 0.5),
        ease_out_cubic(unit(age / REVEAL_SECS)),
    );
}

/// Night everywhere outside a circle around `centre`, in frame units, open
/// by `open`: `0` is shut, `1` clears the whole frame.
fn iris(frame: &Frame, centre: MqVec2, open: f32) {
    if open >= 1.0 {
        return;
    }
    let reach = cover(centre);
    let radius = open * reach;
    frame.arc(frame.at(centre), radius, reach, 0.0, 360.0, brand::NIGHT);
    frame.arc(
        frame.at(centre),
        radius,
        3.0,
        0.0,
        360.0,
        with_alpha(brand::ICE, 1.0 - open),
    );
}

/// How far a circle around `centre` has to reach to cover the frame.
fn cover(centre: MqVec2) -> f32 {
    [
        MqVec2::ZERO,
        MqVec2::new(ui::FRAME_W, 0.0),
        MqVec2::new(0.0, ui::FRAME_H),
        MqVec2::new(ui::FRAME_W, ui::FRAME_H),
    ]
    .iter()
    .map(|&corner| corner.distance(centre))
    .fold(0.0, f32::max)
}

/// The page: wirenook's sky, a gradient at 155°.
fn sky(frame: &Frame) {
    const BANDS: usize = 6;
    let band = ui::FRAME_H / BANDS as f32;
    for i in 0..BANDS {
        let top = i as f32 * band;
        let bottom = top + band;
        frame.gradient(
            frame.at(MqVec2::new(0.0, top)),
            MqVec2::new(ui::FRAME_W, band),
            [
                sky_at(MqVec2::new(0.0, top)),
                sky_at(MqVec2::new(ui::FRAME_W, top)),
                sky_at(MqVec2::new(ui::FRAME_W, bottom)),
                sky_at(MqVec2::new(0.0, bottom)),
            ],
        );
    }
}

/// The sky's colour at a frame position: three stops along 155°.
fn sky_at(at: MqVec2) -> Color {
    let (dx, dy) = (0.423, 0.906);
    let span = ui::FRAME_W.mul_add(dx, ui::FRAME_H * dy);
    let t = at.x.mul_add(dx, at.y * dy) / span;
    if t < 0.48 {
        lerp_colour(brand::SKY_HIGH, brand::SKY_MID, t / 0.48)
    } else {
        lerp_colour(brand::SKY_MID, brand::SKY_LOW, (t - 0.48) / 0.52)
    }
}

/// One ruler tick: its length and colour, given its index and how much of
/// the cursor's bump it sits under.
fn tick(index: usize, bump: f32) -> (f32, Color) {
    let kind = if index % 10 == 0 {
        2
    } else {
        usize::from(index % 5 == 0)
    };
    let length = TICK_BUMP.mul_add(bump, TICK_LEN[kind]);
    let colour = lerp_colour(
        with_alpha(brand::ICE, TICK_ALPHA[kind]),
        with_alpha(brand::MAGENTA, 0.9),
        bump * 0.7,
    );
    (length, colour)
}

/// A top-line chip: solid panel, one-pixel line, a hard shadow.
fn chip(frame: &Frame, centre: MqVec2, size: MqVec2, scale: f32) {
    frame.rect_centred(
        centre + MqVec2::splat(3.0 * scale * frame.scale()),
        size,
        with_alpha(brand::VIOLET, 0.35),
    );
    frame.rect_centred(centre, size, brand::PANEL);
    frame.rect_lines_centred(centre, size, 1.0 * scale, brand::LINE);
}

/// A key cap's chip, pressed or not, at frame position `at`. Returns the
/// colour to draw the key's symbol in.
fn key_chip(frame: &Frame, at: MqVec2, size: MqVec2, pressed: bool, alpha: f32) -> Color {
    let depth = press_depth(pressed);
    frame.rect_centred(
        frame.at(at + MqVec2::new(0.0, 3.0)),
        size,
        with_alpha(brand::VIOLET, 0.45 * alpha),
    );
    let face = frame.at(at + MqVec2::new(0.0, depth));
    let (fill, ink) = if pressed {
        (brand::MAGENTA, brand::PANEL)
    } else {
        (brand::PANEL, brand::INK)
    };
    frame.rect_centred(face, size, with_alpha(fill, alpha));
    frame.rect_lines_centred(face, size, 1.0, with_alpha(brand::LINE, alpha));
    with_alpha(ink, alpha)
}

/// How far a pressed key cap sinks, in frame units.
const fn press_depth(pressed: bool) -> f32 {
    if pressed { 2.5 } else { 0.0 }
}

/// A heartbeat in `1..=1.45`: two quick swells and a rest, like the site's
/// status dots.
fn heartbeat(clock: f32) -> f32 {
    let t = (clock * 0.9).fract();
    let beat = |at: f32, height: f32| height * (-((t - at) * 22.0).powi(2)).exp();
    1.0 + beat(0.12, 0.45).max(beat(0.3, 0.25))
}

/// Move `current` toward `target`, covering most of the gap in about
/// `1 / rate` seconds whatever the frame rate.
fn approach(current: f32, target: f32, rate: f32, dt: f32) -> f32 {
    (target - current).mul_add(1.0 - (-rate * dt).exp(), current)
}

/// `t` clamped to `0..=1`.
const fn unit(t: f32) -> f32 {
    t.clamp(0.0, 1.0)
}

fn lerp(from: f32, to: f32, t: f32) -> f32 {
    (to - from).mul_add(t, from)
}

fn lerp_colour(from: Color, to: Color, t: f32) -> Color {
    let t = unit(t);
    Color::new(
        lerp(from.r, to.r, t),
        lerp(from.g, to.g, t),
        lerp(from.b, to.b, t),
        lerp(from.a, to.a, t),
    )
}

/// A bell curve of width `sigma`, `1` at the centre.
fn gauss(offset: f32, sigma: f32) -> f32 {
    (-(offset * offset) / (2.0 * sigma * sigma)).exp()
}

fn ease_out_cubic(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}

fn ease_in_cubic(t: f32) -> f32 {
    t.powi(3)
}

/// Overshoots a little and settles, for things arriving.
fn ease_out_back(t: f32) -> f32 {
    const C1: f32 = 1.701_58;
    const C3: f32 = C1 + 1.0;
    let u = t - 1.0;
    C3.mul_add(u.powi(3), C1 * u.powi(2)) + 1.0
}
