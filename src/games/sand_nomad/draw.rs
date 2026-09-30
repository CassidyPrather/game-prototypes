//! Drawing Sand Nomad: the map, the camp, the deck with the hold, and what
//! floats over them. Reads the game; changes nothing.

// Pixel drawing is x, y, w, h and a colour, over and over; longer names
// would only hide the arithmetic.
#![allow(clippy::many_single_char_names)]

use std::f32::consts::{PI, TAU};

use game_prototypes::sand_nomad::art;
use game_prototypes::sand_nomad::barter;
use game_prototypes::sand_nomad::grid::Placed;
use game_prototypes::sand_nomad::history;
use game_prototypes::sand_nomad::item::{FONDNESS_PER_WEIGHT, Item, ItemId, Kind};
use game_prototypes::sand_nomad::journey::{self, DAY_PX, PAR, Phase, WATCHES_PER_DAY};
use game_prototypes::sand_nomad::motive::Motive;
use game_prototypes::sand_nomad::world::{MAP_W, Site};
use macroquad::color::Color;
use macroquad::math::{Rect, Vec2, vec2};
use macroquad::texture::{DrawTextureParams, draw_texture_ex};

use super::paint::{
    self, card, disc, dotted, fade, ink, line, outline, px, rect, sprite, sprite_ex,
};
use super::{From, Game, Screen, Service, Spot, layout, services};
use crate::ui::{Frame, MqVec2};

const WHITE: Color = Color::new(1.0, 1.0, 1.0, 1.0);

/// Top left of a cell, in the hold or on the rug.
pub fn cell_origin(hold: bool, x: u8, y: u8) -> Vec2 {
    let base = if hold { layout::HOLD } else { layout::RUG };
    base + vec2(f32::from(x), f32::from(y)) * layout::CELL
}

pub fn scene(g: &Game) {
    match g.screen {
        Screen::Map => map(g),
        Screen::Camp => camp(g),
    }
    if g.fx.night > 0.0 {
        // Night falls and lifts in a moment at every dawn.
        let a = (g.fx.night * PI).sin() * 0.45;
        rect(0.0, 0.0, paint::W, layout::HUD_Y, fade(ink('r'), a));
    }
    deck(g);
    for mote in &g.fx.motes {
        let a = 1.0 - mote.age / mote.life;
        px(mote.pos.x, mote.pos.y, fade(ink(mote.ink), a));
    }
    if g.fx.refused > 0.0 {
        outline(
            0.0,
            0.0,
            paint::W,
            paint::H,
            fade(ink('i'), g.fx.refused * 2.0),
        );
    }
    carried(g);
    hover_card(g);
    if g.hand.loupe {
        sprite_ex(
            &g.art,
            art::LOUPE,
            g.hand.pos.x + 3.0,
            g.hand.pos.y + 3.0,
            WHITE,
            false,
        );
    }
    if g.journey.phase() == Phase::Home {
        ending(g);
    }
    if g.title {
        title(g);
    }
}

// The map.

fn map(g: &Game) {
    let art = &g.art;
    draw_texture_ex(&art.ground, 0.0, 0.0, WHITE, DrawTextureParams::default());
    let day = g.journey.clock();
    history_layer(g, day);

    // Where you have been, faintly.
    let wake = g.journey.wake();
    for pair in wake.windows(2) {
        let (a, b) = (pos(pair[0]), pos(pair[1]));
        dotted(a, b, 1, 3, 0.0, fade(ink('3'), 0.8));
    }

    let hovered = match g.hand.hover {
        Some(Spot::Place(s)) => Some(s),
        _ => None,
    };
    if let (Some(to), Phase::Camp) = (hovered, g.journey.phase()) {
        if to != g.journey.at() {
            route_preview(g, to);
        }
    }

    for &site in &Site::ALL {
        let p = pos(site);
        let lit = hovered == Some(site);
        if lit {
            ring(p + vec2(0.0, 4.0), 15.0, 6.0, fade(ink('7'), 0.8), g.clock);
        }
        sprite(art, art::place(site), p.x - 16.0, p.y - 24.0);
        let place = g.journey.place(site);
        if let Some(motive) = site.waystone() {
            let at = p + vec2(10.0, -26.0);
            let bob = if place.given {
                0.0
            } else {
                (g.clock.mul_add(2.0, p.x).sin() * 1.2).round()
            };
            rect(at.x - 1.0, at.y - 1.0 + bob, 10.0, 10.0, ink('0'));
            if place.given {
                rect(at.x, at.y, 8.0, 8.0, ink('F'));
                sprite(art, art::mark(motive), at.x, at.y);
            } else {
                rect(at.x, at.y + bob, 8.0, 8.0, ink('1'));
                sprite(art, art::mark(motive), at.x, at.y + bob);
            }
        }
        if site == Site::SpadeReef && history::reef_taken(day) {
            // The Harbor's flag over the reef.
            line(p.x + 8.0, p.y - 24.0, p.x + 8.0, p.y - 14.0, ink('3'));
            rect(p.x + 9.0, p.y - 24.0, 5.0, 3.0, ink('i'));
        }
    }

    ship(g);
    if let Some(site) = hovered {
        place_card(g, site);
    }
}

const fn pos(site: Site) -> Vec2 {
    let (x, y) = site.pos();
    vec2(x, y)
}

/// A dotted ellipse turning slowly: this is the one.
fn ring(c: Vec2, rx: f32, ry: f32, colour: Color, clock: f32) {
    for i in 0..28 {
        let a = clock.mul_add(0.8, i as f32 / 28.0 * TAU);
        if i % 2 == 0 {
            px(
                a.cos().mul_add(rx, c.x).round(),
                a.sin().mul_add(ry, c.y).round(),
                colour,
            );
        }
    }
}

/// The ship, where it is, bobbing, facing where it goes.
fn ship(g: &Game) {
    let (x, y) = g.journey.ship();
    let sailing = g.journey.voyage();
    let bob = if sailing.is_some() {
        (g.clock * 9.0).sin().round()
    } else {
        0.0
    };
    let facing_left = sailing.is_some_and(|v| v.to.pos().0 < v.from.pos().0);
    if let Some(v) = sailing {
        // A wake of dust behind, and the course ahead.
        let (from, to) = (pos(v.from), pos(v.to));
        let here = vec2(x, y);
        dotted(from, here, 2, 2, -g.clock * 10.0, fade(ink('6'), 0.9));
        dotted(here, to, 1, 4, 0.0, fade(ink('7'), 0.6));
    }
    rect(x - 6.0, y + 3.0, 12.0, 2.0, fade(ink('a'), 0.6));
    sprite_ex(
        &g.art,
        art::SHIP,
        x - 8.0,
        y - 11.0 + bob,
        WHITE,
        facing_left,
    );
}

/// The way to `to`: a dotted course with a pip for every day, red where
/// the water would run out.
fn route_preview(g: &Game, to: Site) {
    let from = pos(g.journey.at());
    let end = pos(to);
    dotted(from, end, 2, 2, -g.clock * 8.0, fade(ink('7'), 0.9));
    let length = from.distance(end);
    let watches = g.journey.watches_to(to);
    let days = (watches + g.journey.watch() % WATCHES_PER_DAY) / WATCHES_PER_DAY;
    let water = g.journey.sips();
    // The pips fall where each dawn will.
    let into_day = g.journey.watch() % WATCHES_PER_DAY;
    for d in 1..=days {
        let watches_to_dawn = d * WATCHES_PER_DAY - into_day;
        let t = (watches_to_dawn as f32 * DAY_PX / WATCHES_PER_DAY as f32 / length).min(1.0);
        let p = from.lerp(end, t);
        let colour = if d <= water { ink('z') } else { ink('j') };
        rect(p.x - 1.0, p.y - 1.0, 3.0, 3.0, ink('0'));
        px(p.x, p.y, colour);
    }
}

/// What a place is: its waystone's motive, who trades there and what they
/// want, and what else is there.
fn place_card(g: &Game, site: Site) {
    let p = pos(site);
    let place = g.journey.place(site);
    let trader = place.trader;
    let wants = trader.map_or(&[][..], |c| c.taste().wants);
    let services = [
        (site.well(), art::WELL),
        (site.pyre(), art::PYRE),
        (site.bench(), art::BENCH),
    ];
    let w = (services.iter().filter(|s| s.0).count() as f32).mul_add(
        18.0,
        (wants.len() as f32).mul_add(18.0, 8.0 + if trader.is_some() { 34.0 } else { 0.0 }),
    ) + if site.waystone().is_some() { 14.0 } else { 0.0 };
    let h = 42.0;
    // Beside the place, on the side away from the middle, so the course
    // to it stays in view.
    let x = if p.x < MAP_W / 2.0 {
        p.x + 20.0
    } else {
        p.x - 20.0 - w
    };
    let x = x.clamp(2.0, MAP_W - w - 2.0);
    let y = (p.y - h / 2.0 - 8.0).clamp(2.0, layout::HUD_Y - h - 2.0);
    card(x, y, w, h);
    let mut cx = x + 4.0;
    if let Some(m) = site.waystone() {
        sprite(&g.art, art::WAYSTONE, cx - 2.0, y + 5.0);
        sprite(&g.art, art::mark(m), cx + 2.0, y + 12.0);
        if place.given {
            rect(cx - 1.0, y + 36.0, 12.0, 2.0, ink('F'));
        }
        cx += 14.0;
    }
    if let Some(culture) = trader {
        sprite(&g.art, art::trader(culture), cx, y + 5.0);
        cx += 34.0;
    }
    for &kind in wants {
        let s = art::item(kind);
        // Big things shown by their first cell.
        let tex = g.art.tex(s);
        draw_texture_ex(
            tex,
            cx.floor(),
            y + 13.0,
            WHITE,
            DrawTextureParams {
                source: Some(Rect::new(0.0, 0.0, 16.0, 16.0)),
                ..DrawTextureParams::default()
            },
        );
        // A little plus: they want this.
        rect(cx + 12.0, y + 9.0, 5.0, 1.0, ink('p'));
        rect(cx + 14.0, y + 7.0, 1.0, 5.0, ink('p'));
        cx += 18.0;
    }
    for (here, s) in services {
        if here {
            let tex = g.art.tex(s);
            draw_texture_ex(
                tex,
                cx.floor(),
                y + 13.0,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(16.0, 16.0)),
                    ..DrawTextureParams::default()
                },
            );
            cx += 18.0;
        }
    }
}

/// The era going on without you.
fn history_layer(g: &Game, day: f32) {
    let art = &g.art;
    let up = history::towers_up(day);
    for (i, ((x, y), built)) in history::TOWERS.iter().enumerate() {
        if i < up {
            sprite(art, art::TOWER, x - 8.0, y - 14.0);
            // The arms signalling.
            let a = (g.clock.mul_add(1.3, i as f32).sin() * 2.0).round();
            line(x - 3.0, y - 13.0 + a, x + 3.0, y - 13.0 - a, ink('0'));
        } else if day > built - 1.5 {
            // Scaffold going up.
            dotted(vec2(*x, *y), vec2(*x, y - 8.0), 1, 1, 0.0, ink('3'));
        }
    }
    if let Some(ships) = history::convoy(day) {
        let heading_out = day < journey_raid() + 3.0;
        for (x, y) in ships {
            let bob = g.clock.mul_add(8.0, x).sin().round();
            sprite_ex(
                art,
                art::FRIGATE,
                x - 8.0,
                y - 11.0 + bob,
                WHITE,
                !heading_out,
            );
        }
    }
    let smoke = history::reef_smoke(day);
    if smoke > 0.0 {
        let base = pos(Site::SpadeReef) + vec2(-2.0, -18.0);
        for i in 0..14 {
            let t = g.clock.mul_add(0.4, i as f32 / 14.0).fract();
            let p = base + vec2((t * 9.0 + i as f32).sin().mul_add(3.0, t * 10.0), -t * 40.0);
            disc(
                p.x,
                p.y,
                1.0 + t * 3.0,
                fade(ink('m'), smoke * (1.0 - t) * 0.8),
            );
        }
    }
    if let Some(t) = history::fireworks(day) {
        let fort = pos(Site::Fort);
        for i in 0..4 {
            let f = (i as f32).mul_add(0.37, t * 3.0).fract();
            let c = fort
                + vec2(
                    (i as f32).mul_add(8.0, -12.0),
                    ((i % 2) as f32).mul_add(-8.0, -30.0),
                );
            for k in 0..8 {
                let a = k as f32 / 8.0 * TAU;
                let p = c + vec2(a.cos(), a.sin()) * f * 7.0;
                px(p.x, p.y, fade(ink(['F', 'j', 'y', 'z'][i]), 1.0 - f));
            }
        }
    }
    if let Some(t) = history::comet(day) {
        let head = vec2(
            MAP_W * 0.8f32.mul_add(t, 0.1),
            6.0f32.mul_add((t * PI).sin(), 10.0),
        );
        for k in 0..16 {
            px(
                (k as f32).mul_add(-1.5, head.x),
                (k as f32).mul_add(-0.4, head.y),
                fade(ink('7'), 1.0 - k as f32 / 16.0),
            );
        }
        px(head.x, head.y, ink('w'));
    }
    if let Some((x, y)) = history::dirigible(day) {
        let bob = (g.clock * 1.5).sin().round();
        rect(x - 10.0, y + 18.0, 20.0, 2.0, fade(ink('a'), 0.35));
        sprite(art, art::DIRIGIBLE, x - 16.0, y - 8.0 + bob);
    }
}

const fn journey_raid() -> f32 {
    game_prototypes::sand_nomad::world::RAID_DAY as f32
}

// The camp.

/// The sky over camp, by the watch: morning, noon, afternoon, evening.
const fn sky(watch: u32) -> [char; 3] {
    match watch % WATCHES_PER_DAY {
        0 => ['s', '6', '7'],
        1 => ['G', 'd', '7'],
        2 => ['8', '6', '7'],
        _ => ['v', 'E', 's'],
    }
}

fn camp(g: &Game) {
    let art = &g.art;
    let here = g.journey.at();
    let horizon = 100.0;
    let [high, mid, low] = sky(g.journey.watch());
    // Banded, dithered sky.
    for y in 0..horizon as i32 {
        let t = y as f32 / horizon;
        let c = if t < 0.45 {
            high
        } else if t < 0.8 {
            mid
        } else {
            low
        };
        rect(0.0, y as f32, paint::W, 1.0, ink(c));
        // Dither the band edges.
        if (t - 0.45).abs() < 0.03 || (t - 0.8).abs() < 0.03 {
            for x in (i32::from(y % 2 == 0)..400).step_by(2) {
                px(x as f32, y as f32, ink(if t < 0.6 { high } else { mid }));
            }
        }
    }
    // Distant dunes.
    for x in 0..400 {
        let xf = x as f32;
        let h = (xf * 0.021)
            .sin()
            .mul_add(5.0, xf.mul_add(0.053, 1.0).sin() * 3.0)
            + 8.0;
        rect(xf, horizon - h, 1.0, h, ink('9'));
    }
    sprite(art, art::place(here), 250.0, horizon - 36.0);
    draw_texture_ex(&art.camp, 0.0, horizon, WHITE, DrawTextureParams::default());

    // The waystone.
    if let Some(m) = here.waystone() {
        let r = layout::WAYSTONE;
        let given = g.journey.place(here).given;
        if g.fx.stone_glow > 0.0 || matches!(g.hand.hover, Some(Spot::Waystone)) {
            let a = if g.fx.stone_glow > 0.0 {
                g.fx.stone_glow / 2.0
            } else {
                0.4
            };
            for i in 0..10 {
                let ang = (i as f32 / 10.0).mul_add(TAU, g.clock);
                let c = r.center()
                    + vec2(ang.cos(), ang.sin())
                        * g.clock.mul_add(5.0, i as f32).sin().mul_add(2.0, 14.0);
                px(c.x, c.y, fade(ink('F'), a));
            }
        }
        sprite(art, art::WAYSTONE, r.x, r.y);
        if given {
            rect(r.x + 4.0, r.y + 7.0, 8.0, 8.0, ink('F'));
        }
        sprite(art, art::mark(m), r.x + 4.0, r.y + 7.0);
        if !given && carrying_fits_stone(g) {
            ring(r.center(), 12.0, 18.0, ink('F'), g.clock * 3.0);
        }
    }

    // The well, pyre or bench.
    for (service, s) in services(here) {
        let hovered = g.hand.hover == Some(Spot::Service(service));
        match service {
            Service::Well => sprite(art, art::WELL, s.x, s.y),
            Service::Pyre => {
                sprite(art, art::PYRE, s.x, s.y);
                fire(
                    s.x + 16.0,
                    s.y + 18.0,
                    1.0 + g.fx.flare + if hovered { 0.3 } else { 0.0 },
                    g.clock,
                );
            }
            Service::Bench => sprite(art, art::BENCH, s.x, s.y),
        }
        if hovered {
            ring(s.center() + vec2(0.0, 8.0), 17.0, 7.0, ink('7'), g.clock);
        }
    }

    if let Some(culture) = g.journey.trader() {
        rug(g);
        let t = layout::TRADER;
        let bob = ((g.clock * 1.7).sin() * 0.6).round();
        sprite(art, art::trader(culture), t.x, t.y + bob);
        scale(g);
        buttons(g);
        match g.fx.face {
            Some((emote, age)) => {
                let rise = (age * 20.0).min(4.0);
                sprite(art, art::emote(emote), t.x + 22.0, t.y - 14.0 - rise);
            }
            None => wants_bubble(g, culture),
        }
    }
}

/// Whether the thing on the pointer would satisfy the waystone here.
fn carrying_fits_stone(g: &Game) -> bool {
    let Some(want) = g.journey.waystone_here() else {
        return false;
    };
    let fits = |item: &Item| item.motive == Some(want) && !item.is_anima();
    match g.hand.carried {
        Some(c) if c.from == From::Hold => {
            g.journey.hold().get(c.id).is_some_and(|p| fits(&p.item))
        }
        _ => false,
    }
}

/// A flickering fire, `size` times its usual height, its base at `(x, y)`.
fn fire(x: f32, y: f32, size: f32, clock: f32) {
    for i in 0..18 {
        let seed = i as f32 * 1.7;
        let t = clock.mul_add(1.8, seed * 0.13).fract();
        let sway = clock.mul_add(6.0, seed).sin() * 2.0 * (1.0 - t);
        let p = vec2(
            (seed.sin() * 5.0).mul_add(1.0 - t, x) + sway,
            (t * 16.0).mul_add(-size, y),
        );
        let c = if t < 0.3 {
            'F'
        } else if t < 0.7 {
            'x'
        } else {
            'R'
        };
        let r = (1.0 - t) * 2.5;
        disc(p.x, p.y, r, ink(c));
    }
}

/// The rug the trader's things lie on, and the things.
fn rug(g: &Game) {
    let grid = g.journey.rug();
    let r = layout::rug_rect(grid.width(), grid.height());
    rect(r.x - 3.0, r.y - 3.0, r.w + 6.0, r.h + 6.0, ink('h'));
    rect(r.x - 2.0, r.y - 2.0, r.w + 4.0, r.h + 4.0, ink('i'));
    for y in 0..r.h as i32 {
        for x in (0..r.w as i32).step_by(4) {
            if (x / 4 + y / 4) % 2 == 0 && (y % 8 == 2) {
                px(r.x + x as f32, r.y + y as f32, ink('y'));
            }
        }
    }
    for i in 0..(r.w as i32 / 3) {
        px(
            (i as f32).mul_add(3.0, r.x - 3.0),
            r.y + r.h + 3.0,
            ink('k'),
        );
        px((i as f32).mul_add(3.0, r.x - 3.0), r.y - 4.0, ink('k'));
    }
    for placed in grid.placed() {
        let o = cell_origin(false, placed.x, placed.y);
        if placed.on_pan || is_carried(g, placed.item.id) {
            ghost(o, placed);
        } else {
            item(g, placed, o);
        }
    }
}

/// Something that has been lifted from its cell: where it goes back.
fn ghost(o: Vec2, placed: &Placed) {
    let (w, h) = placed.size();
    let (w, h) = (f32::from(w) * 16.0, f32::from(h) * 16.0);
    for i in (0..w as i32).step_by(2) {
        px(o.x + i as f32, o.y, fade(ink('7'), 0.4));
        px(o.x + i as f32, o.y + h - 1.0, fade(ink('7'), 0.4));
    }
    for i in (0..h as i32).step_by(2) {
        px(o.x, o.y + i as f32, fade(ink('7'), 0.4));
        px(o.x + w - 1.0, o.y + i as f32, fade(ink('7'), 0.4));
    }
}

fn is_carried(g: &Game, id: ItemId) -> bool {
    g.hand.carried.is_some_and(|c| c.id == id)
}

/// The brass scale: post, beam, chains, pans, and what is on them.
fn scale(g: &Game) {
    let pivot = layout::PIVOT;
    let base_y = 196.0;
    rect(pivot.x - 1.0, pivot.y, 3.0, base_y - pivot.y, ink('e'));
    rect(pivot.x - 1.0, pivot.y, 1.0, base_y - pivot.y, ink('g'));
    rect(pivot.x - 12.0, base_y, 25.0, 4.0, ink('e'));
    rect(pivot.x - 10.0, base_y, 21.0, 1.0, ink('g'));
    let (left, right) = g.beam_ends();
    line(left.x, left.y, right.x, right.y, ink('f'));
    line(left.x, left.y - 1.0, right.x, right.y - 1.0, ink('g'));
    disc(pivot.x, pivot.y, 2.5, ink('e'));
    px(pivot.x, pivot.y - 3.0, ink('g'));
    // A little pointer above the pivot shows which way it leans.
    let (s, c) = g.fx.beam.sin_cos();
    let tip = pivot + vec2(s, -c) * 10.0;
    line(pivot.x, pivot.y, tip.x, tip.y, ink('e'));
    for (mine, end) in [(true, left), (false, right)] {
        let plate = g.pan_rect(mine);
        let py = plate.y + plate.h - 4.0;
        dotted(end, vec2(plate.x + 4.0, py), 1, 1, 0.0, ink('e'));
        dotted(end, vec2(plate.x + plate.w - 4.0, py), 1, 1, 0.0, ink('e'));
        let hovered = matches!(g.hand.hover, Some(Spot::Pan(m) | Spot::Panned(_, m)) if m == mine);
        for (id, r) in g.pan_layout(mine) {
            if let Some(p) = g.journey.find(id) {
                if !is_carried(g, id) {
                    item_at(g, p.item, false, vec2(r.x, r.y));
                }
            }
        }
        rect(plate.x, py, plate.w, 3.0, ink('f'));
        rect(plate.x + 2.0, py + 3.0, plate.w - 4.0, 1.0, ink('e'));
        rect(
            plate.x,
            py,
            plate.w,
            1.0,
            ink(if hovered { 'w' } else { 'g' }),
        );
    }
}

fn buttons(g: &Game) {
    let art = &g.art;
    let ready = g.journey.deal_ready().is_ok();
    let h = layout::HANDS;
    rect(
        h.x - 2.0,
        h.y - 2.0,
        h.w + 4.0,
        h.h + 4.0,
        ink(if ready { 'F' } else { '2' }),
    );
    rect(h.x - 1.0, h.y - 1.0, h.w + 2.0, h.h + 2.0, ink('1'));
    let tint = if ready { WHITE } else { fade(ink('n'), 0.6) };
    let bounce = if ready {
        ((g.clock * 5.0).sin() * 1.0).round()
    } else {
        0.0
    };
    sprite_ex(art, art::HANDS, h.x, h.y + bounce, tint, false);
    let l = layout::LOUPE;
    rect(
        l.x - 2.0,
        l.y - 2.0,
        l.w + 4.0,
        l.h + 4.0,
        ink(if g.hand.loupe { 'F' } else { '2' }),
    );
    rect(l.x - 1.0, l.y - 1.0, l.w + 2.0, l.h + 2.0, ink('1'));
    sprite(art, art::LOUPE, l.x, l.y);
}

/// What the trader wants, in a thought bubble.
fn wants_bubble(g: &Game, culture: barter::Culture) {
    let wants = culture.taste().wants;
    if wants.is_empty() {
        return;
    }
    let t = layout::TRADER;
    let w = (wants.len() as f32).mul_add(17.0, 5.0);
    let x = t.x + 16.0 - w / 2.0;
    let y = t.y - 24.0;
    rect(x, y, w, 20.0, fade(ink('7'), 0.9));
    outline(x, y, w, 20.0, ink('l'));
    px(t.x + 15.0, t.y - 3.0, ink('7'));
    px(t.x + 14.0, t.y - 1.0, ink('7'));
    for (i, &kind) in wants.iter().enumerate() {
        let tex = g.art.tex(art::item(kind));
        draw_texture_ex(
            tex,
            (i as f32).mul_add(17.0, x + 3.0),
            y + 2.0,
            WHITE,
            DrawTextureParams {
                source: Some(Rect::new(0.0, 0.0, 16.0, 16.0)),
                ..DrawTextureParams::default()
            },
        );
    }
}

// The deck and the hold.

fn deck(g: &Game) {
    let art = &g.art;
    draw_texture_ex(
        &art.deck,
        0.0,
        layout::HUD_Y,
        WHITE,
        DrawTextureParams::default(),
    );
    rect(0.0, layout::HUD_Y, paint::W, 1.0, ink('0'));
    rect(0.0, layout::HUD_Y + 1.0, paint::W, 1.0, ink('5'));
    hold(g);
    jar(g);
    stones(g);
    tally(g);
    // Leave a weightless thing in the sand here.
    let p = layout::PIT;
    let open = matches!(g.hand.hover, Some(Spot::Pit)) && g.hand.carried.is_some();
    // A hollow scooped in a heap of sand.
    let c = p.center() + vec2(0.0, 3.0);
    disc(c.x, c.y, 8.0, ink('0'));
    disc(c.x, c.y, 7.0, ink(if open { '6' } else { '8' }));
    disc(c.x, c.y + 1.0, 4.0, ink('9'));
    disc(c.x, c.y + 2.0, 2.5, ink(if open { '0' } else { 'a' }));
    if g.screen == Screen::Camp {
        let s = layout::SAIL;
        let hover = matches!(g.hand.hover, Some(Spot::Sail));
        rect(s.x, s.y, s.w, s.h, ink(if hover { '5' } else { '2' }));
        outline(s.x, s.y, s.w, s.h, ink('0'));
        sprite(art, art::SAIL, s.x + 2.0, s.y + 4.0);
    }
    let sp = layout::SPEAKER;
    sprite(art, art::SPEAKER, sp.x, sp.y);
    if g.audio.is_muted() {
        line(sp.x + 1.0, sp.y + 14.0, sp.x + 14.0, sp.y + 1.0, ink('i'));
    }
}

fn hold(g: &Game) {
    let grid = g.journey.hold();
    let r = layout::hold_rect(grid.width(), grid.height());
    rect(r.x - 2.0, r.y - 2.0, r.w + 4.0, r.h + 4.0, ink('0'));
    for y in 0..grid.height() {
        for x in 0..grid.width() {
            let o = cell_origin(true, x, y);
            rect(o.x, o.y, 16.0, 16.0, ink('1'));
            outline(o.x, o.y, 16.0, 16.0, ink('2'));
        }
    }
    // Links between the hovered thing and its neighbours: like feeds like,
    // opposites quarrel.
    let focus = match g.hand.hover {
        Some(Spot::Held(id)) if g.hand.carried.is_none() => Some(id),
        _ => None,
    };
    let hint = g.journey.waystone_here().filter(|_| g.fx.hint > 0.0);
    for placed in grid.placed() {
        let o = cell_origin(true, placed.x, placed.y);
        if placed.on_pan || is_carried(g, placed.item.id) {
            ghost(o, placed);
            continue;
        }
        if hint.is_some_and(|m| placed.item.motive == Some(m)) {
            let (w, h) = placed.size();
            outline(
                o.x - 1.0,
                o.y - 1.0,
                f32::from(w).mul_add(16.0, 2.0),
                f32::from(h).mul_add(16.0, 2.0),
                fade(ink('F'), g.fx.hint),
            );
        }
        item(g, placed, o);
    }
    if let Some(id) = focus {
        links(g, id);
    }
    drop_preview(g);
}

/// Neighbour links for `id`.
fn links(g: &Game, id: ItemId) {
    let grid = g.journey.hold();
    let Some(me) = grid.get(id) else { return };
    let Some(motive) = me.item.motive else { return };
    let centre = |p: &Placed| {
        let (w, h) = p.size();
        cell_origin(true, p.x, p.y) + vec2(f32::from(w) * 8.0, f32::from(h) * 8.0)
    };
    let a = centre(me);
    for n in grid.neighbours(id) {
        let b = centre(n);
        match n.item.motive {
            Some(m) if m == motive => {
                dotted(a, b, 2, 1, -g.clock * 12.0, ink(motive_colour(m)));
            }
            Some(m) if m == motive.opposite() => {
                // A jagged spark between quarrelling things.
                let mid = a.lerp(b, 0.5);
                let across = (b - a).perp().normalize_or_zero() * 3.0;
                let jig = if (g.clock * 12.0).sin() > 0.0 {
                    1.0
                } else {
                    -1.0
                };
                line(
                    a.x,
                    a.y,
                    across.x.mul_add(jig, mid.x),
                    across.y.mul_add(jig, mid.y),
                    ink('w'),
                );
                line(
                    across.x.mul_add(jig, mid.x),
                    across.y.mul_add(jig, mid.y),
                    b.x,
                    b.y,
                    ink('w'),
                );
            }
            _ => {}
        }
    }
}

/// Where a carried thing from the hold would land, green if it fits.
fn drop_preview(g: &Game) {
    let Some(c) = g.hand.carried else { return };
    if c.from != From::Hold {
        return;
    }
    let grid = g.journey.hold();
    if !layout::hold_rect(grid.width(), grid.height()).contains(g.hand.pos) {
        return;
    }
    let Some(placed) = grid.get(c.id) else { return };
    let corner = g.hand.pos - c.grab + vec2(8.0, 8.0);
    let cell = ((corner - layout::HOLD) / layout::CELL).floor();
    #[allow(clippy::cast_possible_truncation)]
    let fits = grid.fits(
        &placed.item,
        cell.x as i32,
        cell.y as i32,
        c.turned,
        Some(c.id),
    );
    let (w, h) = placed.item.footprint(c.turned);
    let o = layout::HOLD + cell * layout::CELL;
    let colour = if fits { ink('q') } else { ink('j') };
    outline(o.x, o.y, f32::from(w) * 16.0, f32::from(h) * 16.0, colour);
    outline(
        o.x + 1.0,
        o.y + 1.0,
        f32::from(w).mul_add(16.0, -2.0),
        f32::from(h).mul_add(16.0, -2.0),
        fade(colour, 0.5),
    );
}

/// A placed thing, in its cells.
fn item(g: &Game, placed: &Placed, o: Vec2) {
    item_at(g, placed.item, placed.turned, o);
}

/// A thing with its top left at `o`: sprite, painted mark, rune, water.
fn item_at(g: &Game, it: Item, turned: bool, o: Vec2) {
    let id = it.id;
    let hop =
        g.fx.hops
            .get(&id)
            .map_or(0.0, |t| -((t / 0.25) * PI).sin() * 3.0)
            .round();
    let shake =
        g.fx.shakes
            .get(&id)
            .map_or(0.0, |t| ((t * 60.0).sin() * 2.0).round());
    let alive = if it.is_anima() && it.known {
        (g.clock.mul_add(11.0, f32::from(id.0)).sin() * 0.7).round()
    } else {
        0.0
    };
    let at = o + vec2(shake + alive, hop);
    let s = art::item(it.kind);
    let tint = if g.hand.loupe && !it.known && it.motive.is_some() {
        Color::new(1.0, 1.0, 0.2f32.mul_add((g.clock * 6.0).sin(), 0.8), 1.0)
    } else {
        WHITE
    };
    if turned {
        paint::sprite_turned(&g.art, s, at.x, at.y, tint);
    } else {
        sprite_ex(&g.art, s, at.x, at.y, tint, false);
    }
    let (w, h) = it.footprint(turned);
    let (w, h) = (f32::from(w) * 16.0, f32::from(h) * 16.0);
    if it.is_anima() && it.known {
        // Its eye has opened.
        let eye = at + vec2(w / 2.0, h / 2.0);
        disc(eye.x, eye.y, 2.5, ink('w'));
        px(eye.x + (g.clock * 2.0).sin().round(), eye.y, ink('0'));
    }
    if let Some(m) = it.motive {
        sprite(&g.art, art::mark(m), at.x, at.y + h - 8.0);
    }
    if it.motive.is_some() {
        let flash = g.fx.flashes.get(&id).copied().unwrap_or(0.0);
        paint::rune(at.x + w - 7.0, at.y, it, flash);
    }
    if it.kind == Kind::Water {
        paint::sips(at.x, at.y + h - 2.0, it);
    }
    if let Some((reading, t)) = g.fx.reading {
        if reading == id {
            // The loupe's glint sweeping across.
            let x = (t * 1.2).min(1.0).mul_add(w, at.x);
            line(
                x,
                at.y,
                x - 4.0,
                at.y + h - 1.0,
                fade(ink('w'), 1.0 - (t / 1.8)),
            );
        }
    }
}

/// The burden jar: fills with grains as the circuit weighs on you; three
/// notches mark the pars, and the pebbles beside it are the load now.
fn jar(g: &Game) {
    let r = layout::JAR;
    let top = r.y + 4.0;
    let bottom = r.y + r.h - 2.0;
    let full = PAR[2] as f32 * 1.25;
    let level = (g.journey.burden() as f32 / full).min(1.0);
    // Body.
    rect(r.x + 3.0, r.y, r.w - 6.0, 3.0, ink('0'));
    rect(r.x + 4.0, r.y + 1.0, r.w - 8.0, 1.0, ink('9'));
    rect(r.x, top, r.w, bottom - top + 2.0, ink('0'));
    rect(r.x + 1.0, top + 1.0, r.w - 2.0, bottom - top, ink('1'));
    let fill_h = ((bottom - top) * level).round();
    for y in 0..fill_h as i32 {
        let yy = bottom - y as f32;
        for x in 1..(r.w as i32 - 1) {
            let gr = paint::grain(x, y);
            let c = if gr < 0.3 {
                '2'
            } else if gr > 0.85 {
                'a'
            } else {
                '3'
            };
            px(r.x + x as f32, yy, ink(c));
        }
    }
    for (i, &par) in PAR.iter().enumerate() {
        let y = (bottom - top).mul_add(-(par as f32 / full), bottom);
        let c = ['g', 'd', 'a'][i];
        rect(r.x - 3.0, y.round(), 4.0, 1.0, ink(c));
        rect(r.x + r.w - 1.0, y.round(), 3.0, 1.0, ink(c));
    }
    // The load, a pebble a point.
    let load = g.journey.load().min(20);
    for i in 0..load {
        let y = (i as f32).mul_add(-3.0, bottom - 1.0);
        rect(138.0, y, 3.0, 2.0, ink(if i >= 12 { 'j' } else { 'n' }));
    }
}

/// The circuit: a small standing stone per waystone, lit once given.
fn stones(g: &Game) {
    let mut x = layout::STONES.x;
    let y = layout::STONES.y;
    for place in g.journey.places() {
        let Some(m) = place.site.waystone() else {
            continue;
        };
        let here = g.journey.at() == place.site && g.journey.phase() == Phase::Camp;
        rect(x, y + 2.0, 12.0, 16.0, ink('0'));
        rect(x + 1.0, y + 1.0, 10.0, 16.0, ink('0'));
        rect(
            x + 1.0,
            y + 3.0,
            10.0,
            13.0,
            ink(if place.given { 'n' } else { 'm' }),
        );
        rect(
            x + 2.0,
            y + 2.0,
            8.0,
            1.0,
            ink(if place.given { 'n' } else { 'm' }),
        );
        if place.given {
            rect(x + 2.0, y + 5.0, 8.0, 8.0, ink('F'));
            sprite(&g.art, art::mark(m), x + 2.0, y + 5.0);
        } else {
            sprite_ex(
                &g.art,
                art::mark(m),
                x + 2.0,
                y + 5.0,
                fade(WHITE, 0.45),
                false,
            );
        }
        if here {
            rect(x + 3.0, y + 19.0, 6.0, 1.0, ink('7'));
        }
        x += layout::STONE_STEP;
    }
}

/// Days gone, as tally marks in fives.
fn tally(g: &Game) {
    let days = g.journey.day();
    let o = layout::TALLY;
    let per_row = 8;
    for d in 0..days {
        let group = d / 5;
        let within = d % 5;
        let gx = (group % per_row) as f32 * 14.0;
        let gy = (group / per_row) as f32 * 9.0;
        let at = o + vec2(gx, gy);
        if within < 4 {
            rect((within as f32).mul_add(2.0, at.x), at.y, 1.0, 7.0, ink('7'));
        } else {
            line(at.x - 1.0, at.y + 6.0, at.x + 7.0, at.y + 1.0, ink('7'));
        }
    }
    // The watch, as a sun or moon climbing and setting over the tally.
    let watch = g.journey.watch() % WATCHES_PER_DAY;
    let a = (watch as f32 + 0.5) / WATCHES_PER_DAY as f32 * PI;
    let c = vec2(360.0, 262.0) + vec2(-a.cos() * 12.0, -a.sin() * 10.0);
    rect(344.0, 262.0, 32.0, 1.0, ink('2'));
    disc(c.x, c.y, 2.5, ink(if watch == 3 { 'd' } else { 'y' }));
}

// Floating things.

fn carried(g: &Game) {
    let Some(c) = g.hand.carried else { return };
    let Some(p) = g.journey.find(c.id) else {
        return;
    };
    let at = g.hand.pos - c.grab;
    let (w, h) = p.item.footprint(c.turned);
    rect(
        at.x + 3.0,
        at.y + 4.0,
        f32::from(w).mul_add(16.0, -2.0),
        f32::from(h).mul_add(16.0, -2.0),
        fade(ink('0'), 0.35),
    );
    item_at(g, p.item, c.turned, at - vec2(1.0, 2.0));
}

/// Details of what the pointer rests on: its rune writ large, how fond of
/// it you are growing, how fast.
fn hover_card(g: &Game) {
    if g.hand.carried.is_some() || g.hand.rest < 0.35 {
        return;
    }
    let (id, in_hold) = match g.hand.hover {
        Some(Spot::Held(id)) => (id, true),
        Some(Spot::Rugged(id) | Spot::Panned(id, _)) => (id, false),
        _ => return,
    };
    let Some(p) = g.journey.find(id) else { return };
    let it = p.item;
    let Some(motive) = it.motive else { return };
    let (w, h) = (40.0, 44.0);
    let x = (g.hand.pos.x + 8.0).min(paint::W - w - 2.0);
    let y = if g.hand.pos.y > 150.0 {
        g.hand.pos.y - h - 6.0
    } else {
        g.hand.pos.y + 10.0
    };
    card(x, y, w, h);
    paint::big_rune(vec2(x + 20.0, y + 17.0), it, g.clock);
    let _ = motive;
    // Fondness toward the next point, in quarters, and how much fonder a
    // day where it sits now would make you (in the hold only).
    if it.known && !it.is_anima() {
        let colour = paint::node_ink(Motive::ALL[(motive.index() + usize::from(it.weight)) % 6]);
        for i in 0..FONDNESS_PER_WEIGHT {
            let fx = f32::from(i).mul_add(4.0, x + 3.0);
            let c = if i < it.fondness { colour } else { ink('l') };
            rect(fx, y + h - 7.0, 3.0, 3.0, c);
        }
        if in_hold {
            let rate = g.journey.hold().fondness_for(id);
            for i in 0..rate.min(4) {
                let fx = f32::from(i).mul_add(4.0, x + 24.0);
                let fy = y + h - 8.0;
                // An arrow up per point of fondness a day.
                px(fx + 1.0, fy, ink('i'));
                rect(fx, fy + 1.0, 3.0, 1.0, ink('i'));
                rect(fx + 1.0, fy + 2.0, 1.0, 3.0, ink('i'));
            }
            if rate == 0 {
                rect(x + 24.0, y + h - 6.0, 10.0, 1.0, ink('p'));
            }
        }
    }
}

fn motive_colour(m: Motive) -> char {
    if m == Motive::Hate {
        'n'
    } else {
        art::motive_ink(m)
    }
}

fn title(g: &Game) {
    rect(0.0, 0.0, paint::W, paint::H, fade(ink('r'), 0.82));
    let c = vec2(200.0, 120.0);
    // The six marks around the hexagon, turning slowly, the ship inside.
    for (i, &m) in Motive::ALL.iter().enumerate() {
        let a = g
            .clock
            .mul_add(0.1, (i as f32 / 6.0).mul_add(TAU, -PI * 2.0 / 3.0));
        let p = c + vec2(a.cos(), a.sin()) * 44.0;
        rect(p.x - 5.0, p.y - 5.0, 10.0, 10.0, ink('0'));
        sprite(&g.art, art::mark(m), p.x - 4.0, p.y - 4.0);
        let q = c + vec2((a + TAU / 6.0).cos(), (a + TAU / 6.0).sin()) * 44.0;
        dotted(p, q, 1, 2, 0.0, fade(ink('7'), 0.5));
    }
    let bob = (g.clock * 3.0).sin().round();
    rect(c.x - 20.0, c.y + 14.0, 40.0, 2.0, ink('9'));
    for x in 0..60 {
        let xf = x as f32;
        let h = (xf.mul_add(0.2, g.clock).sin() * 1.5).round();
        px(c.x - 30.0 + xf, c.y + 12.0 + h, ink('8'));
    }
    paint::sprite_ex(&g.art, art::SHIP, c.x - 8.0, c.y - 4.0 + bob, WHITE, false);
    // Click to set out.
    let m = vec2(200.0, 196.0);
    let pulse = (g.clock * 3.0).sin().mul_add(0.5, 0.5);
    rect(m.x - 6.0, m.y - 9.0, 13.0, 19.0, ink('7'));
    rect(
        m.x - 5.0,
        m.y - 8.0,
        5.0,
        7.0,
        fade(ink('F'), 0.4 + pulse * 0.6),
    );
    line(m.x, m.y - 9.0, m.x, m.y - 1.0, ink('0'));
    line(m.x - 6.0, m.y - 1.0, m.x + 6.0, m.y - 1.0, ink('0'));
    outline(m.x - 6.0, m.y - 9.0, 13.0, 19.0, ink('0'));
}

fn ending(g: &Game) {
    let t = g.ended;
    rect(
        0.0,
        0.0,
        paint::W,
        layout::HUD_Y,
        fade(ink('r'), (t * 0.5).min(0.55)),
    );
    // The whole circuit, traced in gold.
    let wake = g.journey.wake();
    // A small, positive count.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let drawn = ((t * 3.0) as usize + 1).min(wake.len().saturating_sub(1));
    for pair in wake.windows(2).take(drawn) {
        let (a, b) = (pos(pair[0]), pos(pair[1]));
        line(a.x, a.y, b.x, b.y, ink('F'));
    }
    for &site in &Site::ALL {
        let p = pos(site);
        rect(p.x - 1.0, p.y - 1.0, 3.0, 3.0, ink('7'));
    }
    if t > 1.5 {
        // Three stones for the marks earned.
        let earned = journey::marks(g.journey.burden());
        let c = vec2(200.0, 60.0);
        for i in 0..3 {
            let x = (i as f32).mul_add(22.0, c.x - 30.0);
            let lit = i < earned;
            let pop = if lit {
                (i as f32).mul_add(-0.3, t - 1.5).clamp(0.0, 0.3) / 0.3
            } else {
                1.0
            };
            let s = if lit { ['g', 'd', 'a'][i] } else { '1' };
            rect(x, 18.0f32.mul_add(1.0 - pop, c.y), 16.0, 22.0, ink('0'));
            rect(
                x + 1.0,
                18.0f32.mul_add(1.0 - pop, c.y + 1.0),
                14.0,
                20.0,
                ink(s),
            );
        }
        // Start again.
        let k = vec2(200.0, 110.0);
        rect(k.x - 7.0, k.y - 7.0, 14.0, 14.0, ink('7'));
        outline(k.x - 7.0, k.y - 7.0, 14.0, 14.0, ink('0'));
        // An R, drawn.
        line(k.x - 3.0, k.y - 4.0, k.x - 3.0, k.y + 4.0, ink('0'));
        line(k.x - 3.0, k.y - 4.0, k.x + 2.0, k.y - 4.0, ink('0'));
        line(k.x + 2.0, k.y - 4.0, k.x + 2.0, k.y, ink('0'));
        line(k.x - 3.0, k.y, k.x + 2.0, k.y, ink('0'));
        line(k.x - 1.0, k.y, k.x + 3.0, k.y + 4.0, ink('0'));
    }
}

/// The menu's picture of Sand Nomad, in the shell's vector primitives: a
/// sand ship on a dune, the sun, and the six motives in their hexagon.
/// Everything stays within half the width across and a fifth of it up and
/// down, like the other emblems.
pub fn emblem(frame: &Frame, centre: MqVec2, width: f32, clock: f32) {
    let u = width / 100.0;
    let s = frame.scale() * u;
    let at = |x: f32, y: f32| centre + MqVec2::new(x, y) * s;
    // The sun, low over the basin.
    frame.circle(at(26.0, -6.0), 9.0 * u, ink('y'));
    frame.circle(at(26.0, -6.0), 6.5 * u, ink('F'));
    // A dune, in bands.
    for (i, (y, ch)) in [(12.0, '9'), (15.0, '8'), (18.0, '6')].iter().enumerate() {
        let wobble = clock.mul_add(0.5, i as f32).sin() * 0.6;
        frame.rect(
            at(-48.0, *y + wobble),
            MqVec2::new(96.0 * u, 3.4 * u),
            ink(*ch),
        );
    }
    // The ship, rocking on it.
    let rock = (clock * 1.6).sin() * 1.2;
    let base = at(-4.0, 11.0 + rock);
    let p = |x: f32, y: f32| base + MqVec2::new(x, y) * s;
    Frame::triangle(p(-13.0, 0.0), p(13.0, 0.0), p(9.0, 4.0), ink('4'));
    Frame::triangle(p(-13.0, 0.0), p(9.0, 4.0), p(-9.0, 4.0), ink('3'));
    frame.line(p(0.0, 0.0), p(0.0, -22.0), 1.2 * u, ink('3'));
    Frame::triangle(p(1.5, -21.0), p(12.0, -3.0), p(1.5, -2.0), ink('k'));
    Frame::triangle(p(-1.5, -17.0), p(-9.0, -3.0), p(-1.5, -3.0), ink('l'));
    // The six motives around their hexagon, turning slowly.
    let hex = at(-30.0, -4.0);
    for (i, &m) in Motive::ALL.iter().enumerate() {
        let a = clock.mul_add(0.2, (i as f32 / 6.0).mul_add(TAU, -PI * 2.0 / 3.0));
        let q = hex + MqVec2::new(a.cos(), a.sin()) * 10.0 * s;
        frame.circle(q, 3.0 * u, ink('0'));
        frame.circle(q, 2.2 * u, ink(art::motive_ink(m)));
    }
    frame.ring(hex, 1.8 * u, 0.8 * u, ink('E'));
}
