#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) enum Personality {
    Blinky,
    Pinky,
    Clyde,
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum MazeType {
    Wall,
    Food,
    PowerPellet,
    Empty,
    Pacman,
    Ghost(Personality),
    VulnerableGhost,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct GamepadState {
    pub a: bool,
    pub b: bool,
    pub x: bool,
    pub y: bool,
    pub start: bool,
    pub select: bool,
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub lb: bool,
    pub rb: bool,
    pub lstick: bool,
    pub rstick: bool,
    pub lt: f32,
    pub rt: f32,
    pub lstick_x: f32,
    pub lstick_y: f32,
    pub rstick_x: f32,
    pub rstick_y: f32,
}

pub(crate) struct Ghost {
    pub(crate) pos: (usize, usize),
    pub(crate) last_pos: (usize, usize),
    pub(crate) under_tile: MazeType, // The tile where the ghost is hovering above
    pub(crate) personality: Personality,
    pub(crate) home: (usize, usize),
}

impl Personality {
    pub(crate) fn icon(&self) -> &'static str {
        match self {
            Personality::Blinky => "\x1B[38;2;253;0;0m󰊠\x1B[0m", // Red
            Personality::Pinky => "\x1B[38;5;213m󰊠\x1B[0m",      // Pink
            Personality::Clyde => "\x1B[38;5;214m󰊠\x1B[0m",      // Orange
        }
    }
}

impl MazeType {
    pub(crate) fn icon(&self) -> &'static str {
        match self {
            MazeType::Wall => "\x1B[38;5;75m▓\x1B[0m",
            MazeType::Food => "\x1B[38;2;222;161;133m•\x1B[0m",
            MazeType::PowerPellet => "\x1B[38;2;222;161;133m\x1B[0m",
            MazeType::Empty => " ",
            MazeType::Pacman => "\x1B[38;2;101;79;240m\x1B[0m", // Wasm Purple
            MazeType::Ghost(personality) => personality.icon(),
            MazeType::VulnerableGhost => "\x1B[38;2;33;33;222m󰊠\x1B[0m", // Blue,
        }
    }
}

impl From<Personality> for MazeType {
    fn from(personality: Personality) -> Self {
        MazeType::Ghost(personality)
    }
}

use MazeType::Empty;
use MazeType::Food;
use MazeType::Pacman;
use MazeType::PowerPellet;
use MazeType::Wall;

pub(crate) const DEFAULT_MAZE: [[MazeType; 30]; 14] = [
    [Wall; 30],
    [
        Wall, Food, Food, Food, Food, Food, Food, Food, Food, Food, Food, Food, Food, Food, Wall,
        Wall, Food, Food, Food, Food, Food, Food, Food, Food, Food, Food, Food, Food, Food, Wall,
    ],
    [
        Wall,
        PowerPellet,
        Wall,
        Wall,
        Wall,
        Wall,
        Food,
        Wall,
        Wall,
        Wall,
        Wall,
        Wall,
        Wall,
        Food,
        Wall,
        Wall,
        Food,
        Wall,
        Wall,
        Wall,
        Wall,
        Wall,
        Wall,
        Food,
        Wall,
        Wall,
        Wall,
        Wall,
        PowerPellet,
        Wall,
    ],
    [
        Wall, Food, Food, Food, Food, Food, Food, Food, Food, Food, Food, Food, Food, Food, Food,
        Food, Food, Food, Food, Food, Food, Food, Food, Food, Food, Food, Food, Food, Food, Wall,
    ],
    [
        Wall, Food, Wall, Wall, Wall, Wall, Food, Wall, Wall, Food, Wall, Wall, Wall, Wall, Wall,
        Wall, Wall, Wall, Wall, Wall, Food, Wall, Wall, Food, Wall, Wall, Wall, Wall, Food, Wall,
    ],
    [
        Wall, Food, Food, Food, Food, Food, Food, Wall, Wall, Food, Food, Food, Food, Food, Wall,
        Wall, Empty, Food, Food, Food, Food, Wall, Wall, Food, Food, Food, Food, Food, Food, Wall,
    ],
    [
        Wall, Wall, Wall, Food, Food, Wall, Wall, Wall, Wall, Wall, Wall, Pacman, Empty, Empty,
        Empty, Empty, Empty, Empty, Empty, Wall, Wall, Wall, Wall, Wall, Wall, Food, Food, Wall,
        Wall, Wall,
    ],
    [
        Wall, Food, Food, Food, Food, Food, Food, Wall, Wall, Food, Food, Food, Food, Food, Food,
        Food, Food, Food, Food, Food, Food, Wall, Wall, Food, Food, Food, Food, Food, Empty, Wall,
    ],
    [
        Wall, Food, Wall, Wall, Wall, Wall, Food, Wall, Wall, Food, Wall, Wall, Wall, Wall, Wall,
        Wall, Wall, Wall, Wall, Wall, Food, Wall, Wall, Food, Wall, Wall, Wall, Wall, Food, Wall,
    ],
    [
        Wall, Food, Food, Food, Food, Food, Food, Food, Food, Food, Food, Food, Food, Food, Wall,
        Wall, Food, Food, Food, Food, Food, Food, Food, Food, Food, Food, Food, Food, Food, Wall,
    ],
    [
        Wall,
        Food,
        Wall,
        Wall,
        Wall,
        Wall,
        PowerPellet,
        Wall,
        Wall,
        Wall,
        Wall,
        Wall,
        Wall,
        Food,
        Food,
        Food,
        Food,
        Wall,
        Wall,
        Wall,
        Wall,
        Wall,
        Wall,
        PowerPellet,
        Wall,
        Wall,
        Wall,
        Wall,
        Food,
        Wall,
    ],
    [
        Wall, Food, Food, Food, Food, Food, Food, Food, Food, Food, Food, Food, Food, Food, Wall,
        Wall, Food, Food, Food, Food, Food, Food, Food, Food, Food, Food, Food, Food, Food, Wall,
    ],
    [
        Wall, Food, Wall, Wall, Wall, Wall, Food, Food, Food, Food, Wall, Wall, Food, Food, Food,
        Food, Food, Food, Wall, Wall, Food, Food, Food, Food, Wall, Wall, Wall, Wall, Food, Wall,
    ],
    [Wall; 30],
];
