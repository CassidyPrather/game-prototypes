//! The pixel art, as data.
//!
//! Every sprite is drawn by hand as rows of palette characters, one
//! character a pixel, on a 16-pixel grid: items are 16 pixels a cell,
//! people and places 32. Keeping the art here, in the pure library, means
//! the tests can check every sprite is rectangular and uses only colours
//! the palette has; the frontend bakes each into a texture once.
//!
//! The six motive marks are the suits from Cassidy's motif designs — the
//! chalice, club, heart, spike, diamond and spade — redrawn small, and the
//! anima mark is the ring with an eye in it.

use crate::sand_nomad::barter::{Culture, Emote};
use crate::sand_nomad::item::Kind;
use crate::sand_nomad::motive::Motive;
use crate::sand_nomad::world::Site;

/// A sprite: rows of palette characters, all the same length. `.` is
/// transparent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sprite(pub &'static [&'static str]);

impl Sprite {
    /// Width in pixels.
    #[must_use]
    pub fn width(self) -> usize {
        self.0.first().map_or(0, |row| row.len())
    }

    /// Height in pixels.
    #[must_use]
    pub const fn height(self) -> usize {
        self.0.len()
    }

    /// The sprite as RGBA bytes, row by row.
    #[must_use]
    pub fn rgba(self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.width() * self.height() * 4);
        for row in self.0 {
            for ch in row.chars() {
                out.extend_from_slice(&colour(ch).unwrap_or([0, 0, 0, 0]));
            }
        }
        out
    }
}

/// The basin's colours: warm sand and wood, brass, the Harbor's crimson,
/// the steppe's teal, and the six motive colours from the designs.
pub const PALETTE: [(char, u32); 46] = [
    ('0', 0x1a_10_14), // outline
    ('1', 0x3a_26_26),
    ('2', 0x5a_3a_2c),
    ('3', 0x7e_52_36),
    ('4', 0xa8_6e_40), // wood
    ('5', 0xd0_9a_5c),
    ('6', 0xf0_cf_94),
    ('7', 0xff_f4_d6), // cream
    ('8', 0xdc_ae_70), // sand
    ('9', 0xb8_83_50),
    ('a', 0x8e_60_40),
    ('b', 0x34_4c_66), // steel
    ('c', 0x78_96_b0),
    ('d', 0xc8_da_e6),
    ('e', 0x7a_50_18), // brass
    ('f', 0xc8_92_2e),
    ('g', 0xf6_d2_66),
    ('h', 0x5c_14_20), // crimson
    ('i', 0xa4_20_2e),
    ('j', 0xe0_58_4c),
    ('k', 0xee_e4_d0), // bone, cloth
    ('l', 0xbc_ae_96),
    ('m', 0x6a_62_72), // grey
    ('n', 0xa8_a0_ac),
    ('o', 0x24_46_3e), // steppe teal
    ('p', 0x3c_8a_78),
    ('q', 0x74_c8_b2),
    ('r', 0x1a_24_40), // night
    ('s', 0xf0_a8_82), // skin
    ('t', 0xc0_76_50),
    ('u', 0x80_46_2e),
    ('v', 0x4a_24_70),
    ('w', 0xff_ff_ff),
    ('x', 0xff_78_00), // zeal
    ('y', 0xff_c4_39), // bliss
    ('z', 0x00_a2_cc), // repose
    ('L', 0xcc_00_00), // love
    ('P', 0x99_00_ff), // pain
    ('H', 0x2a_1e_2e), // hate
    ('F', 0xff_e2_7a), // fire
    ('R', 0xe8_50_1c),
    ('S', 0xf6_f6_f0), // salt
    ('G', 0x9e_d4_e0), // glass
    ('B', 0x2c_74_a8),
    ('E', 0xe8_6a_a0), // pink
    ('I', 0x8a_a0_48), // olive
];

/// The colour a palette character stands for, or `None` for transparent
/// or unknown.
#[must_use]
pub fn colour(ch: char) -> Option<[u8; 4]> {
    PALETTE.iter().find(|(c, _)| *c == ch).map(|&(_, rgb)| {
        [
            ((rgb >> 16) & 0xff) as u8,
            ((rgb >> 8) & 0xff) as u8,
            (rgb & 0xff) as u8,
            0xff,
        ]
    })
}

/// The same colour as `0xRRGGBB`, for the frontend's procedural drawing.
#[must_use]
pub fn rgb(ch: char) -> u32 {
    PALETTE
        .iter()
        .find(|(c, _)| *c == ch)
        .map_or(0xff_00_ff, |&(_, rgb)| rgb)
}

/// A motive's colour, as a palette character.
#[must_use]
pub const fn motive_ink(motive: Motive) -> char {
    match motive {
        Motive::Bliss => 'y',
        Motive::Repose => 'z',
        Motive::Love => 'L',
        Motive::Pain => 'P',
        Motive::Zeal => 'x',
        Motive::Hate => 'H',
    }
}

// Items, one cell of the hold a 16-pixel square.

pub const WATER: Sprite = Sprite(&[
    "................",
    ".....000000.....",
    "....06666660....",
    ".....099990.....",
    "......0990......",
    ".....099990.....",
    "....09888890....",
    "...0988888890...",
    "...0986888890...",
    "...0986888990...",
    "...0998889990...",
    "....09999990....",
    ".....0aaaa0.....",
    "......0000......",
    "................",
    "................",
]);

pub const FLUTE: Sprite = Sprite(&[
    "................",
    "................",
    ".............00.",
    "............0kk0",
    "...........0kl0.",
    "..........0k10..",
    ".........0kl0...",
    "........0k10....",
    ".......0kl0.....",
    "......0k10......",
    ".....0kl0.......",
    "....0kk0........",
    "...0kl0.........",
    "...0l0..........",
    "....0...........",
    "................",
]);

pub const COMB: Sprite = Sprite(&[
    "................",
    "................",
    "................",
    "....00000000....",
    "...0kkkkkkkk0...",
    "..0kk7kkkkkkk0..",
    "..0kllllllllk0..",
    "..0k0k0k0k0k00..",
    "..0k0k0k0k0k0...",
    "..0k0k0k0k0k0...",
    "..0k0k0k0k0k0...",
    "..0l0l0l0l0l0...",
    "...0.0.0.0.0....",
    "................",
    "................",
    "................",
]);

pub const BEADS: Sprite = Sprite(&[
    "................",
    "......0000......",
    "....0045540.....",
    "...04500005400..",
    "..0450....0540..",
    "..050......050..",
    "..040......040..",
    "..050......050..",
    "..0450....0540..",
    "...04500005400..",
    "....00455400....",
    "......0330......",
    ".....033330.....",
    ".....03..30.....",
    "......0..0......",
    "................",
]);

pub const RUG: Sprite = Sprite(&[
    "................................",
    "................................",
    "..0000000000000000000000000000..",
    "k0hhhhhhhhhhhhhhhhhhhhhhhhhhhh0k",
    ".0hiiiiiiiiiiiiiiiiiiiiiiiiiih0.",
    "k0hiiyiiiiiyiiiiiyiiiiiyiiiiih0k",
    ".0hiyjyiiiyjyiiiyjyiiiyjyiiiih0.",
    "k0hyjijyiyjijyiyjijyiyjijyiiih0k",
    ".0hiyjyiiiyjyiiiyjyiiiyjyiiiih0.",
    "k0hiiyiiiiiyiiiiiyiiiiiyiiiiih0k",
    ".0hiiiiiiiiiiiiiiiiiiiiiiiiiih0.",
    "k0hhhhhhhhhhhhhhhhhhhhhhhhhhhh0k",
    "..0000000000000000000000000000..",
    "................................",
    "................................",
    "................................",
]);

pub const COMPASS: Sprite = Sprite(&[
    "................",
    "......0000......",
    "....00gggg00....",
    "...0gffffffg0...",
    "..0gf0kkkk0fg0..",
    "..0f0kkjkkk0f0..",
    ".0gf0kkjjkk0fg0.",
    ".0gf0kkkkkk0fg0.",
    ".0ff0kkcckk0ff0.",
    ".0ef0kkkckk0fe0.",
    "..0ef0kkkk0fe0..",
    "..0eff0000ffe0..",
    "...0eeffffee0...",
    "....00eeee00....",
    "......0000......",
    "................",
]);

pub const PLANK: Sprite = Sprite(&[
    "................................",
    "................................",
    "................................",
    "................................",
    "..0000000000000000000000000000..",
    ".0555555555555555555555555555540",
    ".0544444444434444444444444444430",
    ".0544443444444444444444434444430",
    ".0544444444444444434444444444430",
    ".0433333333333333333333333333320",
    "..0000000000000000000000000000..",
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
]);

pub const LETTERS: Sprite = Sprite(&[
    "................",
    "................",
    "................",
    "...00000000000..",
    "..0kkkkkkkkkk0..",
    "..0k7kkkkkk7k0..",
    ".0kl7kkkkk7lk0..",
    ".0klll7kk7lllk0.",
    ".0klkll77llklk0.",
    ".0kklllLLllkkk0.",
    ".0kkkkLiLkkkkk0.",
    ".0kkkkkLkkkkkk0.",
    ".0lllllllllll0..",
    "..00000000000...",
    "................",
    "................",
]);

pub const SPYGLASS: Sprite = Sprite(&[
    "................................",
    "................................",
    "................................",
    "................................",
    "..................00000000000...",
    "..........00000000gggggggggg00..",
    "...0000000ggggggg0fffffffffff0G0",
    "..0ggggg0ffffffff0fffffffffff0G0",
    "..0fffff0eeeeeeee0eeeeeeeeeee0B0",
    "...0000000eeeeeee0eeeeeeeeee00..",
    "..........00000000000000000.....",
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
]);

pub const SHELL: Sprite = Sprite(&[
    "................",
    "................",
    "................",
    ".......00.......",
    ".....007700.....",
    "....0k7kk7k0....",
    "...0kEkkkkEk0...",
    "..0kkEkkkkEkk0..",
    "..0kEkkEEkkEk0..",
    "..0kEkkEEkkEk0..",
    "...0kEkEEkEk0...",
    "....0kEEEEk0....",
    ".....0llll0.....",
    "....0llllll0....",
    ".....000000.....",
    "................",
]);

pub const MUSIC_BOX: Sprite = Sprite(&[
    "................",
    "................",
    "..........0.....",
    ".........0f0....",
    "..........0.....",
    "...0000000000...",
    "..0555555555540.",
    "..03444444444300",
    "..0gggggggggg0f0",
    "..04433zz334400.",
    "..0443z33z3440..",
    "..0443z33z3440..",
    "..0433333333340.",
    "..0gggggggggg0..",
    "...0000000000...",
    "................",
]);

pub const DOLL: Sprite = Sprite(&[
    "................",
    "......0000......",
    ".....033330.....",
    "....03ssss30....",
    "....0s0ss0s0....",
    "....0ssssss0....",
    ".....0sEEs0.....",
    "....00yyyy00....",
    "...0s0yEEy0s0...",
    "...00yyyyyy00...",
    ".....0yEEy0.....",
    "....0yyyyyy0....",
    "....0yyyyyy0....",
    ".....0s00s0.....",
    ".....00..00.....",
    "................",
]);

pub const LOCKET: Sprite = Sprite(&[
    "......0000......",
    ".....0d00d0.....",
    "......0..0......",
    ".......00.......",
    "......0dd0......",
    ".....000000.....",
    "....0dddddd0....",
    "...0dcccccccd0..",
    "...0dccdLdcccd0.",
    "..0dccLLLLLccc0.",
    "..0dccLLLLLccb0.",
    "..0dcccLLLcccb0.",
    "...0dcccLcccb0..",
    "....0bcccccb0...",
    ".....0bbbbb0....",
    "......00000.....",
]);

pub const KETTLE: Sprite = Sprite(&[
    "................",
    "................",
    ".......00.......",
    "......0990......",
    "....00000000....",
    "...0a888888a0...",
    "..0a88888889a0..",
    "0.0886888888a0..",
    "000868888889a000",
    ".0086888888990.0",
    "..08888888899a0.",
    "..09888888999a0.",
    "...0999999aaa0..",
    "....0aaaaaaa0...",
    ".....0000000....",
    "................",
]);

pub const ROPE: Sprite = Sprite(&[
    "................",
    "................",
    ".....000000.....",
    "...0054455400...",
    "..054400004450..",
    "..0540555504450.",
    ".054050000504450",
    ".054045554045450",
    ".054040040445450",
    ".054045544045450",
    "..05400000445450",
    "..00554444554500",
    "....000000005450",
    "...........05450",
    "............0000",
    "................",
]);

pub const FIGURINE: Sprite = Sprite(&[
    "................",
    ".......0........",
    "......0k0.......",
    "......0kk0......",
    ".....0kkkk0.....",
    ".....0kkkkk0....",
    "....0kkkkkk00...",
    "....00000000....",
    ".......0........",
    "..000000000000..",
    "..0555555555550.",
    "...0444444444400",
    "....04433344440.",
    "....0000000000..",
    "...0a0.....0a0..",
    "...00.......00..",
]);

pub const OAR: Sprite = Sprite(&[
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    "...........................000..",
    "........................000550..",
    "..00000000000000000000004545550.",
    ".0555555555555555555555444555540",
    "..00000000000000000000004444440.",
    "........................0003440.",
    "...........................000..",
    "................................",
    "................................",
    "................................",
    "................................",
]);

pub const SALT: Sprite = Sprite(&[
    "................",
    "................",
    "................",
    "................",
    "....00000000....",
    "...0SSSSSSSS0...",
    "..0SSwSSSSSSS0..",
    "..0000000000000.",
    "..0SSSSSSSSSSn0.",
    "..0SwSSSSSSSSn0.",
    "..0SSSSSSSSSSn0.",
    "..0SSSSSSSSSnn0.",
    "..0nnnnnnnnnnn0.",
    "...00000000000..",
    "................",
    "................",
]);

pub const GLASS_FLOWER: Sprite = Sprite(&[
    "................",
    "......0..0......",
    ".....0G00G0.....",
    "....0G0GG0G0....",
    ".0.0GwG00GwG0.0.",
    "0G00GGBGGBGG00G0",
    ".0GGBwGGGGwBGG0.",
    "..0BGGGwwGGGB0..",
    ".0GGBwGGGGwBGG0.",
    "0G00GGBGGBGG00G0",
    ".0.0GwG00GwG0.0.",
    "....0G0GG0G0....",
    ".....0B00B0.....",
    "......0000......",
    "................",
    "................",
]);

pub const DAGGER: Sprite = Sprite(&[
    "................",
    "..............0.",
    ".............0d0",
    "............0dc0",
    "...........0dc0.",
    "..........0dc0..",
    ".........0dc0...",
    "........0dc0....",
    "...0...0dc0.....",
    "...0f00dc0......",
    "....0fdc0.......",
    "....00f0........",
    "...0330f0.......",
    "..0330..0f0.....",
    ".0330....0......",
    ".000............",
]);

pub const FLAG: Sprite = Sprite(&[
    "................",
    ".00.............",
    ".0500000000000..",
    ".040HHHHHHHHH0..",
    ".040HHHHkHHHH0..",
    ".040HHHkkkHHH0..",
    ".040HHkkkkkHH00.",
    ".040HkkkkkkkHH0.",
    ".040HkkHkHkkH00.",
    ".040HHHHkHHHH0..",
    ".040HHHkkkHHH0..",
    ".040000000000...",
    ".040............",
    ".040............",
    ".030............",
    ".000............",
]);

pub const PISTOL: Sprite = Sprite(&[
    "................",
    "................",
    "................",
    "..........0.....",
    ".000000000g0....",
    "0cddddddcc0f0...",
    "0bbbbbbbbbbbf0..",
    ".0000000g0344400",
    "........0e034440",
    "........0f003440",
    ".........00.0344",
    "............0344",
    "............0344",
    "............0233",
    ".............000",
    "................",
]);

pub const COINS: Sprite = Sprite(&[
    "................",
    "................",
    "......0000......",
    ".....0i00i0.....",
    "......0ii0......",
    ".....000000.....",
    "....0hiiiih0....",
    "...0hiiiiiih0...",
    "..0hiiiiiiiih0..",
    "..0hiiggiiiih0..",
    "..0hiigfgiiih0..",
    "..0hiiggiiiih0..",
    "...0hiiiiiih0g0.",
    "....0hhhhhh0gfg0",
    ".....000000.0g0.",
    "................",
]);

pub const TIN: Sprite = Sprite(&[
    "................",
    "................",
    "................",
    "....00000000....",
    "...0dddddddd0...",
    "..0cddddddddc0..",
    "..0cccccccccc0..",
    "..0iiiiiiiiii0..",
    "..0iijjjiiiii0..",
    "..0ijiiijiiii0..",
    "..0iijjjiiiii0..",
    "..0iiiiiiiiii0..",
    "..0cccccccccc0..",
    "..0bbbbbbbbbb0..",
    "...0000000000...",
    "................",
]);

pub const RIB: Sprite = Sprite(&[
    "................",
    "........00......",
    ".......0kk0.....",
    "......0kkl0.....",
    ".....0kkl0......",
    ".....0kl0.......",
    "....0kkl0.......",
    "....0kl0........",
    "...0kkl0........",
    "...0kl0.........",
    "...0kl0.........",
    "..0kkl0.........",
    "..0kl0..........",
    "..0kl0..........",
    "..0kl0..........",
    "..0kl0..........",
    "..0kl0..........",
    "..0kl0..........",
    "..0kkl0.........",
    "...0kl0.........",
    "...0kl0.........",
    "...0kkl0........",
    "....0kl0........",
    "....0kkl0.......",
    ".....0kl0.......",
    ".....0kkl0......",
    "......0kkl0.....",
    ".......0kk0.....",
    "........00......",
    "................",
    "................",
    "................",
]);

pub const TOOTH: Sprite = Sprite(&[
    "................",
    "................",
    "....00000000....",
    "...0kkkkkkkk0...",
    "..0k7kkkkkkkl0..",
    "..0k7kkkkkkkl0..",
    "..0kkkkkkkkkl0..",
    "...0kkkkkkkl0...",
    "...0kkkkkkkl0...",
    "....0kkkkkl0....",
    "....0kkkkkl0....",
    ".....0kkkl0.....",
    ".....0kkl0......",
    "......0kl0......",
    ".......00.......",
    "................",
]);

pub const CLOCKBIRD: Sprite = Sprite(&[
    "................",
    "..........000...",
    ".........0ggg0..",
    "........0gg0gf0.",
    "........0ggggf00",
    "...000..0fgggf0e",
    "..0gg00.0ffff0..",
    "...0ggg00fffff0.",
    "....0gggfffffff0",
    ".....0ffff0d0ff0",
    "......0ffe0dd0e0",
    ".......0eee000e0",
    "........0000ee0.",
    ".........0e0000.",
    "........00e00...",
    "................",
]);

pub const MASK: Sprite = Sprite(&[
    "................",
    "......0000......",
    "....00xxxx00....",
    "...0xx5555xx0...",
    "..0x55555555x0..",
    "..0x5555555540..",
    "..0500055000540.",
    "..05055550555x0.",
    "..0x5555555554x0",
    "..0x555x55555x0.",
    "...0x55x555540..",
    "...0x5500055x0..",
    "....0x55555x0...",
    ".....0xx4xx0....",
    "......00000.....",
    "................",
]);

pub const IDOL: Sprite = Sprite(&[
    "................",
    "......0000......",
    ".....0xxxx0.....",
    ".....044440.....",
    ".....0x44x0.....",
    ".....044440.....",
    "......0440......",
    "....00444400....",
    "...0x044440x0...",
    "....00444400....",
    ".....044440.....",
    ".....043340.....",
    ".....044440.....",
    "....03333330....",
    "....00000000....",
    "................",
]);

pub const BUST: Sprite = Sprite(&[
    "................................",
    "................................",
    "............00000000............",
    "..........00nnnnnnnn00..........",
    ".........0nnnnnnnnnnnn0.........",
    "........0nnnnkknnnnnnnm0........",
    "........0nnnknnnnnnnnnm0........",
    ".......0nnnnnnnnnnnnnnnm0.......",
    ".......0nnkknnnnnnkknnnm0.......",
    ".......0nnm0nnnnnnm0nnnm0.......",
    ".......0nnnnnnnnnnnnnnnm0.......",
    ".......0nnnnnnmmnnnnnnnm0.......",
    "........0nnnnnnmnnnnnnm0........",
    "........0nnnnnnnnnnnnnm0........",
    "........0nnnnmmmmnnnnnm0........",
    ".........0nnnnnnnnnnnm0.........",
    "..........0nnnnnnnnmm0..........",
    "...........0nnnnnnnm0...........",
    "..........00nnnnnnnm00..........",
    ".......000nnnnnnnnnnnm000.......",
    ".....00nnnnnnnnnnnnnnnnnm00.....",
    "....0nnnnnnnnnnnnnnnnnnnnnm0....",
    "...0nnnkknnnnnnnnnnnnnnnnnnm0...",
    "...0nnknnnnnnnnnnnnnnnnnnnnm0...",
    "...0nnnnnnnnnnnnnnnnnnnnnnnm0...",
    "...0mmmmmmmmmmmmmmmmmmmmmmmm0...",
    "....000000000000000000000000....",
    "........0mmmmmmmmmmmmmmmm0......",
    "........0nnnnnnnnnnnnnnnm0......",
    "........0mmmmmmmmmmmmmmmm0......",
    ".........0000000000000000.......",
    "................................",
]);

pub const MEDAL: Sprite = Sprite(&[
    "................",
    ".....0....0.....",
    "....0i0..0i0....",
    ".....0i00i0.....",
    "......0ii0......",
    ".....000000.....",
    "....0nnnnnn0....",
    "...0nnmmmmnn0...",
    "..0nnmnnnnmnn0..",
    "..0nmnkknnnmn0..",
    "..0nmnknnnnmn0..",
    "..0nmnnnnnnmn0..",
    "..0nnmnnnnmnm0..",
    "...0nnmmmmnm0...",
    "....0mmmmmm0....",
    ".....000000.....",
]);

pub const MAP: Sprite = Sprite(&[
    "................",
    "................",
    "..0000000000000.",
    ".055555555555550",
    ".044444444444440",
    "..0000000000000.",
    "...0kkkkkkkkk0..",
    "...0k8kkIIkkk0..",
    "...0kk88kIkjk0..",
    "...0kkk8kkkkk0..",
    "...0kIkk88kkk0..",
    "...0kIIkkk8kk0..",
    "..0000000000000.",
    ".055555555555550",
    ".044444444444440",
    "..0000000000000.",
]);

pub const LAMP: Sprite = Sprite(&[
    "................",
    "..........0.....",
    ".........0F0....",
    "........0FR0....",
    "........0R0.....",
    ".......000......",
    "..000009a0......",
    ".0999999990.....",
    "0986888889900...",
    "0988888889990a0.",
    ".099999999990a0.",
    "..0aaaaaaaa0aa0.",
    "...00000000000..",
    "................",
    "................",
    "................",
]);

pub const CHARM: Sprite = Sprite(&[
    "................",
    "......0000......",
    ".....0....0.....",
    ".....0....0.....",
    "......0..0......",
    ".......00.......",
    "......0kk0......",
    ".....0kPPk0.....",
    ".....0kPPk0.....",
    "......0PP0......",
    "......0kk0......",
    ".....0kPPk0.....",
    ".....0kkkl0.....",
    "......0kl0......",
    ".......00.......",
    "................",
]);

/// An item's sprite.
#[must_use]
pub const fn item(kind: Kind) -> Sprite {
    match kind {
        Kind::Water => WATER,
        Kind::Flute => FLUTE,
        Kind::Comb => COMB,
        Kind::Beads => BEADS,
        Kind::Rug => RUG,
        Kind::Compass => COMPASS,
        Kind::Plank => PLANK,
        Kind::Letters => LETTERS,
        Kind::Spyglass => SPYGLASS,
        Kind::Shell => SHELL,
        Kind::MusicBox => MUSIC_BOX,
        Kind::Doll => DOLL,
        Kind::Locket => LOCKET,
        Kind::Kettle => KETTLE,
        Kind::Rope => ROPE,
        Kind::Figurine => FIGURINE,
        Kind::Oar => OAR,
        Kind::Salt => SALT,
        Kind::GlassFlower => GLASS_FLOWER,
        Kind::Dagger => DAGGER,
        Kind::Flag => FLAG,
        Kind::Pistol => PISTOL,
        Kind::Coins => COINS,
        Kind::Tin => TIN,
        Kind::Rib => RIB,
        Kind::Tooth => TOOTH,
        Kind::Clockbird => CLOCKBIRD,
        Kind::Mask => MASK,
        Kind::Idol => IDOL,
        Kind::Bust => BUST,
        Kind::Medal => MEDAL,
        Kind::Map => MAP,
        Kind::Lamp => LAMP,
        Kind::Charm => CHARM,
    }
}

// Motive marks, painted on things, 8 pixels square.

pub const MARK_BLISS: Sprite = Sprite(&[
    ".000000.", "0yyyyyy0", "0yyyyyy0", ".0yyyy0.", "..0yy0..", "..0yy0..", ".0yyyy0.", "..0000..",
]);

pub const MARK_REPOSE: Sprite = Sprite(&[
    "..0zz0..", ".0zzzz0.", "00zzzz00", "zzzzzzzz", "zzzzzzzz", "0zz00zz0", ".00zz00.", ".0zzzz0.",
]);

pub const MARK_LOVE: Sprite = Sprite(&[
    ".00..00.", "0LL00LL0", "LLLLLLLL", "LLLLLLLL", "0LLLLLL0", ".0LLLL0.", "..0LL0..", "...00...",
]);

pub const MARK_PAIN: Sprite = Sprite(&[
    "..0P0...", "..0P0...", ".0PPP0..", ".0PPP0..", ".0PPP0..", ".0PPP0..", "..0P0...", "..0P0...",
]);

pub const MARK_ZEAL: Sprite = Sprite(&[
    "..0xx0..", ".0xxxx0.", "0xxxxxx0", "xxxxxxxx", "0xxxxxx0", ".0xxxx0.", "..0xx0..", "...00...",
]);

pub const MARK_HATE: Sprite = Sprite(&[
    "..nHHn..", ".nHHHHn.", "nHHHHHHn", "HHHHHHHH", "HHHHHHHH", "nHnHHnHn", ".nnHHnn.", ".nHHHHn.",
]);

pub const MARK_ANIMA: Sprite = Sprite(&[
    ".0EEEE0.", "0E0000E0", "E00EE00E", "E0EjjE0E", "E0EjjE0E", "E00EE00E", "0E0000E0", ".0EEEE0.",
]);

// Traders' faces, as they appear over their rugs.

pub const TRADER_DRIFTER: Sprite = Sprite(&[
    "................................",
    "................................",
    "................................",
    "............00000000............",
    "...........0qqqqqqqp0...........",
    "..........0qqqqqqqqqp0..........",
    ".........0qqqqqqqqqqqp0.........",
    "........0qqqqqqqqqqqqqp0........",
    "........0qqqqqqqqqqqqqp0........",
    ".......0qqqqqqqqqqqqqqqp0.......",
    ".......0qqqqqttttttqqqqp0.......",
    ".......0qqqqttttttttqqqp0.......",
    ".......0qqqttttttttttqqp0.......",
    ".......0qqqttttttttttqqp0.......",
    ".......0qqqtttttttttttqp0.......",
    "........0qqtt0ttttt0ttqp0.......",
    "........0qqtttttttttttqp0.......",
    "........0qqtttttutttttqp0.......",
    "........0qqtttttttttu0qp0.......",
    "........0qp0tttuuutu00qp0.......",
    "........0qp00tttttu0.0qp0.......",
    "........0qp00tttttu0.0qp0.......",
    "........0qp00tttttu0.0qp0.......",
    ".........000kttttttl0000........",
    ".......00kkkkkkkkkkkkkl00.......",
    "......0kkkkkkkkkkkkkkkkkl0......",
    "....00kkkkkkkkkkkkkkkkkkkl00....",
    "...0iiiiiiiiiiiiiiiiiiiiiiii0...",
    "...0iiiiiiiiiiiiiiiiiiiiiiii0...",
    "...0kkkkkkkkkkkkkkkkkkkkkkkl0...",
    "..0kkkkkkkkkkkkkkkkkkkkkkkkkl0..",
    "..0kkkkkkkkkkkkkkkkkkkkkkkkkl0..",
]);

pub const TRADER_KEEPER: Sprite = Sprite(&[
    "................................",
    "................................",
    "................................",
    "................................",
    ".............000000.............",
    "...........00nnnnnn00...........",
    "..........0nnnnnnnnnn0..........",
    ".........0nnnnnnnnnnnn0.........",
    "........0nnnnnnnnnnnnnn0........",
    "........0nnnnnnnnnnnnnnn0.......",
    "........0nnnnnnnnnnnnnnn0.......",
    "........0nnnnnnnnnnnnsnn0.......",
    "........0nnssfnnnnnfssnn0.......",
    "........0nnsfsfsssfsfsnn0.......",
    "........0nnss0sfffs0ssnn0.......",
    "........0nnsfsfsssfsfsnn0.......",
    ".........0sssfsstssfst00........",
    ".........0ssssssssssst0.........",
    "..........0sssstttsst0..........",
    "...........0ssssssst0...........",
    "............0ssssst0............",
    "............0ssssst0............",
    "............0sskkkt0............",
    ".........000Bsskkksb000.........",
    ".......00BBBBBBBBBBBBBb00.......",
    "......0BBBBBBBBBBBBBBBBBb0......",
    ".....0BBBBBBBBBBgBBBBBBBBb0.....",
    "....0BBBBBBBBBBBBBBBBBBBBBb0....",
    "...0BBBBBBBBBBBBBBBBBBBBBBBb0...",
    "...0BBBBBBBBBBBBgBBBBBBBBBBb0...",
    "..0BBBBBBBBBBBBBBBBBBBBBBBBBb0..",
    "..0BBBBBBBBBBBBBBBBBBBBBBBBBb0..",
]);

pub const TRADER_THESEAN: Sprite = Sprite(&[
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    "............00000000............",
    "..........00iiiiiiih00..........",
    ".........0iiiiiiiiiiih0.........",
    "........0iiiiiiiiiiiiih0........",
    "........0iiiiiiiiiiiiih0........",
    "........0iiiiiiiiiiiiiih0.......",
    "........0iiiiiiiiiiiiiiih0......",
    ".........0iiiiiiiiiiih000h0.....",
    ".........0uuiiiiiiiiu20.0h0.....",
    ".........0uuu0uuuuu0u20.000000..",
    ".........0uuuuuuuuuuu2005555540.",
    ".........0uuuuuu2uuuu2005555540.",
    ".........0uuuuuuuuuuu2005555540.",
    "..........0uuuu222uu20.05555540.",
    "...........0uuuuuuu20...004400..",
    "............0uuuuu20.....0440...",
    "............0uuuuu20.....0440...",
    "..........000uuuuu2000...0440...",
    ".........03333333333320..0440...",
    ".......00z333333333333b000440...",
    "......0zz33333333333333zb0440...",
    ".....0zzz33333333333333zzb440...",
    "....0zzzz33333333333333zzz440...",
    "...0zzzzz33333333333333zzz440...",
    "...0zzzzz33333333333333zzz440...",
    "..0zzzzz3333333333333333zz44b0..",
    "..0zzzzz3333333333333333zzzzb0..",
]);

pub const TRADER_SCRAPER: Sprite = Sprite(&[
    "................................",
    "................................",
    "................................",
    ".............000000.............",
    "...........0066666800...........",
    "..........066666666680..........",
    ".........06666666666680.........",
    ".......000666666666668000.......",
    "....000666666666666666668000....",
    "...06666666666666666666666680...",
    "..0666666666666666666666666680..",
    "..0699999999999999999999999990..",
    "...06666666666666666666666680...",
    "....000666666666666666668000....",
    ".......000tttttttttttu000.......",
    ".........0tt00tttt00tu0.........",
    ".........0tttttttttttu0.........",
    ".........0ttttttuttttu0.........",
    "..........0tttttttttu00.........",
    ".........0SSSSSSSSSSSSS0........",
    ".........0SSSSSSSSSSSSS0........",
    ".........0SSSSSSSSSSSSS0........",
    "..........000tttttu0000.........",
    ".........000Sttttttn000.........",
    ".......00SSSSSSSSSSSSSn00.......",
    "......0SSSSSSSSSSSSSSSSSn0......",
    ".....0SSSSSSSSSSSSSSSSSSSn0.....",
    "....0SSSSSSSSSSSSSSSSSSSSSn0....",
    "...0SSSSSSSSSSSSSSSSSSSSSSSn0...",
    "...0SSSSSSSSSSSSSSSSSSSSSSSn0...",
    "..0SSSSSSSSSSSSSSSSSSSSSSSSSn0..",
    "..0SSSSSSSSSSSSSSSSSSSSSSSSSn0..",
]);

pub const TRADER_PIRATE: Sprite = Sprite(&[
    "................................",
    "................................",
    "................................",
    "................................",
    ".............000000.............",
    "...........00HHHHHH00...........",
    "..........0HHHHHHHHHH0..........",
    ".........0HHHHHHHHHHHH0.........",
    "........0HHHHHHHnHHHHHH0........",
    "........0HHHHHHnnnHHHHH0........",
    "........0HHHHHHHnHHHHHHH0.......",
    "........0HHHHHHHHHHHHHHH0.......",
    "........0HHHHHHHHHHHHHHH0.......",
    ".......0H0tHHHHHHHHHHu00........",
    "......0H00ttHHHHHHHHHu0.........",
    ".......0H0ttHHHtttt0tu0.........",
    "........00ttHHHttttttu00........",
    ".........0ttttttuttttu0g0.......",
    "..........0tttttttttu0.0........",
    "...........0tttuuutu0...........",
    "............0tttttu0............",
    "............0tttttu0............",
    "............0tttttu0............",
    ".........000htttttt1000.........",
    ".......00hiihhhhhhhhhi100.......",
    "......0hhhiihhhhhhhhhiih10......",
    ".....0hhhhiihhhhhhhhhiihh10.....",
    "....0hhhhhiihhhhhhhhhiihhh10....",
    "...0hhhhhhiihhhhhhhhhiihhhh10...",
    "...0hhhhhhiihhhhhhhhhiihhhh10...",
    "..0hhhhhhhiihhhhhhhhhiihhhhh10..",
    "..0hhhhhhhiihhhhhhhhhiihhhhh10..",
]);

pub const TRADER_HARBOR: Sprite = Sprite(&[
    "................................",
    "................................",
    "..........0..........00.........",
    ".........0g00......00Hg0........",
    "........0gHHH00..00HHHHg0.......",
    ".......0HgHHHHH00HHHHHHg0.......",
    "......0HgHHHHHHHHHHHHHHHg0......",
    "......0gHHHHHHHHHHHHHHHHHg0.....",
    ".....0gHHHHHHHHHHHHHHHHHHHg0....",
    "....0HgHHHHHHHHHHHHHHHHHHHg0....",
    "....0ggggggggggggggggggggggg0...",
    ".....0000kssssssssssssl00000....",
    "........0kssssssssssssl0........",
    ".......0kksssssssssssskl0.......",
    ".......0kksssssssssssskl0.......",
    ".......0kksss0sssss0sskl0.......",
    "......0kkksssssssssssskkl0......",
    "......0kkkkssssstsssskkkl0......",
    "......0kkkksssssssssskkkl0......",
    "......0kkkkkssstttsskkkkl0......",
    ".......0kkkkksssssskkkkl0.......",
    "........000kkskkkkkkl000........",
    "...........00skkkkk00...........",
    ".........000iskkkkkh000.........",
    ".......00iiiiiiiiiiiiih00.......",
    "......0iiiiiigggggigiiiih0......",
    ".....0iiiiiiiiggggiiiiiiih0.....",
    "....0iiiiiiiiiggegiiiiiiiih0....",
    "...0iiiiiiiiigggggigiiiiiiih0...",
    "...0iiiiiiiiiiiiiiiiiiiiiiih0...",
    "..0iiiiiiiiiiiiiiiiiiiiiiiiih0..",
    "..0iiiiiiiiiiiiiiiiiiiiiiiiih0..",
]);

pub const TRADER_HERMIT: Sprite = Sprite(&[
    "................................",
    "................................",
    ".............000000.............",
    "...........00mmmmm100...........",
    "..........0mmmmmmmmm10..........",
    ".........0mmmmmmmmmmm10.........",
    "........0mmmmmmmmmmmmm10........",
    ".......0mmmmmmmmmmmmmmm10.......",
    ".......0mmmmmmmmmmmmmmm10.......",
    ".......0mmmmmmssssmmmmm10.......",
    "......0mmmmmssssssssmmmm10......",
    "......0mmmmmssssssssmmmm10......",
    "......0mmmmssssssssssmmm10......",
    "......0mmmmsfsfsssfsfmmm10......",
    "......0mmmmss0sfsfs0smmm10......",
    "......0mmmmssssssssssmmm10......",
    ".......0mmmssssstssssmm10.......",
    ".......0mmskkkkkkkkkksm10.......",
    ".......0mmmkkkkkkkkkkmm10.......",
    "........0mmkkkkkkkkkmm10........",
    ".........0mmkkkkkkkkm10.........",
    "..........0mkkkkkkkk10..........",
    "...........0kkkkkkkl0...........",
    ".........000kkkkkkkl000.........",
    ".......00mmmkkkkkkkmmm100.......",
    "......0mmmmmmkkkkkkmmmmm10......",
    ".....0mmmmmmmkkkkkkmmmmmm10.....",
    "....0mmmmmmmmkkkkkmmmmmmmm10....",
    "...0mmmmmmmmmmkkkmmmmmmmmmm10...",
    "...0mmmmmmmmmmmkmmmmmmmmmmm10...",
    "..0mmmmmmmmmmmmmmmmmmmmmmmmm10..",
    "..0mmmmmmmmmmmmmmmmmmmmmmmmm10..",
]);

pub const TRADER_PYREKIN: Sprite = Sprite(&[
    "...............0................",
    "..............0R0...............",
    "............000RR000............",
    "...........0FF0RR0FF0...........",
    ".........000FF0RR0FF000.........",
    "........0RR0FF0RR0FF0RR0........",
    "........0RR0FFRRR0FF0RR0........",
    "........0RRFFFRRRFFFFRR0........",
    ".......0Rxxxxxxxxxxxxxxx0.......",
    "........0xxxxxxxxxxxxxxx0.......",
    ".........0uuuuuuuuuuu200........",
    ".........0uuuuuuuuuuu20.........",
    ".........0uuuuuuuuuuu20.........",
    ".........0unnnnnnnnnnn0.........",
    ".........0uuu0uuuuu0u20.........",
    ".........0uuuuuuuuuuu20.........",
    ".........0uuuuuunuuuu20.........",
    ".........0uuuuuunuuuu20.........",
    "..........0uuuu222uu20..........",
    "...........0uuuuuuu20...........",
    "............0uuuuu20............",
    "............0FFFFFFF0...........",
    "............0FFFFFFF0...........",
    ".........000xuuuuuuR000.........",
    ".......00xxxxxxxxxxxxxR00.......",
    "......0xxxxxxxxxxxxxxxxxR0......",
    ".....0xxxxxxxxxxxxxxxxxxxR0.....",
    "....0xxxxxxxxxxxxxxxxxxxxxR0....",
    "...0xxxxxxxxxxxxxxxxxxxxxxxR0...",
    "...0xxxxxxxxxxxxxxxxxxxxxxxR0...",
    "..0xxxxxxxxxxxxxxxxxxxxxxxxxR0..",
    "..0xxxxxxxxxxxxxxxxxxxxxxxxxR0..",
]);

pub const TRADER_BONES: Sprite = Sprite(&[
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    "...........0000000000...........",
    "........000kkkkkkkkkl000........",
    "......00kkkkkkkkkkkkkkkl00......",
    ".....0kkkkkkkkkkkkkkkkkkkl0.....",
    "....0kkkkkkkkkkkkkkkkkkkkkl0....",
    "...0kkkkkkkkkkkkkkkkkkkkkkkl0...",
    "..0kkkkkkkkkkkkkkkkkkkkkkkkkl0..",
    "..0kkkkk1111kkkkkkkk1111kkkkl0..",
    ".0kkkkk111111kkkkkk111111kkkkl0.",
    ".0kkkkk111111kkkkkk111111kkkkl0.",
    ".0kkkkkk1111kkkkkkkk1111kkkkkl0.",
    ".0kkkkkkkllllllllllllllkkkkkkl0.",
    "..0kkllllllllllllllllllllllkl0..",
    "..0llkklkklkklkklkklkklkklkkl0..",
    ".0ll1kk1kk1kk1kk1kk1kk1kk1kk1l0.",
    "0lll1111111111111111111111111ll0",
    "0llllllllllllllllllllllllllllll0",
    ".0llllllllllllllllllllllllllll0.",
    "..0llll888888888888888888llll0..",
    "00888888888888888888888888888800",
    "88888888888888888888888888888888",
    "88888888888888888888888888888888",
    "00888888888888888888888888888800",
    "..0000088888888888888888800000..",
]);

// Faces traders make: the motive emotes, hex badges from the designs.

pub const EMOTE_BLISS: Sprite = Sprite(&[
    "....00000000....",
    "...0yyyyyyyy0...",
    "..0yyyyyyyyyy0..",
    "..0yyyyyyyyyy0..",
    ".0yywwwwwwwwyy0.",
    ".0yywwwwwwwwyy0.",
    "0yyywwwwwwwwyyy0",
    "0yyyywwwwwwyyyy0",
    "0yyyyywwwwyyyyy0",
    "0yyyyyywwyyyyyy0",
    ".0yyyyywwyyyyy0.",
    ".0yyyywwwwyyyy0.",
    "..0yyyyyyyyyy0..",
    "..0yyyyyyyyyy0..",
    "...0yyyyyyyy0...",
    "....00000000....",
]);

pub const EMOTE_REPOSE: Sprite = Sprite(&[
    "....00000000....",
    "...0zzzzzzzz0...",
    "..0zzzzzzzzzz0..",
    "..0zzzzwwzzzz0..",
    ".0zzzzwwwwzzzz0.",
    ".0zzzzwwwwzzzz0.",
    "0zzzwwzwwzwwzzz0",
    "0zzwwwwwwwwwwzz0",
    "0zzwwwwwwwwwwzz0",
    "0zzzwwzwwzwwzzz0",
    ".0zzzzzwwzzzzz0.",
    ".0zzzzwwwwzzzz0.",
    "..0zzzzzzzzzz0..",
    "..0zzzzzzzzzz0..",
    "...0zzzzzzzz0...",
    "....00000000....",
]);

pub const EMOTE_LOVE: Sprite = Sprite(&[
    "....00000000....",
    "...0LLLLLLLL0...",
    "..0LLLLLLLLLL0..",
    "..0LLLLLLLLLL0..",
    ".0LLwwwLLwwwLL0.",
    ".0LwwwwwwwwwwL0.",
    "0LLwwwwwwwwwwLL0",
    "0LLwwwwwwwwwwLL0",
    "0LLLwwwwwwwwLLL0",
    "0LLLLwwwwwwLLLL0",
    ".0LLLLwwwwLLLL0.",
    ".0LLLLLwwLLLLL0.",
    "..0LLLLLLLLLL0..",
    "..0LLLLLLLLLL0..",
    "...0LLLLLLLL0...",
    "....00000000....",
]);

pub const EMOTE_PAIN: Sprite = Sprite(&[
    "....00000000....",
    "...0PPPPPPPP0...",
    "..0PPPPPPPPPP0..",
    "..0PPPPwPPPPP0..",
    ".0PPPPPwPPPPPP0.",
    ".0PPPPwwwPPPPP0.",
    "0PPPPPwwwPPPPPP0",
    "0PPPPPwwwPPPPPP0",
    "0PPPPPwwwPPPPPP0",
    "0PPPPwwwwwPPPPP0",
    ".0PPPPwwwPPPPP0.",
    ".0PPPPPwPPPPPP0.",
    "..0PPPPPPPPPP0..",
    "..0PPPPPPPPPP0..",
    "...0PPPPPPPP0...",
    "....00000000....",
]);

pub const EMOTE_ZEAL: Sprite = Sprite(&[
    "....00000000....",
    "...0xxxxxxxx0...",
    "..0xxxxxxxxxx0..",
    "..0xxxxwwxxxx0..",
    ".0xxxxwwwwxxxx0.",
    ".0xxxwwwwwwxxx0.",
    "0xxxwwwwwwwwxxx0",
    "0xxwwwwwwwwwwxx0",
    "0xxxwwwwwwwwxxx0",
    "0xxxxwwwwwwxxxx0",
    ".0xxxxwwwwxxxx0.",
    ".0xxxxxwwxxxxx0.",
    "..0xxxxxxxxxx0..",
    "..0xxxxxxxxxx0..",
    "...0xxxxxxxx0...",
    "....00000000....",
]);

pub const EMOTE_HATE: Sprite = Sprite(&[
    "....00000000....",
    "...0HHHHHHHH0...",
    "..0HHHHHHHHHH0..",
    "..0HHHHnnHHHH0..",
    ".0HHHHnnnnHHHH0.",
    ".0HHHnnnnnnHHH0.",
    "0HHHnnnnnnnnHHH0",
    "0HHnnnnnnnnnnHH0",
    "0HHnnnnnnnnnnHH0",
    "0HHHnnnHHnnnHHH0",
    ".0HHHHHnnHHHHH0.",
    ".0HHHHnnnnHHHH0.",
    "..0HHHHHHHHHH0..",
    "..0HHHHHHHHHH0..",
    "...0HHHHHHHH0...",
    "....00000000....",
]);

pub const EMOTE_ANIMA: Sprite = Sprite(&[
    "....00000000....",
    "...0qqqqqqqq0...",
    "..0qqq0000qqq0..",
    "..0q00000000q0..",
    ".0q000EEEE000q0.",
    ".0q00E0000E00q0.",
    "0q00E00jj00E00q0",
    "0q00E0jjjj0E00q0",
    "0q00E0jjjj0E00q0",
    "0q00E00jj00E00q0",
    ".0q00E0000E00q0.",
    ".0q000EEEE000q0.",
    "..0q00000000q0..",
    "..0qqq0000qqq0..",
    "...0qqqqqqqq0...",
    "....00000000....",
]);

// Things in camp, the interface, ships and history.

pub const WAYSTONE: Sprite = Sprite(&[
    "................",
    ".......0........",
    "......0m0.......",
    ".....0nnm0......",
    "....0nnnnm0.....",
    "...0nnnmnnm0....",
    "...0mnnnnnnm0...",
    "...0nnnnnnnm0...",
    "...0n111111m0...",
    "...0n111111m0...",
    "...0n111111m0...",
    "...0n111111m0...",
    "...0n111111m0...",
    "...0n111111m0...",
    "...0nnnnnnnm0...",
    "...0nnnnnnmm0...",
    "...0nnnmnnnm0...",
    "...0mnnnmmmm0...",
    "..0nnmmmmnnm0...",
    "..0nnnnnnmnnm0..",
    "..0nnnmnnnnnm0..",
    "..0mnnnnnnnnm0..",
    "..0nnnnnnnnmm0..",
    "..0nnnnnmnnnm0..",
    "..0nnmmmmnnnm0..",
    "..0nnnnnmmmnm0..",
    "..0nnnnnnnmnm0..",
    "..0nnnnmnnnnm0..",
    "..0nmnnnnnnnm0..",
    ".00999999999900.",
    "0999999999999990",
    "0999999999999990",
]);

pub const WELL: Sprite = Sprite(&[
    "................................",
    "................................",
    "................................",
    "................................",
    "....000000000000000000000000....",
    "...03333333333333333333333330...",
    "...03333333333333333333333330...",
    "....043000000000l00000000430....",
    "....0430.......0l0......0430....",
    "....0430.......0l0......0430....",
    "....0430.......0l0......0430....",
    "....0430.......0l0......0430....",
    "....0430.......0l0......0430....",
    "....0430.......0l0......0430....",
    "....0430......00l00.....0430....",
    "....0430.....0aaaaa0....0430....",
    "....0430.....0999990....0430....",
    "....0430...0009999900...0430....",
    "....0430000nnn99999nn0000430....",
    "....0430nnBBBB99999BBBnn0430....",
    "....044nBBBrrrrrrrrrrBBBn430....",
    "....044BrrrrrrrrrrrrrrrrB430....",
    "...0n44rrrrrrrrrrrrrrrrrr44n0...",
    "...0n44rrrrrrrrrrrrrrrrrr44n0...",
    "...0n44nnnnnnnnnnnnnnnnnn44nn0..",
    "...0nmnnnmmmmmmmmmmmmmmnnmnnn0..",
    "...0nmmmmmmmmmmmmmmmmmmmmmnnn0..",
    "...0mmmmmmmmmmmmmmmmmmmmmmmmn0..",
    "...0mmmmmmmmmmmmmmmmmmmmmmmmn0..",
    "....0mmmmmmmmmmmmmmmmmmmmm000...",
    ".....0000mmmmmmmmmmmmmm000......",
    ".........00000000000000.........",
]);

pub const PYRE: Sprite = Sprite(&[
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    "..........0000000000000.........",
    ".........055444444444550........",
    "........05555444444455550.......",
    ".........055444444444550........",
    "........00044444444444000.......",
    ".......0553333333333333550......",
    "......055553333333333355550.....",
    ".......0553311111111133550......",
    "......000333111111111333000.....",
    ".....05544441111111114444550....",
    "....0555544411111111144455550...",
    ".....05544441111111114444550....",
    "....0004444411111111144444000...",
    "...055333333111111111333333550..",
    "..05555333331111111113333355550.",
    "...055333333111111111333333550..",
    "..0mm33333333333333333333333m0..",
    "..0mmmmmmmmmmmmmmmmmmmmmmmmmm0..",
    "...0mmmmmmmmmmmmmmmmmmmmmmmm0...",
    "....000mmmmmmmmmmmmmmmmmm000....",
    ".......000000000000000000.......",
]);

pub const BENCH: Sprite = Sprite(&[
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    "................000000..........",
    "............0000cc33330000......",
    "..........00cccccc33330ccc00....",
    ".........0cccccccc33334444440...",
    "........0ccccccccc33334444440...",
    "..0000000000000000333344444400..",
    ".055555555555555555555555555540.",
    ".055555555555555555555555555540.",
    ".055555555555555555555555555540.",
    ".055555555555555555555555555540.",
    ".033333333333333333333333333330.",
    "..0044400000000000000000044400..",
    "...04440................04440...",
    "...04440000000000000000004440...",
    "...04433333333333333333333440...",
    "...04440000000000000000004440...",
    "...04440................04440...",
    "...04440................04440...",
    "...04440................04440...",
    "...04440................04440...",
    "...04440................04440...",
    "...04440................04440...",
    "...04440................04440...",
    "....000..................000....",
]);

pub const LOUPE: Sprite = Sprite(&[
    "......0.........",
    "...000f000......",
    "..0fffffff0.....",
    ".0fffGGGfff0....",
    ".0ffGwGGGff0....",
    ".0fGwGGGGGf0....",
    "0ffGGGGGGGff0...",
    ".0fGGGGGGGf0....",
    ".0ffGGGGGff0....",
    ".0fffGGGfff0....",
    "..0fffffff340...",
    "...000f0003340..",
    "......0...03340.",
    "...........03340",
    "............0330",
    ".............00.",
]);

pub const HANDS: Sprite = Sprite(&[
    "................",
    "................",
    "................",
    "................",
    ".....00.00......",
    "..000tt0ss000...",
    "00tttttsussss000",
    "iittttsususssszz",
    "iitttttsusussszz",
    "iit00000susssszz",
    "ii0.....000000zz",
    "00............00",
    "................",
    "................",
    "................",
    "................",
]);

pub const SAIL: Sprite = Sprite(&[
    ".......0........",
    "......030.......",
    "......030.......",
    "......03k0......",
    ".....003kk0.....",
    "....0l03kkk0....",
    "...0ll03kkk0....",
    "...0ll03kkkk0...",
    "..0lll03kkkkk0..",
    ".0llll03kkkkkk0.",
    "..000003kkk000..",
    "..000003000000..",
    ".04444444444440.",
    "..044444444440..",
    "...0444444440...",
    "....00000000....",
]);

pub const CURSOR: Sprite = Sprite(&[
    "....000.........",
    "...0kkl0........",
    "...0kkl0........",
    "...0kkl0........",
    "...0kkl0000.....",
    "...0kkkkkkl00...",
    "...0kkkkkkkkl0..",
    "...0kkkkkkkkl0..",
    "..0kkkkkkkkkl0..",
    "..0kkkkkkkkkl0..",
    "..0kkkkkkkkkl0..",
    "..0kkkkkkkkkl0..",
    "...00kkkkkkkl0..",
    "....0kkkkkkl0...",
    ".....0000000....",
    "................",
]);

pub const GRAB: Sprite = Sprite(&[
    "................",
    "................",
    "................",
    ".......00.00....",
    "....000kl0kl0...",
    "...0kl0kl0kl0...",
    "...0kkkkkkkkl0..",
    "...0kkkkkkkkl0..",
    "..0kkkkkkkkkl0..",
    "..0kkkkkkkkkl0..",
    "..0kkkkkkkkkl0..",
    "..0kkkkkkkkkl0..",
    "...0kkkkkkkkl0..",
    "....0kkkkkkl0...",
    ".....0000000....",
    "................",
]);

pub const SPEAKER: Sprite = Sprite(&[
    "................",
    ".........0......",
    "........0l0.....",
    ".......0kl0.....",
    "......0kkl0.....",
    "..0000kkkl0.....",
    ".0kkkkkkkl0.....",
    ".0kkkkkkkl0.....",
    ".0kkkkkkkl0.....",
    ".0kkkkkkkl0.....",
    ".0kkkkkkkl0.....",
    "..00000kkl0.....",
    ".......0kl0.....",
    "........0l0.....",
    ".........0......",
    "................",
]);

pub const SHIP: Sprite = Sprite(&[
    "......0zz0......",
    "......030.......",
    "......03k0......",
    ".....003kk0.....",
    "....0l03kkk0....",
    "....0l03kkj0....",
    "...0ll03kkkj0...",
    "..0lll03kkkkk0..",
    "..0lll03kkkkkk0.",
    ".0llll03kkk000..",
    ".00000030000000.",
    "0444444344444430",
    ".04444444444430.",
    "..044444444430..",
    "..000000000000..",
    ".0440......0440.",
]);

pub const FRIGATE: Sprite = Sprite(&[
    "......0gg0......",
    "......030.......",
    "......03i0......",
    ".....003ii0.....",
    "....0h03iii0....",
    "....0h03iii0....",
    "...0hh03iiii0...",
    "..0hhh03iiiii0..",
    "..0hhh03iiiiii0.",
    ".0hhhh03iii000..",
    ".00000030000000.",
    "0222222322222210",
    ".02222222222210.",
    "..022222222210..",
    "..000000000000..",
    ".0440......0440.",
]);

pub const DIRIGIBLE: Sprite = Sprite(&[
    "..........000000000000..........",
    "......0000iiiiiiiiiiih0000......",
    "....00iiiiiiiiiiiiiiiiiiih00..0.",
    "...0iiijjjjjjjjjjiiiiiiiiiih00h0",
    "..0iiiijjjjjjjjjjiiiiiiiiiiihhh0",
    ".0iiiiiiiiiiiiiiiiiiiiiiiiiihhh0",
    ".0iihhhhhhhhhhhhhhhhhhhhhhhhhhh0",
    "..0iiiiiiiiiiiiiiiiiiiiiiiiihhh0",
    "...0iiiiiiiiiiiiiiiiiiiiiiih00h0",
    "....00iiiiiiiiiiiiiiiiiiih00..0.",
    "......0000iimiiiiiiimh0000......",
    ".........000m0000000m0..........",
    "........0n00m3333333m0..........",
    ".........0n03333333330..........",
    "........0n003333333330..........",
    ".........0..000000000...........",
]);

pub const TOWER: Sprite = Sprite(&[
    ".......00.......",
    "......0330......",
    "......0330......",
    "......0330......",
    "......0330......",
    "......0430......",
    "......0430......",
    ".....044430.....",
    ".....044430.....",
    ".....044430.....",
    ".....044430.....",
    ".....044430.....",
    "....04444430....",
    "....04444430....",
    "....04444430....",
    ".....000000.....",
]);

// Places, as they stand on the map.

pub const PLACE_TETHER: Sprite = Sprite(&[
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    "....................00..........",
    "...................0qp0.........",
    ".........00.....0.0qqpp0........",
    "........0kl0...030qqqpp0........",
    ".......0kkll0..03kqq11pp0.......",
    "......0kkkll0.003kkq11ppp0......",
    "....0.0kk11ll0l03kkq11pppp00....",
    "...030kkk11llll03kkk000000030...",
    "...03kkkk11llll03kkkk00...030...",
    "...030000999lll93kk9999000030...",
    "..0039999999999939999999999300..",
    ".099399999944444344449999993990.",
    "09993999999944444444999999939990",
    "09999999999999999999999999999990",
    ".099999999999999999999999999990.",
    "..0099999999999999999999999900..",
    "....000009999999999999900000....",
]);

pub const PLACE_LIGHTHOUSE: Sprite = Sprite(&[
    "..............0hh0..............",
    ".............0hhhh0.............",
    "............0hhhhhh0............",
    "............0fffffff0...........",
    "............0fFFFFFf0...........",
    "............0fFFFFFf0...........",
    "............0fffffff0...........",
    ".............0kkkl00............",
    "............00kkkl000...........",
    "...........0iiiiiiiii0..........",
    "...........0iiiiiiiii0..........",
    "............0kkkkkl00...........",
    "............0kkkkkl0............",
    "............0kkkkkl00...........",
    "...........0iiiiiiiii0..........",
    "...........0iiiiiiiii0..........",
    "...........0kkkkkkkl0...........",
    "......000000kkkkkkkl0000000.....",
    ".....0ppppppkkkkkkkkppppppp0....",
    ".....0ppppppppppppppppppppp0....",
    ".....0ppppppppppppppppppppp0....",
    "....0aaaaaaaaaaaaaaaaaaaaa20....",
    "....0aaaaaaaaaaaaaaaaaaaaa20....",
    "....0aaaaaaaaaaaaaaaaaaaaa20....",
    "...0aaaaaaaaaaaaaaaaaaaaaaa20...",
    "...0aaaaaaaaaaaaaaaaaaaaaaa20...",
    "...0aaaaaaaaaaaaaaaaaaaaaaa20...",
    "..0aaaaaaaaaaaaaaaaaaaaaaaaa20..",
    "..0aaaaaaaaaaaaaaaaaaaaaaaaa20..",
    "..0aaaaaaaaaaaaaaaaaaaaaaaaa20..",
    ".0aaaaaaaaaaaaaaaaaaaaaaaaaaa20.",
    "..0000000000000000000000000000..",
]);

pub const PLACE_HEARTHRING: Sprite = Sprite(&[
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    "...............00...............",
    "..............0kl0..............",
    ".............0kkll0.............",
    ".......00...0kkkll0....00.......",
    "......0kl0..0kk11ll0..0qp0......",
    ".....0kkll00kkk11lll00qqpp0.....",
    "....0kkkll0kkkk11llllqqqpp0.....",
    "....0kk11ll0000000000qq11pp0....",
    "...0kkk11lll0......0qqq11ppp0...",
    "..0kkkk11llll0....0qqqq11pppp0..",
    "...0000000000......0000000000...",
    ".....00..................00.....",
    "....0ji0.......00.......0yf0....",
    "...0jjii0000000RF0000000yyff0...",
    "..0jjjii099999RRRR99999yyyff0...",
    "..0jj11ii99999RRRR99999yy11ff0..",
    ".0jjj11iii99999RR99999yyy11fff0.",
    "0jjjj11iiii9999999999yyyy11ffff0",
    "09999999999999999999999999999990",
    ".099999999999999999999999999990.",
    "..0099999999999999999999999900..",
    "....000009999999999999900000....",
]);

pub const PLACE_DRYDOCK: Sprite = Sprite(&[
    "................................",
    "................................",
    "................................",
    ".........0......................",
    "........0200....................",
    "........02220...................",
    "........020020..................",
    "........020.0200......00000000..",
    "........020..0220....0444444440.",
    "........020...0020...0444444440.",
    "........020.....0.....00004400..",
    "........020..............0440...",
    "........020..............0440...",
    "........020..............0440...",
    "........020..............0440...",
    "........020............0004400..",
    "......0.02000000000000033344320.",
    "..00002002233323336666666623320.",
    ".03333233323332333666666662320..",
    "..0333233323332333666666662320..",
    "..0333233323332333666666662320..",
    "...03323332333233366666666220...",
    "...03322332233223366666666220...",
    "...03332333233323332333233320...",
    "....0332333233323332333233320...",
    "..0093323332333233323332333200..",
    ".099993233323332333233323332990.",
    "09999999999999999999999999999990",
    "09999999999999999999999999999990",
    ".099999999999999999999999999990.",
    "..0099999999999999999999999900..",
    "....000009999999999999900000....",
]);

pub const PLACE_SALTFLATS: Sprite = Sprite(&[
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    ".......00.......................",
    "......0330......................",
    ".....033330.....................",
    "....03333330....................",
    "...0333333330...................",
    "..033333333330..................",
    "...055555555550.................",
    "...055555555550........00.......",
    "...055555555550.......0Sn0......",
    "...055555555550.......0Sn0......",
    "...055511155550......0SSSn0.....",
    "...055511155550.....0SSSSn0.....",
    "...055511155550000000SSSSSn0....",
    "...05551115555SSSSSSSSSSSSSn0...",
    "...05551115555SSSSSSSSSSSSSn0...",
    "..00nSSSSSnSSSSSnSSSSSSSSSSSn0..",
    ".0SSnSSSSSnSSSSSnSSSSSnSSSSSnn0.",
    "0SSSSnSSSSSnSSSSSnSSSSSnSSSSSnn0",
    "0SSSSnSSSSSnSSSSSnSSSSSnSSSSSnn0",
    "0SSSSnnSSSSnnSSSSnnSSSSnnSSSSnn0",
    "0SSSSSnSSSSSnSSSSSnSSSSSnSSSSSn0",
    ".0SSSSnSSSSSnSSSSSnSSSSSnSSSSSn0",
    "..00SSSnSSSSSnSSSSSnSSSSSnSn000n",
    "....00SnSSSSSnSSSSSnSSSSSn00..0n",
    "......0000SSSSSSSSSSSn0000.....0",
    "..........000000000000..........",
]);

pub const PLACE_REEF: Sprite = Sprite(&[
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    ".........00000..................",
    "........0jjjji0.................",
    ".......0jjjjji0.................",
    ".......0jjjkji0.........00000...",
    ".......0jjjjji0........0jjjji0..",
    ".......0jjjjji0.......0jjjjji0..",
    "...00000jjjjji0.......0jjjkji0..",
    "..0jjjjijjkjji0.......0jjjjji0..",
    ".0jjjjjijjjjji0.......0jjjjji0..",
    ".0jjjkjijjjjji0.......0jjjjji0..",
    ".0jjjjji0jjjji0....0..0jjkjji0..",
    ".0jjjjji0jjjji0...030.0jjjjji0..",
    ".0jjjjji0jjjji0...03H00jjjjji0..",
    ".0jjkjji0jjjji0..003HH00jjjji0..",
    ".0jjjjji0jjjji0.0103HH00jjjji0..",
    ".0jjjjji0jjjji001103HHH0jjjji0..",
    "..0jjjji0jjjji001103HHHHjjjji0..",
    "..0jjjji0jjjjja111a3HHa0jjjji0..",
    "..0jjjjjajjjjjaaaaa3aaaajjjji0..",
    ".0ajjjjjajjjjj2222232222jjjjja0.",
    "0aajjjjjajjjjja22222222ajjjjjaa0",
    "0aajjjjjajjjjjaaaaaaaaaajjjjjaa0",
    ".0aaaaaaaaaaaaaaaaaaaaaaaaaaaa0.",
    "..00aaaaaaaaaaaaaaaaaaaaaaaa00..",
    "....00000aaaaaaaaaaaaaa00000....",
]);

pub const PLACE_LEVIATHAN: Sprite = Sprite(&[
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    ".............000................",
    "............0kkk0.000...........",
    "........0000kk0kk0kkk0..........",
    ".......0kkk0kk0kkkk0kk0.00......",
    "...0000kk0kk00.00kk0kk00kk0.....",
    "..0kkk0kk0kkk0.0kk0.00kkkkk0....",
    ".0kk0kkk0.0kk0.0kk0.0kkk0kk0....",
    ".0kk0kk0..0kk0.0kk0.0kk0.0kk0...",
    "0kk0.0kk0.0kk0..00..0kk0.0kk0...",
    "0kk0.0kk0.0kk0.0kk0.0kk0..00....",
    "0kk0.0kk0.0kk0..00..0kk0.0kk0...",
    ".00..0kk000kk000kk000kk0..00....",
    "0kk000kk0999999999999kk000kk00..",
    ".000999999999999llllllllllllll0.",
    "0kkllllllllllllllkk99kkk999kk90.",
    "09999999999999999999999999999990",
    "09999999999999999999999999999990",
    ".099999999999999999999999999990.",
    "..0099999999999999999999999900..",
    "....000009999999999999900000....",
]);

pub const PLACE_FORT: Sprite = Sprite(&[
    "................................",
    "................00000000........",
    "...............03iiiiiii0.......",
    "...............03iiiiiii0.......",
    "...............03iiigiii0.......",
    "...............03iiiiiii0.......",
    "...............03iiiiiii0.......",
    "...............030000000........",
    "...............030..............",
    ".........0..0..0300..0..........",
    "........0m00m00m30m00m00........",
    "........0mmmmmmm3mmmmmmm0.......",
    "........0nnnnnnnnnnnnnnm0.......",
    "........0nnnnnnnnnnnnnnm0.......",
    "........0nnnnnnnnnnnnnnm0.......",
    "........0nnnnnnnnnnnnnnm0.......",
    "........0nnnnnnnnnnnnnnm0.......",
    "........0nnnnnnnnnnnnnnm0.......",
    ".......0nnnnnnnnnnnnnnnm0.......",
    "......0nnnnnnnnnnnnnnnnnm0......",
    ".....0nnnnnnnnnnnnnnnnnnnm0.....",
    ".....0nnnnnnnnnnnnnnnnnnnm0.....",
    "....0nnnnnnnnn11111nnnnnnnm0....",
    "...0nnnnnnnnnn11111nnnnnnnnm0...",
    "...0nnnnnnnnnn11111nnnnnnnnm0...",
    "..0nnnnnnnnnnn11111nnnnnnnnnm0..",
    ".0nnnnnnnnnnnn11111nnnnnnnnnnm0.",
    "0aaaaaaaaaaaaa11111aaaaaaaaaaaa0",
    "0aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa0",
    ".0aaaaaaaaaaaaaaaaaaaaaaaaaaaa0.",
    "..00aaaaaaaaaaaaaaaaaaaaaaaa00..",
    "....00000aaaaaaaaaaaaaa00000....",
]);

pub const PLACE_ASHFLEET: Sprite = Sprite(&[
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    "...............00...............",
    "..............0RR0..............",
    "..............0RR0..............",
    "..............0RR0..............",
    ".............0RFFR0.............",
    ".............0RFFR0.............",
    ".............0RFFR0.............",
    "............0RFFFFR0............",
    ".......0....0RFFFFR0.....0......",
    "......030....003300.....030.....",
    "......03x0....0330......03x0....",
    ".....003xx0..03330.....003xx0...",
    "....0R03xx0..033330...0R03xx0...",
    "...0RR03xxx0.033330..0RR03xxx0..",
    "...0RR03xxxx0333330000RR03xxxx0.",
    "..0RRR03xxaaa333333aaRRR03xx00..",
    "..00aaa3aaaaa333333aaaaaa3aa00..",
    ".044444344443333333a44444344440.",
    "0aa44444444a33333333a44444444aa0",
    "0aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa0",
    ".0aaaaaaaaaaaaaaaaaaaaaaaaaaaa0.",
    "..00aaaaaaaaaaaaaaaaaaaaaaaa00..",
    "....00000aaaaaaaaaaaaaa00000....",
]);

pub const PLACE_SENATE: Sprite = Sprite(&[
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    ".............000000.............",
    "..........000nnnnnm000..........",
    ".........0nnnn1nnnnnnm0.........",
    "........0nnnnnn1nnnnnnm0........",
    ".......0nnnnnnn1nnnnnnnm0.......",
    "......0nnnnnnnnn11nnnnnnm0......",
    "......0nnnnnnnnnn1nnnnnnm0......",
    ".....0nnnnnnnnnnnn1nnnnnnm0.....",
    ".....0kkkkkkkkkkkkkkkkkkkkk0....",
    ".....0kkkkkkkkkkkkkkkkkkkkk0....",
    ".....0nnmkkmmkkmmkkmmkkmmnm0....",
    ".....0nnmkkmmkkmmkkmmkkmmnm0....",
    ".....0nnmkkmmkkmmkkmmkkmmnm0....",
    ".....0nnmkkmmkkmmkkmmkkmmnm0....",
    ".....0nnmkkmmkkmmkkmmkkmmnm0....",
    ".....0nnmkkmmkkmmkkmmkkmmnm00...",
    ".....0nnmkkmmkkmmkkmmkk8888880..",
    ".....0nnmkkmmkkmmkkmmk888888880.",
    "....00nn888888888888888888888880",
    ".0008888888888888888888888888880",
    "08888888888888888888888888888880",
    "88888888888888888888888888888888",
    "88888888888888888888888888888888",
    "08888888888888888888888888888880",
    ".000888888888888888888888888000.",
]);

pub const PLACE_WELL: Sprite = Sprite(&[
    "................................",
    "................................",
    "................................",
    "................................",
    "................................",
    ".............0000000............",
    "...........00kkkkkkk00..........",
    ".........00kkkkkkkkkkk00........",
    "........0kkkkkkkkkkkkkkk0.......",
    ".......0kkkkkkkl00lkkkkkk0......",
    ".......0kkkkl00l00l00lkkk0......",
    "......0kkkk0l0.0..0.0lkkkk0.....",
    ".....0kkkl0.0........00kkkk0....",
    ".....0kkkl0...........0kkkk0....",
    "....0kkkk0.............0lkkk0...",
    "....0kkk0..............0lkkk0...",
    "....0kkk0...............0kkk0...",
    "....0klk0...............0kkk0...",
    "...0kklk0...............0kkkk0..",
    "...0kkk0.................0kkk0..",
    "...0kkk0.................0kkk0..",
    "...0kkk0.....000000......0kkk0..",
    "..0kkkk0...00nnnnnn00....0kkkk0.",
    "..0kkk0..00nBBBBBBBBn00...0kkk0.",
    "..0kkk0009nnBBBBBBBBnn90000kkk0.",
    "..0kkk9999nnnnnnnnnnnn99999kkk0.",
    ".09kkk99999nnnnnnnnnn999999kkk0.",
    "099kkk9999999nnnnnn99999999kkk90",
    "099kkk999999999999999999999kkk90",
    ".099999999999999999999999999990.",
    "..0099999999999999999999999900..",
    "....000009999999999999900000....",
]);

/// A motive's mark.
#[must_use]
pub const fn mark(motive: Motive) -> Sprite {
    match motive {
        Motive::Bliss => MARK_BLISS,
        Motive::Repose => MARK_REPOSE,
        Motive::Love => MARK_LOVE,
        Motive::Pain => MARK_PAIN,
        Motive::Zeal => MARK_ZEAL,
        Motive::Hate => MARK_HATE,
    }
}

/// A face, as an emote badge.
#[must_use]
pub const fn emote(emote: Emote) -> Sprite {
    match emote {
        Emote::Love => EMOTE_LOVE,
        Emote::Bliss => EMOTE_BLISS,
        Emote::Repose => EMOTE_REPOSE,
        Emote::Zeal => EMOTE_ZEAL,
        Emote::Hate => EMOTE_HATE,
        Emote::Pain => EMOTE_PAIN,
        Emote::Anima => EMOTE_ANIMA,
    }
}

/// Who stands over a culture's rug.
#[must_use]
pub const fn trader(culture: Culture) -> Sprite {
    match culture {
        Culture::Drifter => TRADER_DRIFTER,
        Culture::Keeper => TRADER_KEEPER,
        Culture::Thesean => TRADER_THESEAN,
        Culture::Scraper => TRADER_SCRAPER,
        Culture::Pirate => TRADER_PIRATE,
        Culture::Harbor => TRADER_HARBOR,
        Culture::Hermit => TRADER_HERMIT,
        Culture::PyreKin => TRADER_PYREKIN,
        Culture::Bones => TRADER_BONES,
    }
}

/// A place on the map.
#[must_use]
pub const fn place(site: Site) -> Sprite {
    match site {
        Site::Tether => PLACE_TETHER,
        Site::Lighthouse => PLACE_LIGHTHOUSE,
        Site::Hearthring => PLACE_HEARTHRING,
        Site::Drydock => PLACE_DRYDOCK,
        Site::SaltFlats => PLACE_SALTFLATS,
        Site::SpadeReef => PLACE_REEF,
        Site::Leviathan => PLACE_LEVIATHAN,
        Site::Fort => PLACE_FORT,
        Site::Ashfleet => PLACE_ASHFLEET,
        Site::Senate => PLACE_SENATE,
        Site::WellOfTeeth => PLACE_WELL,
    }
}

/// Every sprite that is not an item, for the tests and for baking.
pub const OTHERS: [Sprite; 45] = [
    MARK_BLISS,
    MARK_REPOSE,
    MARK_LOVE,
    MARK_PAIN,
    MARK_ZEAL,
    MARK_HATE,
    MARK_ANIMA,
    TRADER_DRIFTER,
    TRADER_KEEPER,
    TRADER_THESEAN,
    TRADER_SCRAPER,
    TRADER_PIRATE,
    TRADER_HARBOR,
    TRADER_HERMIT,
    TRADER_PYREKIN,
    TRADER_BONES,
    EMOTE_BLISS,
    EMOTE_REPOSE,
    EMOTE_LOVE,
    EMOTE_PAIN,
    EMOTE_ZEAL,
    EMOTE_HATE,
    EMOTE_ANIMA,
    WAYSTONE,
    WELL,
    PYRE,
    BENCH,
    LOUPE,
    HANDS,
    SAIL,
    CURSOR,
    GRAB,
    SPEAKER,
    SHIP,
    FRIGATE,
    DIRIGIBLE,
    TOWER,
    PLACE_TETHER,
    PLACE_LIGHTHOUSE,
    PLACE_HEARTHRING,
    PLACE_DRYDOCK,
    PLACE_SALTFLATS,
    PLACE_REEF,
    PLACE_LEVIATHAN,
    PLACE_FORT,
];

#[cfg(test)]
mod tests {
    use super::*;

    fn every_sprite() -> Vec<Sprite> {
        let mut all: Vec<Sprite> = Kind::ALL.iter().map(|&k| item(k)).collect();
        all.extend(OTHERS);
        all.extend([PLACE_ASHFLEET, PLACE_SENATE, PLACE_WELL]);
        all
    }

    #[test]
    fn every_sprite_is_a_rectangle_on_the_grid_in_the_palette() {
        for sprite in every_sprite() {
            let w = sprite.width();
            assert!(
                w > 0 && w % 8 == 0 && sprite.height() % 8 == 0,
                "{:?}",
                sprite.0[0]
            );
            for row in sprite.0 {
                assert_eq!(row.len(), w, "ragged row {row:?}");
                for ch in row.chars() {
                    assert!(
                        ch == '.' || colour(ch).is_some(),
                        "{ch:?} is not in the palette"
                    );
                }
            }
            assert_eq!(sprite.rgba().len(), w * sprite.height() * 4);
        }
    }

    #[test]
    fn items_are_drawn_to_their_footprint() {
        for kind in Kind::ALL {
            let (w, h) = kind.size();
            let sprite = item(kind);
            assert_eq!(sprite.width(), usize::from(w) * 16, "{kind:?}");
            assert_eq!(sprite.height(), usize::from(h) * 16, "{kind:?}");
        }
    }

    #[test]
    fn the_palette_has_no_duplicates() {
        for (i, (a, _)) in PALETTE.iter().enumerate() {
            for (b, _) in &PALETTE[i + 1..] {
                assert_ne!(a, b);
            }
        }
    }
}
