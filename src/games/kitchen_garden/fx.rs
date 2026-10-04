//! Cosmetic motion: particles, food flying to the pantry, a shake.
//!
//! None of it is rules. It is seeded from a fixed number so it looks the
//! same every run, and nothing in the day ever reads it.

use game_prototypes::cooking::Food;
use macroquad::color::Color;
use macroquad::math::{Vec2, vec2};

use super::pen::ink;

/// What a particle looks like.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shape {
    /// A soft round mote: dust, flour, steam, smoke.
    Mote,
    /// A four-pointed sparkle.
    Spark,
    Heart,
    Coin,
    /// A little square chip: wood, carrot.
    Chip,
}

#[derive(Clone, Copy, Debug)]
pub struct Particle {
    pub pos: Vec2,
    pub vel: Vec2,
    /// Seconds lived, and seconds it lives.
    pub age: f32,
    pub life: f32,
    pub size: f32,
    pub color: Color,
    pub shape: Shape,
    /// Pull toward the ground (positive) or the sky (negative).
    pub gravity: f32,
    /// Where it is headed, for coins on their way to the jar.
    pub home: Option<Vec2>,
}

impl Particle {
    /// `0..=1` of its life gone.
    #[must_use]
    pub fn spent(&self) -> f32 {
        (self.age / self.life).clamp(0.0, 1.0)
    }
}

/// Food in the air between two places.
#[derive(Clone, Copy, Debug)]
pub struct Flyer {
    pub food: Food,
    pub from: Vec2,
    pub to: Vec2,
    /// Seconds since launch; negative while waiting to go.
    pub age: f32,
}

impl Flyer {
    pub const TIME: f32 = 0.32;

    /// Where it is now, on a little arc.
    #[must_use]
    pub fn at(&self) -> Vec2 {
        let t = (self.age / Self::TIME).clamp(0.0, 1.0);
        let ease = (1.0 - t).mul_add(-(1.0 - t), 1.0);
        let lift = (t * std::f32::consts::PI).sin() * 34.0;
        self.from.lerp(self.to, ease) - vec2(0.0, lift)
    }
}

#[derive(Default)]
pub struct Fx {
    pub particles: Vec<Particle>,
    pub flyers: Vec<Flyer>,
    /// Seconds of shake left.
    pub shake: f32,
    rng: Option<fastrand::Rng>,
}

impl Fx {
    fn rng(&mut self) -> &mut fastrand::Rng {
        self.rng
            .get_or_insert_with(|| fastrand::Rng::with_seed(0x4B47_4152))
    }

    fn spread(&mut self, n: f32) -> f32 {
        self.rng().f32().mul_add(2.0, -1.0) * n
    }

    pub fn update(&mut self, dt: f32) {
        for p in &mut self.particles {
            p.age += dt;
            if let Some(home) = p.home {
                // Coins wheel out, then home in on the jar.
                let pull = (home - p.pos) * (p.spent() * 9.0);
                p.vel = (p.vel + pull * dt) * 2.4f32.mul_add(-dt, 1.0).max(0.0);
            } else {
                p.vel.y = p.gravity.mul_add(dt, p.vel.y);
                p.vel *= 1.6f32.mul_add(-dt, 1.0).max(0.0);
            }
            p.pos += p.vel * dt;
        }
        self.particles.retain(|p| p.age < p.life);
        for f in &mut self.flyers {
            f.age += dt;
        }
        self.flyers.retain(|f| f.age < Flyer::TIME);
        self.shake = (self.shake - dt).max(0.0);
    }

    /// The camera's offset this frame, from any shake.
    #[must_use]
    pub fn offset(&self, clock: f32) -> Vec2 {
        if self.shake <= 0.0 {
            return Vec2::ZERO;
        }
        let k = self.shake * 10.0;
        vec2((clock * 71.0).sin() * k, (clock * 53.0).cos() * k)
    }

    pub const fn shake(&mut self, secs: f32) {
        self.shake = self.shake.max(secs);
    }

    pub fn fly(&mut self, food: Food, from: Vec2, to: Vec2) {
        self.fly_after(food, from, to, 0.0);
    }

    pub fn fly_after(&mut self, food: Food, from: Vec2, to: Vec2, delay: f32) {
        self.flyers.push(Flyer {
            food,
            from,
            to,
            age: -delay,
        });
    }

    fn burst(&mut self, at: Vec2, n: usize, make: impl Fn(&mut Self) -> Particle) {
        for _ in 0..n {
            let mut p = make(self);
            p.pos = at + vec2(self.spread(10.0), self.spread(6.0));
            self.particles.push(p);
        }
    }

    const fn base(shape: Shape, color: Color) -> Particle {
        Particle {
            pos: Vec2::ZERO,
            vel: Vec2::ZERO,
            age: 0.0,
            life: 0.6,
            size: 3.0,
            color,
            shape,
            gravity: 0.0,
            home: None,
        }
    }

    /// Dust, flour, droplets: a puff of motes in `color`.
    pub fn dust(&mut self, at: Vec2, n: usize, color: Color) {
        self.burst(at, n, |fx| {
            let vel = vec2(fx.spread(70.0), fx.spread(30.0) - 50.0);
            let size = fx.rng().f32().mul_add(2.0, 2.0);
            Particle {
                vel,
                size,
                life: 0.5,
                gravity: 260.0,
                ..Self::base(Shape::Mote, color)
            }
        });
    }

    pub fn chips(&mut self, at: Vec2, n: usize) {
        self.burst(at, n, |fx| {
            let vel = vec2(fx.spread(90.0), fx.spread(20.0) - 90.0);
            Particle {
                vel,
                size: 3.0,
                life: 0.7,
                gravity: 420.0,
                ..Self::base(Shape::Chip, ink::WOOD_LIGHT)
            }
        });
    }

    pub fn sparkles(&mut self, at: Vec2, n: usize) {
        self.burst(at, n, |fx| {
            let vel = vec2(fx.spread(50.0), fx.spread(20.0) - 40.0);
            let size = fx.rng().f32().mul_add(3.0, 4.0);
            Particle {
                vel,
                size,
                life: 0.8,
                ..Self::base(Shape::Spark, ink::GLOW)
            }
        });
    }

    pub fn embers(&mut self, at: Vec2, n: usize) {
        self.burst(at, n, |fx| {
            let vel = vec2(fx.spread(40.0), fx.spread(20.0) - 70.0);
            let color = if fx.rng().bool() {
                ink::FLAME_HOT
            } else {
                ink::FLAME
            };
            Particle {
                vel,
                size: 2.2,
                life: 0.9,
                gravity: -40.0,
                ..Self::base(Shape::Mote, color)
            }
        });
    }

    /// Steam off a pot: slow, rising, fading.
    pub fn steam(&mut self, at: Vec2, n: usize) {
        self.burst(at, n, |fx| {
            let vel = vec2(fx.spread(8.0), -22.0 + fx.spread(6.0));
            let size = fx.rng().f32().mul_add(3.0, 4.0);
            Particle {
                vel,
                size,
                life: 1.4,
                gravity: -12.0,
                ..Self::base(Shape::Mote, Color::new(1.0, 1.0, 1.0, 0.5))
            }
        });
    }

    pub fn smoke(&mut self, at: Vec2, n: usize) {
        self.burst(at, n, |fx| {
            let vel = vec2(fx.spread(14.0), -30.0 + fx.spread(8.0));
            let size = fx.rng().f32().mul_add(4.0, 5.0);
            Particle {
                vel,
                size,
                life: 1.8,
                gravity: -10.0,
                ..Self::base(Shape::Mote, Color::new(0.3, 0.27, 0.26, 0.55))
            }
        });
    }

    pub fn hearts(&mut self, at: Vec2, n: usize) {
        self.burst(at, n, |fx| {
            let vel = vec2(fx.spread(40.0), -60.0 + fx.spread(15.0));
            Particle {
                vel,
                size: 6.0,
                life: 1.1,
                gravity: -20.0,
                ..Self::base(Shape::Heart, ink::HEART)
            }
        });
    }

    pub fn coin_burst(&mut self, at: Vec2, n: usize) {
        self.burst(at, n, |fx| {
            let vel = vec2(fx.spread(60.0), -90.0 + fx.spread(20.0));
            Particle {
                vel,
                size: 5.0,
                life: 0.6,
                gravity: 400.0,
                ..Self::base(Shape::Coin, ink::GOLD)
            }
        });
    }

    /// Coins from a customer arcing into the jar.
    pub fn coins_to(&mut self, at: Vec2, jar: Vec2, n: usize) {
        for i in 0..n {
            let vel = vec2(self.spread(120.0), -120.0 + self.spread(40.0));
            let life = (i as f32).mul_add(0.05, 0.9);
            let mut p = Self::base(Shape::Coin, ink::GOLD);
            p.pos = at + vec2(self.spread(8.0), self.spread(8.0));
            p.vel = vel;
            p.size = 5.5;
            p.life = life;
            p.home = Some(jar);
            self.particles.push(p);
        }
    }
}
