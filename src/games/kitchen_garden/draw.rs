//! Everything on screen, back to front.
//!
//! Reads the [`Game`] and never changes it. Positions come from
//! [`layout`](super::layout), so what is drawn is what the pointer hits.

// Drawing is points, radii and colours, over and over.
#![allow(clippy::many_single_char_names)]

use std::f32::consts::{PI, TAU};

use game_prototypes::cooking::{self, Drive, Food, Kind, Stage, recipe};
use game_prototypes::kitchen_garden::day::{
    self, Chore, Customer, MARKET, MARKS, PANTRY, PLOTS, Place, Plot, SEATS, StationId,
};
use game_prototypes::kitchen_garden::prices;
use macroquad::color::Color;
use macroquad::math::{Rect, Vec2, vec2};

use super::fx::Shape;
use super::icons;
use super::layout::{self, mid};
use super::pen::{Pen, alpha, dark, hex, ink, light, mix};
use super::{Game, Spot};
use crate::ui::{Frame, MqVec2, pulse};

/// How long the pointer rests on food before its recipe card shows.
const CARD_DELAY: f32 = 0.35;

pub fn scene(game: &Game, frame: &Frame) {
    let pen = Pen::new(frame).shifted(game.fx.offset(game.clock));
    sky(game, &pen);
    garden(game, &pen);
    kitchen(game, &pen);
    hatch(game, &pen);
    jar(&pen, game.day.coins(), game.clock);
    speaker(game, &pen);
    targets(game, &pen);
    particles(game, &pen);
    carried(game, &pen);
    card(game, &pen);
    if game.title {
        title(game, &pen);
    } else if game.day.is_over() {
        summary(game, &pen);
    }
}

/// A gentle up-and-down for things that want noticing.
fn bob(clock: f32, phase: f32) -> f32 {
    (clock.mul_add(3.2, phase)).sin() * 2.0
}

/// A squash-and-stretch for something just touched: 1 at rest.
fn squash(since: f32) -> f32 {
    if since > 0.4 {
        return 1.0;
    }
    let t = since / 0.4;
    (t * PI * 3.0).sin().mul_add((1.0 - t) * 0.16, 1.0)
}

// The sky and the ground.

fn sky_colours(daylight: f32, over: bool) -> (Color, Color) {
    if over {
        return (hex(0x1b_2448), hex(0x4a_3b6a));
    }
    let dawn = (hex(0xf7_c6a3), hex(0xfd_e8c8));
    let noon = (ink::SKY_HIGH, ink::SKY);
    let dusk = (hex(0x6a_5a9a), hex(0xf5_a26a));
    if daylight < 0.15 {
        let t = daylight / 0.15;
        (mix(dawn.0, noon.0, t), mix(dawn.1, noon.1, t))
    } else if daylight < 0.75 {
        noon
    } else {
        let t = (daylight - 0.75) / 0.25;
        (mix(noon.0, dusk.0, t), mix(noon.1, dusk.1, t))
    }
}

fn sky(game: &Game, pen: &Pen) {
    let t = if game.title { 0.0 } else { game.day.daylight() };
    let over = game.day.is_over();
    let (top, bottom) = sky_colours(t, over);
    pen.gradient(
        vec2(-20.0, -20.0),
        vec2(840.0, layout::SKY_H + 22.0),
        top,
        bottom,
    );
    // The hatch looks out on the same sky.
    pen.gradient(
        vec2(layout::HATCH_X, layout::SKY_H),
        vec2(130.0, 400.0),
        bottom,
        light(bottom, 0.3),
    );
    if over {
        for i in 0..40 {
            let x = (i as f32 * 97.3) % 800.0;
            let y = (i as f32 * 37.7) % (layout::SKY_H - 6.0);
            let tw = game.clock.mul_add(2.0, i as f32).sin().mul_add(0.3, 0.6);
            pen.sparkle(vec2(x, y + 3.0), 2.0, alpha(ink::WHITE, tw));
        }
    } else {
        // The sun crosses the sky with the day.
        let x = t.mul_add(720.0, 40.0);
        let y = (t * PI).sin().mul_add(-34.0, 52.0);
        let warm = mix(hex(0xff_f2b0), hex(0xff_a860), (t - 0.7).max(0.0) * 3.0);
        pen.circle(vec2(x, y), 22.0, alpha(warm, 0.25));
        pen.circle(vec2(x, y), 15.0, warm);
        pen.circle(vec2(x - 4.0, y - 4.0), 5.0, alpha(ink::WHITE, 0.5));
    }
    // Clouds drift by.
    for i in 0..4 {
        let fi = i as f32;
        let x = (game.clock.mul_add(fi.mul_add(2.0, 6.0), fi * 230.0)) % 980.0 - 90.0;
        let y = (fi * 13.0) % 30.0 + 14.0;
        let c = alpha(if over { hex(0x6a_6a9a) } else { ink::WHITE }, 0.85);
        pen.ellipse(vec2(x, y), vec2(26.0, 9.0), 0.0, c);
        pen.circle(vec2(x - 9.0, y - 5.0), 9.0, c);
        pen.circle(vec2(x + 7.0, y - 7.0), 11.0, c);
    }
    // The far hills.
    let hill = if over { hex(0x2e_3f4a) } else { hex(0x7f_b36a) };
    for i in 0..9 {
        let x = (i as f32).mul_add(100.0, -20.0);
        pen.ellipse(vec2(x, layout::SKY_H + 4.0), vec2(80.0, 16.0), 0.0, hill);
    }
}

fn garden(game: &Game, pen: &Pen) {
    let ground = Rect::new(0.0, layout::SKY_H, layout::GARDEN_W, 600.0 - layout::SKY_H);
    pen.rect(
        vec2(ground.x, ground.y),
        vec2(ground.w, ground.h),
        ink::GRASS,
    );
    // Tufts, fixed.
    for i in 0..60 {
        let x = (i as f32 * 53.7) % 290.0 + 5.0;
        let y = (i as f32 * 91.3) % 520.0 + 70.0;
        let c = if i % 3 == 0 {
            ink::GRASS_LIGHT
        } else {
            ink::GRASS_DARK
        };
        pen.tri(vec2(x - 2.0, y), vec2(x, y - 6.0), vec2(x + 2.0, y), c);
        pen.tri(
            vec2(x + 1.0, y),
            vec2(x + 4.0, y - 4.0),
            vec2(x + 4.0, y),
            c,
        );
    }
    // A path down the middle.
    pen.rrect(
        vec2(140.0, 280.0),
        vec2(160.0, 18.0),
        9.0,
        alpha(ink::SOIL, 0.25),
    );
    for i in 0..PLOTS {
        plot(game, pen, i);
    }
    hen(game, pen);
    cow(game, pen);
    well(game, pen);
    stump(game, pen);
    compost(game, pen);
}

fn plot(game: &Game, pen: &Pen, i: usize) {
    let r = layout::plot(i);
    let plot = game.day.plots()[i];
    let rich = matches!(plot, Plot::Growing { rich: true, .. });
    let soil = if rich { ink::SOIL_RICH } else { ink::SOIL };
    pen.rrect(vec2(r.x, r.y + 4.0), vec2(r.w, r.h), 10.0, ink::SOIL_DARK);
    pen.rrect(vec2(r.x, r.y), vec2(r.w, r.h), 10.0, soil);
    for row in 0..3 {
        let y = (row as f32).mul_add(18.0, r.y + 14.0);
        pen.capsule(
            vec2(r.x + 12.0, y),
            vec2(r.x + r.w - 12.0, y),
            3.0,
            dark(soil, 0.25),
        );
    }
    let squish = squash(game.bumped(Spot::Place(Place::Plot(i))));
    match plot {
        Plot::Empty => {
            // A dotted outline asks for a seed.
            let c = alpha(ink::CREAM, 0.35);
            for k in 0..8 {
                let x = (k as f32).mul_add(14.0, r.x + 16.0);
                pen.circle(vec2(x, r.y + r.h - 8.0), 1.5, c);
            }
        }
        Plot::Growing { crop, .. } => {
            let g = plot.growth();
            for k in 0..3 {
                let x = (k as f32).mul_add(38.0, r.x + 27.0);
                let y = r.y + 46.0;
                let sway = (game.clock.mul_add(1.7, k as f32) + i as f32).sin() * 2.0 * g;
                sprout(pen, crop, vec2(x, y), g * squish, sway);
            }
        }
        Plot::Ripe { crop, left } => {
            for k in 0..usize::from(left).min(4) {
                let x = (k as f32).mul_add(30.0, r.x + 20.0);
                let y = bob(game.clock, k as f32).mul_add(0.5, r.y + 34.0);
                sprout(pen, crop, vec2(x, y + 14.0), 1.0, 0.0);
                pen.shadow(vec2(x, y + 14.0), 22.0);
                icons::food(pen, crop, vec2(x, y), 28.0 * squish);
            }
        }
    }
}

/// Leaves coming up, `g` of the way grown.
fn sprout(pen: &Pen, crop: Food, base: Vec2, g: f32, sway: f32) {
    let g = g.clamp(0.0, 1.0);
    let h = 18.0f32.mul_add(g, 6.0);
    let tip = base + vec2(sway, -h);
    let leaf = match crop {
        Food::Wheat | Food::Rice => hex(0x9c_b84a),
        Food::TeaLeaf => ink::LEAF_DARK,
        _ => ink::LEAF,
    };
    pen.line(base, tip, 2.0, dark(leaf, 0.2));
    let size = 6.0f32.mul_add(g, 3.0);
    pen.ellipse(
        tip + vec2(-size * 0.6, size * 0.3),
        vec2(size, size * 0.45),
        -0.5,
        leaf,
    );
    pen.ellipse(
        tip + vec2(size * 0.6, size * 0.5),
        vec2(size, size * 0.45),
        0.5,
        light(leaf, 0.15),
    );
    if g > 0.6 && matches!(crop, Food::Wheat | Food::Rice) {
        pen.ellipse(tip, vec2(2.5, 5.0), 0.0, ink::WHEAT);
    }
}

fn hen(game: &Game, pen: &Pen) {
    let r = layout::HEN;
    let peck = game.clock.mul_add(1.3, 0.0).sin().max(0.0).powi(8) * 6.0;
    let s = squash(game.bumped(Spot::Hen));
    let body = vec2(r.x + 38.0, r.y + 38.0);
    pen.shadow(body + vec2(0.0, 22.0), 50.0);
    pen.ellipse(body, vec2(22.0, 17.0) * s, 0.0, ink::WHITE);
    pen.ellipse(body + vec2(-14.0, -6.0), vec2(9.0, 7.0), -0.6, ink::CREAM);
    let head = body + vec2(18.0, -16.0 + peck);
    pen.circle(head, 9.0, ink::WHITE);
    pen.circle(head + vec2(-2.0, -9.0), 4.0, ink::TOMATO);
    pen.circle(head + vec2(2.0, -9.0), 3.5, ink::TOMATO);
    pen.tri(
        head + vec2(8.0, -2.0),
        head + vec2(15.0, 1.0),
        head + vec2(8.0, 4.0),
        ink::YOLK,
    );
    pen.circle(head + vec2(3.0, -1.0), 1.6, ink::LINE);
    pen.ellipse(head + vec2(6.0, 6.0), vec2(2.0, 3.0), 0.0, ink::TOMATO);
    for dx in [-6.0, 4.0] {
        pen.line(body + vec2(dx, 14.0), body + vec2(dx, 22.0), 1.6, ink::YOLK);
    }
    // The nest, and whatever she has laid.
    let n = layout::NEST;
    let ns = squash(game.bumped(Spot::Place(Place::Nest)));
    pen.ellipse(
        mid(n) + vec2(0.0, 6.0),
        vec2(36.0, 12.0),
        0.0,
        ink::WOOD_DARK,
    );
    pen.ellipse(mid(n) + vec2(0.0, 2.0), vec2(34.0, 10.0), 0.0, ink::WHEAT);
    for k in 0..game.day.eggs() {
        let x = 0.5f32
            .mul_add(-f32::from(game.day.eggs() - 1), f32::from(k))
            .mul_add(22.0, mid(n).x);
        icons::food(pen, Food::Egg, vec2(x, mid(n).y - 6.0), 24.0 * ns);
    }
    for k in 0..9 {
        let x = (k as f32).mul_add(7.5, n.x + 9.0);
        pen.line(
            vec2(x, n.y + 18.0),
            vec2(x + 5.0, n.y + 26.0),
            1.5,
            dark(ink::WHEAT, 0.2),
        );
    }
}

fn cow(game: &Game, pen: &Pen) {
    let r = layout::COW;
    let s = squash(game.bumped(Spot::Chore(Chore::Milk)));
    let body = vec2(r.x + 62.0, r.y + 48.0);
    pen.shadow(body + vec2(0.0, 44.0), 120.0);
    // Legs.
    for dx in [-38.0, -22.0, 22.0, 36.0] {
        pen.capsule(
            body + vec2(dx, 18.0),
            body + vec2(dx, 40.0),
            5.0,
            ink::WHITE,
        );
        pen.circle(body + vec2(dx, 41.0), 5.0, ink::LINE);
    }
    pen.rrect_centred(body, vec2(104.0, 52.0) * vec2(1.0, s), 24.0, ink::WHITE);
    // Patches.
    pen.ellipse(body + vec2(-22.0, -8.0), vec2(16.0, 12.0), 0.3, ink::LINE);
    pen.ellipse(body + vec2(20.0, 6.0), vec2(12.0, 9.0), -0.4, ink::LINE);
    // Udder, fuller when there is milk in it.
    let full = f32::from(game.day.udder()) / f32::from(day::UDDER_MAX);
    pen.ellipse(
        body + vec2(8.0, 26.0),
        vec2(5.0f32.mul_add(full, 9.0), 3.0f32.mul_add(full, 6.0)),
        0.0,
        hex(0xf2_a7b5),
    );
    // Head, looking out at you.
    let nod = (game.clock * 0.9).sin() * 1.5;
    let head = body + vec2(-56.0, -14.0 + nod);
    pen.ellipse(head + vec2(-6.0, -14.0), vec2(9.0, 4.0), -0.5, ink::WHITE);
    pen.ellipse(head + vec2(14.0, -14.0), vec2(9.0, 4.0), 0.5, ink::WHITE);
    pen.ellipse(head, vec2(17.0, 20.0), 0.0, ink::WHITE);
    pen.ellipse(
        head + vec2(0.0, 11.0),
        vec2(14.0, 10.0),
        0.0,
        hex(0xf2_a7b5),
    );
    pen.circle(head + vec2(-5.0, 11.0), 2.0, dark(hex(0xf2_a7b5), 0.4));
    pen.circle(head + vec2(5.0, 11.0), 2.0, dark(hex(0xf2_a7b5), 0.4));
    let blink = if (game.clock * 0.5).fract() < 0.05 {
        0.3
    } else {
        2.4
    };
    pen.ellipse(head + vec2(-7.0, -3.0), vec2(2.4, blink), 0.0, ink::LINE);
    pen.ellipse(head + vec2(7.0, -3.0), vec2(2.4, blink), 0.0, ink::LINE);
    pen.capsule(
        head + vec2(-8.0, -20.0),
        head + vec2(-12.0, -26.0),
        2.5,
        ink::CREAM,
    );
    pen.capsule(
        head + vec2(8.0, -20.0),
        head + vec2(12.0, -26.0),
        2.5,
        ink::CREAM,
    );
    // Tail.
    let wag = (game.clock * 2.3).sin() * 6.0;
    pen.line(
        body + vec2(50.0, -10.0),
        body + vec2(58.0, 14.0 + wag * 0.3),
        2.0,
        ink::LINE,
    );
    pail(game, pen);
    stroke_pips(pen, game, Chore::Milk, vec2(r.x + 30.0, r.y - 2.0));
}

fn pail(game: &Game, pen: &Pen) {
    let p = layout::PAIL;
    let ps = squash(game.bumped(Spot::Place(Place::Pail)));
    let pc = mid(p) + vec2(0.0, 6.0);
    pen.shadow(pc + vec2(0.0, 18.0), 44.0);
    pen.poly(
        &[
            pc + vec2(-18.0, -12.0 * ps),
            pc + vec2(18.0, -12.0 * ps),
            pc + vec2(14.0, 18.0),
            pc + vec2(-14.0, 18.0),
        ],
        ink::STONE_LIGHT,
    );
    pen.ellipse(
        pc + vec2(0.0, -12.0 * ps),
        vec2(18.0, 5.0),
        0.0,
        ink::STONE_DARK,
    );
    if game.day.pail() > 0 {
        let milk = f32::from(game.day.pail()) / f32::from(day::WAITING_MAX);
        pen.ellipse(
            pc + vec2(0.0, -11.0 * ps),
            vec2(16.0 * milk.max(0.6), 4.0),
            0.0,
            ink::MILK,
        );
    }
    pen.arc(
        pc + vec2(0.0, -12.0),
        18.0,
        1.6,
        180.0,
        180.0,
        ink::IRON_LIGHT,
    );
    for k in 0..game.day.pail() {
        icons::food(
            pen,
            Food::Milk,
            pc + vec2(f32::from(k).mul_add(14.0, -7.0), -30.0),
            20.0,
        );
    }
}

/// Dots that fill as a chore's strokes add up toward its next result.
fn stroke_pips(pen: &Pen, game: &Game, chore: Chore, at: Vec2) {
    let (done, goal) = game.day.chore_progress(chore);
    if done == 0 {
        return;
    }
    for k in 0..goal {
        let c = vec2(f32::from(k).mul_add(10.0, at.x), at.y);
        pen.circle(c, 4.0, alpha(ink::LINE, 0.4));
        if k < done {
            pen.circle(c, 3.0, ink::GLOW);
        }
    }
}

fn well(game: &Game, pen: &Pen) {
    let r = layout::WELL;
    let c = vec2(r.x + 50.0, r.y + 96.0);
    let s = squash(game.bumped(Spot::Chore(Chore::Crank)));
    // Posts and roof.
    pen.rect(
        vec2(c.x - 40.0, c.y - 84.0),
        vec2(6.0, 70.0),
        ink::WOOD_DARK,
    );
    pen.rect(
        vec2(c.x + 34.0, c.y - 84.0),
        vec2(6.0, 70.0),
        ink::WOOD_DARK,
    );
    pen.poly(
        &[
            vec2(c.x - 52.0, c.y - 78.0),
            vec2(c.x, c.y - 100.0),
            vec2(c.x + 52.0, c.y - 78.0),
        ],
        ink::BRICK,
    );
    pen.line(
        vec2(c.x - 52.0, c.y - 78.0),
        vec2(c.x + 52.0, c.y - 78.0),
        3.0,
        ink::BRICK_DARK,
    );
    // The crank, turning with each stroke.
    let (done, goal) = game.day.chore_progress(Chore::Crank);
    let turn = (f32::from(done) / f32::from(goal))
        .mul_add(TAU, game.bumped(Spot::Chore(Chore::Crank)).min(0.3) * 8.0);
    let axle = vec2(c.x, c.y - 56.0);
    pen.capsule(
        axle - vec2(36.0, 0.0),
        axle + vec2(36.0, 0.0),
        3.0,
        ink::WOOD,
    );
    let handle = axle + vec2(44.0, 0.0) + Vec2::from_angle(turn) * 8.0;
    pen.line(axle + vec2(40.0, 0.0), handle, 3.0, ink::IRON_LIGHT);
    pen.circle(handle, 3.5, ink::WOOD_DARK);
    pen.line(axle, axle + vec2(0.0, 26.0), 1.2, ink::LINE);
    // The stone ring.
    pen.shadow(c + vec2(0.0, 30.0), 100.0);
    pen.rrect_centred(c + vec2(0.0, 8.0), vec2(90.0 * s, 44.0), 10.0, ink::STONE);
    pen.ellipse(
        c + vec2(0.0, -14.0),
        vec2(45.0, 12.0),
        0.0,
        ink::STONE_LIGHT,
    );
    pen.ellipse(c + vec2(0.0, -14.0), vec2(36.0, 8.0), 0.0, ink::WATER_DARK);
    for k in 0..4 {
        let x = (k as f32).mul_add(22.0, c.x - 33.0);
        pen.rrect(vec2(x, c.y + 2.0), vec2(18.0, 10.0), 3.0, ink::STONE_DARK);
    }
    // The bucket.
    let b = layout::BUCKET;
    let bs = squash(game.bumped(Spot::Place(Place::Bucket)));
    let bc = mid(b) + vec2(0.0, 6.0);
    pen.shadow(bc + vec2(0.0, 16.0), 40.0);
    pen.poly(
        &[
            bc + vec2(-16.0, -10.0 * bs),
            bc + vec2(16.0, -10.0 * bs),
            bc + vec2(12.0, 16.0),
            bc + vec2(-12.0, 16.0),
        ],
        ink::WOOD,
    );
    pen.line(bc + vec2(-14.0, 0.0), bc + vec2(14.0, 0.0), 2.0, ink::IRON);
    pen.ellipse(
        bc + vec2(0.0, -10.0 * bs),
        vec2(16.0, 4.5),
        0.0,
        ink::WOOD_DARK,
    );
    if game.day.bucket() > 0 {
        pen.ellipse(bc + vec2(0.0, -9.5 * bs), vec2(13.0, 3.5), 0.0, ink::WATER);
    }
    for k in 0..game.day.bucket() {
        icons::food(
            pen,
            Food::Water,
            bc + vec2(f32::from(k).mul_add(14.0, -7.0), -26.0),
            18.0,
        );
    }
    stroke_pips(pen, game, Chore::Crank, vec2(r.x + 36.0, r.y - 2.0));
}

fn stump(game: &Game, pen: &Pen) {
    let r = layout::STUMP;
    let c = vec2(r.x + 36.0, r.y + 58.0);
    pen.shadow(c + vec2(0.0, 18.0), 70.0);
    pen.rrect_centred(c, vec2(54.0, 36.0), 6.0, ink::WOOD);
    pen.ellipse(c - vec2(0.0, 18.0), vec2(27.0, 9.0), 0.0, ink::WOOD_LIGHT);
    pen.ring(c - vec2(0.0, 18.0), 14.0, 1.4, ink::WOOD);
    pen.ring(c - vec2(0.0, 18.0), 7.0, 1.4, ink::WOOD);
    // A log waiting on the block, and the axe, which swings on a stroke.
    let since = game.bumped(Spot::Chore(Chore::Axe));
    let swing = if since < 0.18 {
        (1.0 - since / 0.18) * 1.2
    } else {
        0.0
    };
    let pivot = c + vec2(18.0, -22.0);
    let dir = Vec2::from_angle(-2.2 + swing);
    let head = pivot + dir * 34.0;
    pen.capsule(pivot, head, 2.6, ink::WOOD_DARK);
    let across = vec2(-dir.y, dir.x);
    pen.poly(
        &[
            head - across * 2.0,
            head + across * 10.0 - dir * 4.0,
            head + across * 12.0 + dir * 6.0,
            head + dir * 6.0,
        ],
        ink::IRON_LIGHT,
    );
    stroke_pips(pen, game, Chore::Axe, vec2(r.x + 22.0, r.y + 4.0));
}

fn compost(game: &Game, pen: &Pen) {
    let r = layout::COMPOST;
    let c = vec2(r.x + 30.0, r.y + 52.0);
    let s = squash(game.bumped(Spot::Place(Place::Compost)));
    pen.shadow(c + vec2(0.0, 12.0), 58.0);
    // A little wattle pen with a heap that grows.
    let heap = 7.0f32.mul_add(f32::from(game.day.compost()), 10.0);
    pen.ellipse(
        c - vec2(0.0, 4.0),
        vec2(24.0, heap * 0.6 * s),
        0.0,
        ink::SOIL_RICH,
    );
    pen.circle(c + vec2(-8.0, -6.0 - heap * 0.3), 4.0, dark(ink::LEAF, 0.3));
    pen.circle(c + vec2(6.0, -4.0 - heap * 0.4), 3.0, ink::BEAN);
    for k in 0..6 {
        let x = (k as f32).mul_add(10.0, c.x - 26.0);
        pen.capsule(vec2(x, c.y - 18.0), vec2(x, c.y + 8.0), 2.0, ink::WOOD);
    }
    pen.capsule(
        vec2(c.x - 28.0, c.y - 8.0),
        vec2(c.x + 26.0, c.y - 8.0),
        1.6,
        ink::WOOD_LIGHT,
    );
    for k in 0..game.day.compost() {
        let x = f32::from(k).mul_add(12.0, c.x - 12.0);
        pen.circle(vec2(x, c.y - 34.0), 4.0, alpha(ink::GLOW, 0.9));
        pen.sparkle(vec2(x, c.y - 34.0), 5.0, ink::WHITE);
    }
}

// The kitchen.

fn kitchen(game: &Game, pen: &Pen) {
    // Wall, beams and a wainscot.
    pen.rect(
        vec2(layout::GARDEN_W, layout::SKY_H),
        vec2(layout::HATCH_X - layout::GARDEN_W, 600.0),
        ink::WALL,
    );
    for k in 0..4 {
        let x = (k as f32).mul_add(130.0, 300.0);
        pen.rect(vec2(x, layout::SKY_H), vec2(10.0, 50.0), ink::WALL_DARK);
    }
    pen.rect(
        vec2(layout::GARDEN_W, layout::SKY_H),
        vec2(390.0, 10.0),
        ink::WOOD_DARK,
    );
    pen.rect(vec2(layout::GARDEN_W, 438.0), vec2(390.0, 162.0), ink::WOOD);
    for k in 0..10 {
        let x = (k as f32).mul_add(40.0, 302.0);
        pen.line(vec2(x, 438.0), vec2(x, 600.0), 1.5, ink::WOOD_DARK);
    }
    // The timber post between garden and kitchen.
    pen.rect(
        vec2(layout::GARDEN_W - 4.0, layout::SKY_H),
        vec2(8.0, 540.0),
        ink::WOOD_DARK,
    );
    hearth(game, pen);
    counter(game, pen);
    pantry(game, pen);
    market(game, pen);
    for id in StationId::ALL {
        station(game, pen, id);
    }
}

fn hearth(game: &Game, pen: &Pen) {
    let r = layout::HEARTH;
    pen.panel(
        vec2(r.x, r.y),
        vec2(r.w, r.h),
        8.0,
        3.0,
        ink::BRICK,
        ink::BRICK_DARK,
    );
    for row in 0..8 {
        let y = (row as f32).mul_add(25.0, r.y + 4.0);
        pen.line(
            vec2(r.x + 3.0, y),
            vec2(r.x + r.w - 3.0, y),
            1.2,
            ink::BRICK_DARK,
        );
        let shift = if row % 2 == 0 { 0.0 } else { 18.0 };
        for k in 0..6 {
            let x = (k as f32).mul_add(36.0, r.x + 10.0 + shift);
            if x < r.x + r.w - 6.0 {
                pen.line(vec2(x, y), vec2(x, y + 25.0), 1.2, ink::BRICK_DARK);
            }
        }
    }
    // The hob the pot and pan sit on.
    pen.rrect(
        vec2(r.x - 4.0, 206.0),
        vec2(r.w + 8.0, 12.0),
        4.0,
        ink::IRON,
    );
    // The firebox.
    let f = layout::FIREBOX;
    pen.rrect(vec2(f.x, f.y), vec2(f.w, f.h), 30.0, hex(0x2a_1c18));
    let fire = game.day.fire();
    let lit = fire > 0.0;
    if lit {
        let glow = 0.3f32.mul_add(pulse(game.clock * 2.0), 0.5);
        pen.rrect(
            vec2(f.x + 6.0, f.y + 6.0),
            vec2(f.w - 12.0, f.h - 10.0),
            26.0,
            alpha(ink::EMBER, glow * 0.5),
        );
    }
    // The log burning, and its flames.
    let base = vec2(mid(f).x, f.y + f.h - 14.0);
    if lit || game.day.logs() > 0 {
        pen.capsule(
            base + vec2(-40.0, 2.0),
            base + vec2(30.0, -2.0),
            7.0,
            ink::WOOD_DARK,
        );
        pen.capsule(
            base + vec2(-26.0, -6.0),
            base + vec2(42.0, 2.0),
            7.0,
            ink::WOOD,
        );
    }
    if lit {
        let h = 26.0f32.mul_add(fire.max(0.25), 18.0);
        for k in 0..5 {
            let x = (k as f32).mul_add(18.0, base.x - 36.0);
            let flick = (k as f32).mul_add(1.7, game.clock * 9.0).sin() * 4.0;
            let hk = h * 0.3f32.mul_add((k as f32 * 2.1).sin().abs(), 0.7);
            flame(pen, vec2(x, base.y - 4.0), hk + flick, 9.0);
        }
    }
    // The woodpile under the hatch side.
    let logs = layout::LOGS;
    for k in 0..usize::from(game.day.logs()) {
        let col = k % 6;
        let x = (col as f32).mul_add(34.0, logs.x + 16.0);
        let y = logs.y + 6.0;
        pen.capsule(vec2(x - 12.0, y), vec2(x + 12.0, y), 6.0, ink::WOOD);
        pen.circle(vec2(x + 12.0, y), 6.0, ink::WOOD_LIGHT);
        pen.ring(vec2(x + 12.0, y), 3.0, 1.0, ink::WOOD);
    }
}

/// One tongue of flame.
fn flame(pen: &Pen, base: Vec2, h: f32, w: f32) {
    pen.poly(
        &[
            base + vec2(-w, 0.0),
            base + vec2(0.0, -h),
            base + vec2(w, 0.0),
        ],
        ink::FLAME,
    );
    pen.circle(base, w, ink::FLAME);
    pen.poly(
        &[
            base + vec2(-w * 0.5, 0.0),
            base + vec2(0.0, -h * 0.6),
            base + vec2(w * 0.5, 0.0),
        ],
        ink::FLAME_HOT,
    );
    pen.circle(base, w * 0.5, ink::FLAME_HOT);
}

fn counter(_game: &Game, pen: &Pen) {
    let c = layout::COUNTER;
    pen.rrect(vec2(c.x, c.y + 4.0), vec2(c.w, c.h), 4.0, ink::WOOD_DARK);
    pen.rrect(vec2(c.x, c.y), vec2(c.w, c.h - 6.0), 4.0, ink::WOOD_LIGHT);
    pen.line(
        vec2(c.x, c.y + 8.0),
        vec2(c.x + c.w, c.y + 8.0),
        1.2,
        ink::WOOD,
    );
}

fn pantry(game: &Game, pen: &Pen) {
    // Two shelves of four.
    for row in 0..2 {
        let y = (row as f32).mul_add(72.0, 452.0 + 60.0);
        pen.rrect(vec2(306.0, y), vec2(258.0, 8.0), 2.0, ink::WOOD_DARK);
        pen.rect(vec2(306.0, y), vec2(258.0, 3.0), ink::WOOD_LIGHT);
    }
    for i in 0..PANTRY {
        let r = layout::shelf(i);
        let c = mid(r);
        pen.rrect(
            vec2(r.x + 2.0, r.y + 2.0),
            vec2(r.w - 4.0, r.h - 6.0),
            8.0,
            alpha(ink::WOOD_DARK, 0.25),
        );
        let Some((food, n)) = game.day.pantry()[i] else {
            continue;
        };
        let lifted = game.carry.is_some_and(|c| c.from == Place::Shelf(i));
        let shown = if lifted { n - 1 } else { n };
        let s = squash(game.bumped(Spot::Place(Place::Shelf(i))));
        for k in 0..shown {
            let off = vec2(
                f32::from(k).mul_add(5.0, -f32::from(shown - 1) * 2.5),
                -f32::from(k) * 4.0,
            );
            pen.shadow(c + vec2(off.x, 24.0), 34.0);
            icons::food(pen, food, c + off + vec2(0.0, 6.0), 36.0 * s);
        }
        if shown > 1 {
            // Tally pips, so a full stack reads at a glance.
            for k in 0..shown {
                pen.circle(
                    vec2(f32::from(k).mul_add(7.0, r.x + 8.0), r.y + 8.0),
                    2.5,
                    ink::CREAM,
                );
            }
        }
    }
}

fn market(game: &Game, pen: &Pen) {
    let first = layout::ware(0);
    pen.panel(
        vec2(first.x - 4.0, first.y - 6.0),
        vec2(first.w + 8.0, 152.0),
        8.0,
        3.0,
        ink::WOOD_LIGHT,
        ink::WOOD_DARK,
    );
    for (i, &food) in MARKET.iter().enumerate() {
        let r = layout::ware(i);
        let price = prices::market_price(food).unwrap_or(0);
        let afford = price <= game.day.coins();
        let s = squash(game.bumped(Spot::Place(Place::Market(food))));
        pen.rrect(
            vec2(r.x + 2.0, r.y + 2.0),
            vec2(r.w - 4.0, r.h - 4.0),
            6.0,
            alpha(ink::WOOD, 0.5),
        );
        icons::food(pen, food, vec2(r.x + 26.0, mid(r).y), 34.0 * s);
        for k in 0..price {
            let c = vec2((k as f32).mul_add(18.0, r.x + 62.0), mid(r).y);
            pen.coin(c, 7.0);
        }
        if !afford {
            pen.rrect(
                vec2(r.x + 2.0, r.y + 2.0),
                vec2(r.w - 4.0, r.h - 4.0),
                6.0,
                alpha(ink::LINE, 0.4),
            );
        }
    }
}

fn station(game: &Game, pen: &Pen, id: StationId) {
    let r = layout::station(id);
    let s = squash(game.bumped(Spot::Place(Place::Station(id))));
    let st = game.day.station(id);
    match id {
        StationId::Pot => pot(game, pen, r, s),
        StationId::Pan => pan(game, pen, r, s),
        StationId::Oven => oven(game, pen, r, s),
        StationId::Board => board(game, pen, r, s),
        StationId::Quern => quern(game, pen, r, s),
        StationId::Bowl => bowl(pen, r, s),
        StationId::Crock => crock(game, pen, r, s),
    }
    // What is in it, floating where it can be seen.
    let lifted = game.carry.is_some_and(|c| c.from == Place::Station(id));
    let contents = st.contents();
    let n = contents.len() - usize::from(lifted && st.is_idle() && !contents.is_empty());
    let spot = contents_spot(id, r);
    for (k, &food) in contents.iter().take(n).enumerate() {
        let spread = (n as f32 - 1.0).mul_add(-0.5, k as f32) * 22.0;
        let wobble = if st.is_idle() {
            0.0
        } else {
            bob(game.clock * 1.5, k as f32)
        };
        icons::food(pen, food, spot + vec2(spread, wobble), 28.0 * s);
    }
    // What came out of it.
    if let Some(food) = st.ready() {
        if !lifted {
            let scorch = st.scorch();
            let lift = bob(game.clock, 0.0) - 6.0;
            let at = spot + vec2(0.0, lift);
            if food != Food::Charcoal && food != Food::Mush {
                let g = 0.3f32.mul_add(pulse(game.clock), 0.4);
                pen.circle(at, 26.0, alpha(ink::GLOW, g * (1.0 - scorch)));
            }
            icons::food(pen, food, at, 42.0 * s);
            if scorch > 0.0 {
                pen.circle(at, 20.0, alpha(ink::LINE, scorch * 0.45));
            }
        }
    }
    badge(game, pen, id, r);
    hint(game, pen, id, r);
}

/// Where a station's contents float.
fn contents_spot(id: StationId, r: Rect) -> Vec2 {
    match id {
        StationId::Pot => vec2(mid(r).x, r.y + 26.0),
        StationId::Pan => vec2(mid(r).x - 10.0, r.y + 30.0),
        StationId::Oven => vec2(mid(r).x, r.y + 120.0),
        StationId::Board | StationId::Quern | StationId::Bowl | StationId::Crock => {
            vec2(mid(r).x, r.y + 28.0)
        }
    }
}

fn pot(game: &Game, pen: &Pen, r: Rect, s: f32) {
    let c = vec2(mid(r).x, r.y + 72.0);
    let working = !game.day.station(StationId::Pot).is_idle();
    pen.capsule(
        c + vec2(-46.0, -14.0),
        c + vec2(-38.0, -14.0),
        5.0,
        ink::IRON,
    );
    pen.capsule(c + vec2(38.0, -14.0), c + vec2(46.0, -14.0), 5.0, ink::IRON);
    pen.rrect_centred(c + vec2(0.0, 8.0), vec2(80.0 * s, 52.0), 18.0, ink::IRON);
    pen.rrect_centred(c + vec2(-16.0, 4.0), vec2(14.0, 30.0), 7.0, ink::IRON_LIGHT);
    pen.ellipse(
        c - vec2(0.0, 18.0),
        vec2(42.0 * s, 10.0),
        0.0,
        ink::IRON_LIGHT,
    );
    let water = if working {
        light(ink::WATER, 0.25)
    } else {
        ink::WATER_DARK
    };
    pen.ellipse(c - vec2(0.0, 18.0), vec2(36.0 * s, 7.0), 0.0, water);
    if working {
        for k in 0..4 {
            let t = (k as f32).mul_add(0.37, game.clock * 1.8).fract();
            let x = (k as f32 * 2.3).sin().mul_add(22.0, c.x);
            pen.circle(
                vec2(x, c.y - 18.0 - t * 3.0),
                2.0 + t * 2.0,
                alpha(ink::WHITE, 1.0 - t),
            );
        }
    }
}

fn pan(game: &Game, pen: &Pen, r: Rect, s: f32) {
    let c = vec2(mid(r).x - 10.0, r.y + 56.0);
    let working = !game.day.station(StationId::Pan).is_idle();
    pen.capsule(
        c + vec2(38.0, -2.0),
        c + vec2(66.0, -10.0),
        4.0,
        ink::WOOD_DARK,
    );
    pen.ellipse(c + vec2(0.0, 4.0), vec2(42.0 * s, 13.0), 0.0, ink::IRON);
    pen.ellipse(c, vec2(40.0 * s, 11.0), 0.0, ink::IRON_LIGHT);
    pen.ellipse(
        c,
        vec2(34.0 * s, 8.0),
        0.0,
        if working { hex(0x8a_6a3a) } else { ink::IRON },
    );
    if working {
        for k in 0..5 {
            let t = (k as f32).mul_add(0.29, game.clock * 3.0).fract();
            let x = (k as f32 * 1.9).cos().mul_add(26.0, c.x);
            pen.circle(vec2(x, c.y - t * 10.0), 1.4, alpha(ink::FLAME_HOT, 1.0 - t));
        }
    }
}

fn oven(game: &Game, pen: &Pen, r: Rect, s: f32) {
    let working = !game.day.station(StationId::Oven).is_idle();
    let c = mid(r);
    // A brick dome on a stone base.
    pen.rrect(vec2(r.x, r.y + 150.0), vec2(r.w, 62.0), 6.0, ink::STONE);
    for k in 0..4 {
        let x = (k as f32).mul_add(38.0, r.x + 4.0);
        pen.rrect(
            vec2(x, r.y + 160.0),
            vec2(34.0, 18.0),
            3.0,
            ink::STONE_LIGHT,
        );
        pen.rrect(
            vec2(x + 14.0, r.y + 184.0),
            vec2(34.0, 18.0),
            3.0,
            ink::STONE_DARK,
        );
    }
    let floor = r.y + 150.0;
    dome(
        pen,
        vec2(c.x, floor),
        vec2(r.w * 0.5 * s, 100.0),
        ink::BRICK_DARK,
    );
    dome(
        pen,
        vec2(c.x, floor),
        vec2(r.w.mul_add(0.5, -6.0), 94.0),
        ink::BRICK,
    );
    pen.rect(vec2(r.x, floor), vec2(r.w, 2.0), ink::BRICK_DARK);
    for row in 0..4 {
        let y = (row as f32).mul_add(22.0, r.y + 66.0);
        let half = ((y - floor) / 94.0)
            .mul_add(-((y - floor) / 94.0), 1.0)
            .max(0.0)
            .sqrt()
            * r.w.mul_add(0.5, -8.0);
        pen.line(
            vec2(c.x - half, y),
            vec2(c.x + half, y),
            1.2,
            ink::BRICK_DARK,
        );
    }
    // The mouth.
    let mouth = vec2(c.x, r.y + 132.0);
    pen.rrect_centred(
        mouth + vec2(0.0, 8.0),
        vec2(84.0, 50.0),
        24.0,
        ink::BRICK_DARK,
    );
    pen.rrect_centred(
        mouth + vec2(0.0, 10.0),
        vec2(74.0, 44.0),
        22.0,
        hex(0x2a_1c18),
    );
    if working || game.day.fire() > 0.0 {
        let glow = if working {
            0.2f32.mul_add(pulse(game.clock * 2.0), 0.55)
        } else {
            0.2
        };
        pen.rrect_centred(
            mouth + vec2(0.0, 14.0),
            vec2(66.0, 34.0),
            17.0,
            alpha(ink::EMBER, glow),
        );
    }
    // The chimney.
    pen.rect(
        vec2(c.x + 30.0, r.y - 10.0),
        vec2(20.0, 40.0),
        ink::BRICK_DARK,
    );
}

fn board(game: &Game, pen: &Pen, r: Rect, s: f32) {
    let c = vec2(mid(r).x, r.y + 74.0);
    pen.rrect_centred(
        c + vec2(0.0, 4.0),
        vec2(80.0 * s, 26.0),
        6.0,
        ink::WOOD_DARK,
    );
    pen.rrect_centred(c, vec2(80.0 * s, 24.0), 6.0, ink::WOOD_LIGHT);
    pen.circle(c + vec2(32.0, 0.0), 3.0, ink::WOOD);
    // The knife comes down with each stroke.
    let since = game.bumped(Spot::Place(Place::Station(StationId::Board)));
    let chop = if since < 0.12 {
        1.0 - since / 0.12
    } else {
        0.0
    };
    let blade = c + vec2(20.0, -30.0 + chop * 16.0);
    pen.poly(
        &[
            blade,
            blade + vec2(26.0, 2.0),
            blade + vec2(26.0, 10.0),
            blade + vec2(2.0, 12.0),
        ],
        ink::STONE_LIGHT,
    );
    pen.capsule(
        blade + vec2(26.0, 6.0),
        blade + vec2(40.0, 6.0),
        3.5,
        ink::WOOD_DARK,
    );
}

fn quern(game: &Game, pen: &Pen, r: Rect, s: f32) {
    let c = vec2(mid(r).x, r.y + 72.0);
    let st = game.day.station(StationId::Quern);
    let turn = st.progress() * TAU * 2.0;
    pen.ellipse(
        c + vec2(0.0, 10.0),
        vec2(38.0 * s, 12.0),
        0.0,
        ink::STONE_DARK,
    );
    pen.rrect_centred(c + vec2(0.0, 4.0), vec2(76.0 * s, 14.0), 6.0, ink::STONE);
    pen.ellipse(
        c - vec2(0.0, 4.0),
        vec2(36.0 * s, 11.0),
        0.0,
        ink::STONE_LIGHT,
    );
    pen.ellipse(c - vec2(0.0, 4.0), vec2(8.0, 3.0), 0.0, ink::STONE_DARK);
    let peg = c - vec2(0.0, 4.0) + vec2(turn.cos() * 26.0, turn.sin() * 7.0);
    pen.capsule(peg, peg - vec2(0.0, 14.0), 3.0, ink::WOOD_DARK);
}

fn bowl(pen: &Pen, r: Rect, s: f32) {
    let c = vec2(mid(r).x, r.y + 70.0);
    pen.pie(c, 36.0 * s, 0.0, 180.0, ink::CERAMIC);
    pen.arc(c, 30.0 * s, 5.0, 10.0, 160.0, ink::GLAZE);
    pen.ellipse(c, vec2(36.0 * s, 8.0), 0.0, dark(ink::CERAMIC, 0.15));
    pen.ellipse(c, vec2(31.0 * s, 5.5), 0.0, dark(ink::CERAMIC, 0.3));
    pen.capsule(
        c + vec2(10.0, -2.0),
        c + vec2(30.0, -26.0),
        2.0,
        ink::WOOD_DARK,
    );
}

fn crock(game: &Game, pen: &Pen, r: Rect, s: f32) {
    let st = game.day.station(StationId::Crock);
    let c = vec2(mid(r).x, r.y + 70.0);
    pen.rrect_centred(c, vec2(54.0 * s, 46.0), 16.0, ink::CLAY);
    pen.rrect_centred(
        c + vec2(-10.0, -2.0),
        vec2(10.0, 28.0),
        5.0,
        light(ink::CLAY, 0.2),
    );
    pen.rect_centred(
        c + vec2(0.0, -6.0),
        vec2(54.0 * s, 4.0),
        dark(ink::CLAY, 0.25),
    );
    let shut = !st.is_idle();
    let lift = if shut {
        (game.clock * 4.0).sin().abs() * 1.2
    } else {
        -8.0
    };
    pen.ellipse(
        c - vec2(0.0, 24.0 - lift * 0.0 + if shut { 0.0 } else { 8.0 }),
        vec2(24.0, 6.0),
        if shut { 0.0 } else { -0.25 },
        dark(ink::CLAY, 0.15),
    );
    pen.circle(
        c - vec2(0.0, 30.0 + if shut { -lift } else { 8.0 }),
        4.0,
        dark(ink::CLAY, 0.3),
    );
}

/// The top half of an ellipse standing on `base`.
fn dome(pen: &Pen, base: Vec2, r: Vec2, color: Color) {
    let points: Vec<Vec2> = (0..=24)
        .map(|i| {
            let a = PI + PI * i as f32 / 24.0;
            base + vec2(a.cos() * r.x, a.sin() * r.y)
        })
        .collect();
    pen.poly(&points, color);
}

/// A small disc on the station's corner showing how it works — hand, flame
/// or hourglass — ringed by its progress.
fn badge(game: &Game, pen: &Pen, id: StationId, r: Rect) {
    let st = game.day.station(id);
    let kind = st.processes()[0].kind();
    let at = vec2(r.x + r.w - 12.0, r.y + 12.0);
    let colour = kind_colour(kind);
    let state = st.state();
    let active = !matches!(state, cooking::State::Idle);
    pen.circle(at, 13.0, alpha(ink::WHITE, if active { 0.95 } else { 0.7 }));
    icons::kind(pen, kind, at, 16.0);
    match state {
        cooking::State::Working { .. } => {
            pen.arc(at, 14.0, 4.0, -90.0, 360.0, alpha(ink::LINE, 0.15));
            pen.arc(at, 14.0, 4.0, -90.0, 360.0 * st.progress(), colour);
        }
        cooking::State::Ready { .. } => {
            let scorch = st.scorch();
            if scorch > 0.0 {
                pen.arc(at, 14.0, 4.0, -90.0, 360.0 * scorch, ink::BAD);
            } else {
                pen.ring(at, 16.0, 3.0, ink::GOOD);
            }
        }
        cooking::State::Idle => {}
    }
}

const fn kind_colour(kind: Kind) -> Color {
    match kind {
        Kind::Physical => hex(0x5a_8fd0),
        Kind::Heat => ink::FLAME,
        Kind::Time => ink::GOOD,
    }
}

/// Above an idle station with something in it: what starting it would make
/// — or, if nothing yet, what it is on the way to and what is missing —
/// and a nudge to start it.
fn hint(game: &Game, pen: &Pen, id: StationId, r: Rect) {
    let st = game.day.station(id);
    let Some((process, recipe)) = st.plan() else {
        return;
    };
    let above = vec2(mid(r).x, r.y - 4.0);
    let (out, missing, ready) = recipe.map_or_else(
        || {
            st.toward()
                .next()
                .map_or((Food::Mush, Vec::new(), true), |r| {
                    (r.output, recipe::missing(r, st.contents()), false)
                })
        },
        |recipe| (recipe.output, Vec::new(), true),
    );
    let w = 20.0f32.mul_add(missing.len() as f32, 44.0);
    pen.rrect_centred(above, vec2(w, 30.0), 12.0, alpha(ink::WHITE, 0.85));
    pen.tri(
        above + vec2(-6.0, 14.0),
        above + vec2(6.0, 14.0),
        above + vec2(0.0, 20.0),
        alpha(ink::WHITE, 0.85),
    );
    let left = above.x - w * 0.5 + 18.0;
    let fade = if ready { 1.0 } else { 0.45 };
    icons::process(pen, process, vec2(left, above.y), 18.0);
    let arrow = vec2(left + 14.0, above.y);
    pen.tri(
        arrow + vec2(-3.0, -4.0),
        arrow + vec2(3.0, 0.0),
        arrow + vec2(-3.0, 4.0),
        ink::LINE,
    );
    ghost(pen, out, vec2(left + 30.0, above.y), 24.0, fade);
    for (k, &m) in missing.iter().enumerate() {
        let at = vec2((k as f32).mul_add(20.0, left + 52.0), above.y);
        pen.circle(at, 9.0, alpha(ink::BAD, 0.2));
        ghost(pen, m, at, 16.0, 0.8);
    }
    if ready {
        // It can go: pulse the badge.
        let at = vec2(r.x + r.w - 12.0, r.y + 12.0);
        let p = pulse(game.clock * 1.5);
        pen.ring(
            at,
            17.0 + p * 3.0,
            2.0,
            alpha(kind_colour(process.kind()), 0.5f32.mul_add(p, 0.5)),
        );
        if st.drive() == Drive::Effort {
            // Hold the button to work.
            let dot = vec2(r.x + r.w - 12.0, r.y + 34.0);
            pen.circle(dot, 3.0 + p * 1.5, alpha(ink::LINE, 0.5));
        }
    }
}

/// A food drawn faintly, for things that are not there yet.
fn ghost(pen: &Pen, food: Food, at: Vec2, size: f32, fade: f32) {
    icons::food(pen, food, at, size);
    if fade < 1.0 {
        pen.circle(at, size * 0.55, alpha(ink::WHITE, 1.0 - fade));
    }
}

// The hatch.

fn hatch(game: &Game, pen: &Pen) {
    let x = layout::HATCH_X;
    // The street beyond.
    pen.rect(vec2(x, 400.0), vec2(120.0, 40.0), hex(0xb8_a888));
    for seat in 0..SEATS {
        let r = layout::seat(seat);
        if let Some(c) = game.day.seats()[seat] {
            let rise = (game.arrived[seat] / 0.35).min(1.0);
            let ease = 1.0 - (1.0 - rise).powi(3);
            villager(game, pen, &c, r, (1.0 - ease) * 60.0, c.mood());
            bubble(game, pen, &c, r, ease);
        }
        for l in game.leaving.iter().filter(|l| l.seat == seat) {
            let t = l.age / 1.6;
            let (drop, mood) = if l.happy {
                (
                    (t * PI * 3.0)
                        .sin()
                        .max(0.0)
                        .mul_add(-10.0, (t * 2.0 - 0.6).max(0.0).powi(2) * 120.0),
                    1.0,
                )
            } else {
                (t * t * 120.0, 0.0)
            };
            villager(game, pen, &l.customer, r, drop, mood);
            if !l.happy && t < 0.6 {
                let head = vec2(mid(r).x, r.y + 70.0 + drop);
                pen.ellipse(
                    head - vec2(0.0, 34.0),
                    vec2(14.0, 6.0),
                    0.0,
                    alpha(ink::SMOKE, 0.8),
                );
                pen.line(
                    head - vec2(2.0, 30.0),
                    head - vec2(6.0, 22.0),
                    1.5,
                    ink::YOLK,
                );
            }
        }
        // The sill they lean on, drawn over them.
        pen.rrect(
            vec2(x - 2.0, r.y + r.h - 12.0),
            vec2(114.0, 14.0),
            4.0,
            ink::WOOD_LIGHT,
        );
        pen.rect(vec2(x - 2.0, r.y + r.h), vec2(114.0, 4.0), ink::WOOD_DARK);
    }
    // The window frame.
    pen.rect(
        vec2(x - 6.0, layout::SKY_H),
        vec2(10.0, 380.0),
        ink::WOOD_DARK,
    );
    pen.rect(vec2(x - 6.0, 438.0), vec2(120.0, 162.0), ink::WOOD);
}

fn villager(game: &Game, pen: &Pen, c: &Customer, r: Rect, sink: f32, mood: f32) {
    let look = c.look;
    let skins = [0xf3_d2b4, 0xe0_ac85, 0xc6_8a5e, 0x8d_5a3b, 0x6b_4129];
    let shirts = [
        0x5a_8fd0, 0xd0_6a5a, 0x6c_b06a, 0xd8_a84a, 0x9a_6ac0, 0x4a_9a9a,
    ];
    let hairs = [0x3b_2a22, 0x7a_4e30, 0xe8_c070, 0xb0_4a2a, 0x8a_8580];
    let skin = hex(skins[(look % 5) as usize]);
    let shirt = hex(shirts[((look / 5) % 6) as usize]);
    let hair = hex(hairs[((look / 30) % 5) as usize]);
    let style = (look / 150) % 4;
    let breathe = game.clock.mul_add(1.4, (look % 7) as f32).sin() * 1.2;
    let base = vec2(mid(r).x, r.y + r.h - 8.0 + sink);
    // Shoulders.
    pen.rrect_centred(
        base - vec2(0.0, 10.0 - breathe * 0.3),
        vec2(70.0, 34.0),
        16.0,
        shirt,
    );
    pen.rrect_centred(base - vec2(0.0, 20.0), vec2(16.0, 10.0), 4.0, skin);
    let head = base - vec2(0.0, 46.0 - breathe);
    // Hair behind.
    if style == 1 {
        pen.circle(head - vec2(0.0, 18.0), 9.0, hair);
    }
    pen.circle(head, 22.0, skin);
    match style {
        0 => {
            // A cap.
            pen.pie(head - vec2(0.0, 6.0), 23.0, 180.0, 180.0, shirt);
            pen.capsule(
                head - vec2(4.0, 6.0),
                head - vec2(-26.0, 6.0),
                3.0,
                dark(shirt, 0.3),
            );
        }
        1 => pen.pie(head - vec2(0.0, 4.0), 22.0, 180.0, 180.0, hair),
        2 => {
            // A straw hat.
            pen.ellipse(head - vec2(0.0, 14.0), vec2(34.0, 7.0), 0.0, ink::WHEAT);
            pen.pie(head - vec2(0.0, 14.0), 17.0, 180.0, 180.0, ink::WHEAT);
            pen.rect(
                vec2(head.x - 17.0, head.y - 18.0),
                vec2(34.0, 4.0),
                ink::TOMATO,
            );
        }
        _ => {
            pen.pie(head - vec2(0.0, 6.0), 22.0, 200.0, 140.0, hair);
            pen.circle(head + vec2(0.0, 14.0), 10.0, hair);
            pen.circle(head + vec2(0.0, 6.0), 9.0, skin);
        }
    }
    // A face that sours as they wait.
    let eye = if mood < 0.25 { 1.6 } else { 2.4 };
    pen.ellipse(head + vec2(-7.0, 2.0), vec2(2.2, eye), 0.0, ink::LINE);
    pen.ellipse(head + vec2(7.0, 2.0), vec2(2.2, eye), 0.0, ink::LINE);
    pen.circle(head + vec2(-12.0, 9.0), 3.5, alpha(ink::HEART, 0.35));
    pen.circle(head + vec2(12.0, 9.0), 3.5, alpha(ink::HEART, 0.35));
    let m = head + vec2(0.0, 11.0);
    if mood > 0.5 {
        pen.arc(m - vec2(0.0, 3.0), 5.0, 1.8, 20.0, 140.0, ink::LINE);
    } else if mood > 0.25 {
        pen.line(m - vec2(4.0, 0.0), m + vec2(4.0, 0.0), 1.8, ink::LINE);
    } else {
        pen.arc(m + vec2(0.0, 5.0), 5.0, 1.8, 200.0, 140.0, ink::LINE);
    }
}

fn bubble(game: &Game, pen: &Pen, c: &Customer, r: Rect, show: f32) {
    let s = squash(game.bumped(Spot::Place(Place::Seat(
        (0..SEATS).find(|&i| layout::seat(i) == r).unwrap_or(0),
    ))));
    let at = vec2(mid(r).x - 4.0, r.y + 22.0);
    let size = vec2(80.0, 40.0) * show * s;
    if size.x < 4.0 {
        return;
    }
    let mood = c.mood();
    let ring = if mood > 0.5 {
        ink::GOOD
    } else if mood > 0.25 {
        ink::YOLK
    } else {
        ink::BAD
    };
    pen.rrect_centred(at + vec2(0.0, 2.0), size, 16.0, alpha(ink::LINE, 0.25));
    pen.rrect_centred(at, size, 16.0, ink::WHITE);
    pen.tri(
        at + vec2(4.0, 18.0),
        at + vec2(16.0, 18.0),
        at + vec2(10.0, 28.0),
        ink::WHITE,
    );
    let dish = at - vec2(20.0, 0.0);
    pen.arc(dish, 16.0, 3.5, -90.0, 360.0 * mood, ring);
    icons::food(pen, c.wants, dish, 26.0 * show);
    // The price, in coins.
    let price = prices::price(c.wants);
    for k in 0..price.min(12) {
        let (col, row) = ((k % 4) as f32, (k / 4) as f32);
        pen.coin(
            at + vec2(col.mul_add(8.0, 2.0), row.mul_add(8.0, -8.0)),
            3.6,
        );
    }
}

// The jar, and everything else on top.

fn jar(pen: &Pen, coins: u32, clock: f32) {
    let r = layout::JAR;
    let top = MARKS[2] as f32 * 1.12;
    let fill = (coins as f32 / top).min(1.0);
    let inner = Rect::new(r.x + 8.0, r.y + 22.0, r.w - 16.0, r.h - 30.0);
    pen.rrect(
        vec2(r.x, r.y + 14.0),
        vec2(r.w, r.h - 14.0),
        14.0,
        alpha(ink::WHITE, 0.35),
    );
    pen.rrect(
        vec2(r.x + 10.0, r.y),
        vec2(r.w - 20.0, 18.0),
        5.0,
        ink::WOOD_DARK,
    );
    // Coins, stacked.
    let level = inner.h.mul_add(1.0 - fill, inner.y);
    pen.rrect(
        vec2(inner.x, level),
        vec2(inner.w, inner.y + inner.h - level),
        8.0,
        ink::GOLD_DARK,
    );
    // A positive height over a small step.
    #[allow(clippy::cast_sign_loss)]
    let rows = ((inner.y + inner.h - level) / 7.0) as usize;
    for k in 0..rows {
        let y = (k as f32).mul_add(-7.0, inner.y + inner.h - 4.0);
        let wob = if k % 2 == 0 { -3.0 } else { 3.0 };
        pen.ellipse(
            vec2(mid(inner).x + wob, y),
            vec2(inner.w * 0.42, 3.4),
            0.0,
            ink::GOLD,
        );
    }
    // The three marks.
    for (i, &m) in MARKS.iter().enumerate() {
        let y = inner.h.mul_add(1.0 - m as f32 / top, inner.y);
        let reached = coins >= m;
        pen.line(vec2(r.x + 2.0, y), vec2(r.x + 14.0, y), 2.5, ink::LINE);
        let c = vec2(r.x - 8.0, y);
        if reached {
            pen.sparkle(c, pulse(clock + i as f32).mul_add(2.0, 9.0), ink::GOLD);
        } else {
            pen.sparkle(c, 7.0, alpha(ink::LINE, 0.35));
        }
    }
    pen.rrect(
        vec2(r.x + 6.0, r.y + 24.0),
        vec2(6.0, r.h - 50.0),
        3.0,
        alpha(ink::WHITE, 0.4),
    );
}

fn speaker(game: &Game, pen: &Pen) {
    let c = mid(layout::SPEAKER);
    let col = alpha(ink::WHITE, 0.85);
    pen.rect_centred(c - vec2(5.0, 0.0), vec2(6.0, 8.0), col);
    pen.tri(
        c - vec2(5.0, 0.0),
        c + vec2(3.0, -9.0),
        c + vec2(3.0, 9.0),
        col,
    );
    if game.audio.is_muted() {
        pen.line(c + vec2(6.0, -5.0), c + vec2(14.0, 5.0), 2.0, col);
        pen.line(c + vec2(6.0, 5.0), c + vec2(14.0, -5.0), 2.0, col);
    } else {
        pen.arc(c + vec2(3.0, 0.0), 7.0, 1.8, -50.0, 100.0, col);
        pen.arc(c + vec2(3.0, 0.0), 12.0, 1.8, -50.0, 100.0, col);
    }
}

/// While carrying, light up everywhere it could go.
fn targets(game: &Game, pen: &Pen) {
    let Some(carry) = game.carry else {
        return;
    };
    let mut places: Vec<Place> = Vec::new();
    places.extend((0..PANTRY).map(Place::Shelf));
    places.extend(StationId::ALL.map(Place::Station));
    places.extend((0..PLOTS).map(Place::Plot));
    places.extend((0..SEATS).map(Place::Seat));
    places.push(Place::Compost);
    let p = pulse(game.clock * 1.5);
    for place in places {
        if place == carry.from || game.day.can_move(carry.from, place).is_err() {
            continue;
        }
        let r = layout::place(place);
        let hot = r.contains(game.pointer);
        let wanted = matches!(place, Place::Seat(_));
        let c = if wanted { ink::GOOD } else { ink::GLOW };
        let a = if hot { 0.95 } else { 0.3f32.mul_add(p, 0.35) };
        pen.rrect(
            vec2(r.x - 2.0, r.y - 2.0),
            vec2(r.w + 4.0, r.h + 4.0),
            12.0,
            alpha(c, a * 0.25),
        );
        outline(pen, r, 12.0, 2.5, alpha(c, a));
    }
}

/// A rounded outline.
fn outline(pen: &Pen, r: Rect, rad: f32, thick: f32, color: Color) {
    let rad = rad.min(r.w * 0.5).min(r.h * 0.5);
    let (x0, y0, x1, y1) = (r.x, r.y, r.x + r.w, r.y + r.h);
    pen.line(vec2(x0 + rad, y0), vec2(x1 - rad, y0), thick, color);
    pen.line(vec2(x0 + rad, y1), vec2(x1 - rad, y1), thick, color);
    pen.line(vec2(x0, y0 + rad), vec2(x0, y1 - rad), thick, color);
    pen.line(vec2(x1, y0 + rad), vec2(x1, y1 - rad), thick, color);
    pen.arc(vec2(x0 + rad, y0 + rad), rad, thick, 180.0, 90.0, color);
    pen.arc(vec2(x1 - rad, y0 + rad), rad, thick, 270.0, 90.0, color);
    pen.arc(vec2(x1 - rad, y1 - rad), rad, thick, 0.0, 90.0, color);
    pen.arc(vec2(x0 + rad, y1 - rad), rad, thick, 90.0, 90.0, color);
}

fn particles(game: &Game, pen: &Pen) {
    for p in &game.fx.particles {
        let fade = 1.0 - p.spent();
        let c = alpha(p.color, p.color.a * fade);
        match p.shape {
            Shape::Mote => pen.circle(p.pos, p.size * 0.6f32.mul_add(p.spent(), 0.6), c),
            Shape::Spark => pen.sparkle(p.pos, p.size * fade.max(0.3), c),
            Shape::Heart => pen.heart(p.pos, p.size, c),
            Shape::Coin => pen.coin(p.pos, p.size),
            Shape::Chip => pen.rect_centred(p.pos, Vec2::splat(p.size), c),
        }
    }
    for f in &game.fx.flyers {
        if f.age >= 0.0 {
            icons::food(pen, f.food, f.at(), 30.0);
        }
    }
}

fn carried(game: &Game, pen: &Pen) {
    if let Some(carry) = game.carry {
        let at = game.pointer + vec2(0.0, -6.0);
        pen.shadow(game.pointer + vec2(4.0, 20.0), 30.0);
        icons::food(pen, carry.food, at, 40.0);
    }
}

/// The food under the pointer, if any, for its recipe card.
fn hovered_food(game: &Game) -> Option<Food> {
    match game.hover? {
        Spot::Place(Place::Seat(i)) => game.day.seats()[i].map(|c| c.wants),
        Spot::Place(Place::Station(id)) => {
            let st = game.day.station(id);
            st.ready()
                .or_else(|| st.plan().and_then(|(_, r)| r.map(|r| r.output)))
        }
        Spot::Place(place) => game.day.peek(place).ok().or(match place {
            Place::Market(f) => Some(f),
            _ => None,
        }),
        _ => None,
    }
}

/// How a food is made, in pictures, beside the pointer: what goes in, the
/// mark of the process, and what comes out. Raw food shows where it comes
/// from instead.
fn card(game: &Game, pen: &Pen) {
    if game.carry.is_some() || game.title || game.day.is_over() || game.hover_for < CARD_DELAY {
        return;
    }
    let Some(food) = hovered_food(game) else {
        return;
    };
    let appear = ((game.hover_for - CARD_DELAY) / 0.12).min(1.0);
    let icon = 30.0;
    let (inputs, process) = recipe::making(food).next().map_or_else(
        || (Vec::new(), None),
        |r| (r.inputs.to_vec(), Some(r.process)),
    );
    let slots = inputs.len().max(1) as f32;
    let w = slots.mul_add(icon + 6.0, 100.0);
    let h = 54.0;
    let mut at = game.pointer + vec2(18.0, 18.0);
    at.x = at.x.min(796.0 - w);
    at.y = at.y.min(596.0 - h);
    let size = vec2(w, h) * appear;
    pen.rrect(at + vec2(0.0, 3.0), size, 12.0, alpha(ink::LINE, 0.3));
    pen.rrect(at, size, 12.0, ink::CREAM);
    if appear < 1.0 {
        return;
    }
    let y = at.y + h * 0.5;
    let mut x = at.x + 6.0 + icon * 0.5;
    if let Some(process) = process {
        for (k, &input) in inputs.iter().enumerate() {
            icons::food(pen, input, vec2(x, y), icon);
            x += icon + 6.0;
            if k + 1 < inputs.len() {
                pen.rect_centred(vec2(x - icon * 0.5 - 3.0, y), vec2(6.0, 2.0), ink::LINE);
                pen.rect_centred(vec2(x - icon * 0.5 - 3.0, y), vec2(2.0, 6.0), ink::LINE);
            }
        }
        pen.circle(
            vec2(x + 2.0, y),
            15.0,
            alpha(kind_colour(process.kind()), 0.3),
        );
        icons::process(pen, process, vec2(x + 2.0, y), 22.0);
        x += 22.0;
    } else {
        source(pen, food, vec2(x, y), icon);
        x += icon;
    }
    pen.tri(
        vec2(x, y - 6.0),
        vec2(x + 10.0, y),
        vec2(x, y + 6.0),
        ink::LINE,
    );
    icons::food(pen, food, vec2(x + 32.0, y), icon + 6.0);
    if food.stage() == Stage::Dish {
        let price = prices::price(food);
        for k in 0..price.min(12) {
            pen.coin(
                vec2((k as f32).mul_add(7.0, at.x + 10.0), at.y + h + 4.0),
                3.6,
            );
        }
    }
}

/// Where a raw food comes from, in a picture.
fn source(pen: &Pen, food: Food, at: Vec2, size: f32) {
    let u = size / 30.0;
    match food {
        Food::Egg => {
            pen.circle(at, 10.0 * u, ink::WHITE);
            pen.circle(at - vec2(0.0, 9.0 * u), 4.0 * u, ink::TOMATO);
            pen.tri(
                at + vec2(8.0, -2.0) * u,
                at + vec2(14.0, 1.0) * u,
                at + vec2(8.0, 4.0) * u,
                ink::YOLK,
            );
            pen.circle(at + vec2(3.0, -1.0) * u, 1.5 * u, ink::LINE);
        }
        Food::Milk => {
            pen.ellipse(at, vec2(11.0, 12.0) * u, 0.0, ink::WHITE);
            pen.ellipse(
                at + vec2(0.0, 7.0) * u,
                vec2(9.0, 6.0) * u,
                0.0,
                hex(0xf2_a7b5),
            );
            pen.ellipse(
                at + vec2(-6.0, -4.0) * u,
                vec2(4.0, 3.0) * u,
                0.0,
                ink::LINE,
            );
        }
        Food::Water => {
            pen.ellipse(
                at + vec2(0.0, 4.0) * u,
                vec2(13.0, 7.0) * u,
                0.0,
                ink::STONE,
            );
            pen.ellipse(at, vec2(11.0, 4.0) * u, 0.0, ink::WATER_DARK);
            pen.poly(
                &[
                    at + vec2(-13.0, -6.0) * u,
                    at + vec2(0.0, -14.0) * u,
                    at + vec2(13.0, -6.0) * u,
                ],
                ink::BRICK,
            );
        }
        _ if MARKET.contains(&food) => pen.coin(at, 10.0 * u),
        _ => {
            pen.ellipse(at + vec2(0.0, 6.0) * u, vec2(13.0, 6.0) * u, 0.0, ink::SOIL);
            sprout(pen, food, at + vec2(0.0, 4.0) * u, 0.7, 0.0);
        }
    }
}

fn title(game: &Game, pen: &Pen) {
    pen.rect(
        vec2(-20.0, -20.0),
        vec2(840.0, 640.0),
        alpha(ink::LINE, 0.45),
    );
    let c = vec2(400.0, 290.0);
    pen.rrect_centred(
        c + vec2(0.0, 5.0),
        vec2(470.0, 340.0),
        24.0,
        alpha(ink::LINE, 0.4),
    );
    pen.rrect_centred(c, vec2(470.0, 340.0), 24.0, ink::CREAM);
    emblem_art(pen, c + vec2(0.0, -88.0), 1.3, game.clock);
    // The three kinds of process, each with an example: work it, watch it,
    // wait for it.
    let rows = [
        (Kind::Physical, Food::Carrot, Food::ChoppedCarrot),
        (Kind::Heat, Food::Egg, Food::FriedEgg),
        (Kind::Time, Food::Dough, Food::RisenDough),
    ];
    for (i, (kind, from, to)) in rows.into_iter().enumerate() {
        let y = (i as f32).mul_add(46.0, c.y + 10.0);
        let x = c.x - 110.0;
        pen.circle(vec2(x, y), 17.0, alpha(kind_colour(kind), 0.35));
        icons::kind(pen, kind, vec2(x, y), 22.0);
        icons::food(pen, from, vec2(x + 70.0, y), 34.0);
        pen.tri(
            vec2(x + 104.0, y - 7.0),
            vec2(x + 118.0, y),
            vec2(x + 104.0, y + 7.0),
            ink::LINE,
        );
        icons::food(pen, to, vec2(x + 150.0, y), 34.0);
        if kind == Kind::Heat {
            // ...and heat, left too long, burns.
            pen.tri(
                vec2(x + 180.0, y - 5.0),
                vec2(x + 190.0, y),
                vec2(x + 180.0, y + 5.0),
                alpha(ink::BAD, 0.7),
            );
            icons::food(pen, Food::Charcoal, vec2(x + 214.0, y), 28.0);
        }
    }
    // Click to begin.
    let m = c + vec2(0.0, 146.0);
    let p = pulse(game.clock);
    pen.rrect_centred(m, vec2(22.0, 32.0), 11.0, ink::LINE);
    pen.rrect_centred(
        m + vec2(-5.0, -8.0),
        vec2(9.0, 13.0),
        4.0,
        alpha(ink::GLOW, 0.5f32.mul_add(p, 0.5)),
    );
    pen.ring(
        m,
        p.mul_add(6.0, 24.0),
        2.0,
        alpha(ink::LINE, 0.4 * (1.0 - p)),
    );
}

fn summary(game: &Game, pen: &Pen) {
    let t = (game.dusk_for / 1.2).min(1.0);
    pen.rect(
        vec2(-20.0, -20.0),
        vec2(840.0, 640.0),
        alpha(hex(0x1b_2448), 0.55 * t),
    );
    if game.dusk_for < 0.6 {
        return;
    }
    let c = vec2(400.0, 300.0);
    let rise = ((game.dusk_for - 0.6) / 0.4).min(1.0);
    let c = c + vec2(0.0, (1.0 - rise) * 40.0);
    pen.rrect_centred(
        c + vec2(0.0, 5.0),
        vec2(500.0, 330.0),
        24.0,
        alpha(ink::LINE, 0.4),
    );
    pen.rrect_centred(c, vec2(500.0, 330.0), 24.0, ink::CREAM);
    // The marks, big.
    let marks = game.day.marks();
    for i in 0..3 {
        let at = c + vec2((i as f32 - 1.0) * 70.0, -110.0);
        let lit = i < marks && game.dusk_for > (i as f32).mul_add(0.45, 1.4);
        if lit {
            pen.sparkle(
                at,
                pulse(game.clock + i as f32).mul_add(4.0, 30.0),
                ink::GOLD,
            );
            pen.sparkle(at, 16.0, light(ink::GOLD, 0.6));
        } else {
            pen.sparkle(at, 24.0, alpha(ink::LINE, 0.2));
        }
    }
    // Every dish served today.
    let cols = 10;
    for (k, &food) in game.served.iter().enumerate().take(30) {
        let (col, row) = ((k % cols) as f32, (k / cols) as f32);
        let at = c + vec2(col.mul_add(42.0, -189.0), row.mul_add(42.0, -40.0));
        let pop = ((k as f32).mul_add(-0.05, game.dusk_for - 1.0) / 0.2).clamp(0.0, 1.0);
        icons::food(pen, food, at, 36.0 * pop);
    }
    // The coins.
    let coins = game.day.coins().min(120);
    for k in 0..coins {
        let (col, row) = ((k % 30) as f32, (k / 30) as f32);
        pen.coin(
            c + vec2(col.mul_add(14.0, -203.0), row.mul_add(14.0, 92.0)),
            5.0,
        );
    }
    // Again: click, or a key.
    if game.dusk_for > 1.5 {
        let p = pulse(game.clock);
        let at = c + vec2(200.0, 130.0);
        pen.circle(at, p.mul_add(2.0, 18.0), alpha(ink::GOOD, 0.6));
        pen.arc(at, 9.0, 3.0, -60.0, 290.0, ink::WHITE);
        pen.tri(
            at + vec2(2.0, -14.0),
            at + vec2(10.0, -9.0),
            at + vec2(2.0, -4.0),
            ink::WHITE,
        );
    }
}

/// The prototype's mark: a pot over a fire, steaming, with a carrot.
fn emblem_art(pen: &Pen, c: Vec2, k: f32, clock: f32) {
    let u = k;
    for i in 0..3 {
        let x = ((i as f32 - 1.0) * 16.0).mul_add(u, c.x);
        let flick = (i as f32).mul_add(2.0, clock * 8.0).sin() * 3.0 * u;
        flame(
            pen,
            vec2(x, 36.0f32.mul_add(u, c.y)),
            (16.0 + flick) * u,
            7.0 * u,
        );
    }
    pen.capsule(
        c + vec2(-48.0, -2.0) * u,
        c + vec2(-40.0, -2.0) * u,
        5.0 * u,
        ink::IRON,
    );
    pen.capsule(
        c + vec2(40.0, -2.0) * u,
        c + vec2(48.0, -2.0) * u,
        5.0 * u,
        ink::IRON,
    );
    pen.rrect_centred(
        c + vec2(0.0, 12.0) * u,
        vec2(84.0, 50.0) * u,
        18.0 * u,
        ink::IRON,
    );
    pen.rrect_centred(
        c + vec2(-18.0, 8.0) * u,
        vec2(14.0, 30.0) * u,
        7.0 * u,
        ink::IRON_LIGHT,
    );
    pen.ellipse(
        c - vec2(0.0, 12.0) * u,
        vec2(44.0, 10.0) * u,
        0.0,
        ink::IRON_LIGHT,
    );
    pen.ellipse(
        c - vec2(0.0, 12.0) * u,
        vec2(38.0, 7.0) * u,
        0.0,
        ink::BROTH,
    );
    for i in 0..3 {
        let t = (i as f32).mul_add(0.33, clock * 0.6).fract();
        let x = (t * TAU)
            .sin()
            .mul_add(4.0, (i as f32 - 1.0) * 18.0)
            .mul_add(u, c.x);
        pen.circle(
            vec2(x, (20.0 + t * 30.0).mul_add(-u, c.y)),
            (5.0 + t * 4.0) * u,
            alpha(ink::WHITE, 0.8 * (1.0 - t)),
        );
    }
    icons::food(pen, Food::Carrot, c + vec2(26.0, -26.0) * u, 34.0 * u);
    icons::food(pen, Food::TeaLeaf, c + vec2(-24.0, -24.0) * u, 24.0 * u);
}

/// The menu's picture of Kitchen Garden.
pub fn emblem(frame: &Frame, centre: MqVec2, width: f32, clock: f32) {
    let pen = Pen::new(frame);
    // The emblem is drawn in frame units about `centre`, which arrives in
    // pixels.
    let c = frame.frame_pos(centre);
    let k = width / 120.0;
    pen.ellipse(
        c + vec2(0.0, 40.0) * k,
        vec2(56.0, 9.0) * k,
        0.0,
        alpha(ink::LINE, 0.3),
    );
    emblem_art(&pen, c, k, clock);
}
