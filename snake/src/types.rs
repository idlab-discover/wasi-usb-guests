#[derive(Clone, Copy, PartialEq)]
pub(crate) enum CellType {
    Wall,
    Empty,
    Food,
    SnakeHead,
    SnakeBody,
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub(crate) fn opposite(&self) -> Direction {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }

    pub(crate) fn delta(&self) -> (isize, isize) {
        match self {
            Direction::Up => (-1, 0),
            Direction::Down => (1, 0),
            Direction::Left => (0, -1),
            Direction::Right => (0, 1),
        }
    }
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

impl CellType {
    pub(crate) fn icon(&self) -> &'static str {
        match self {
            CellType::Wall => "\x1B[38;5;75m▓\x1B[0m",
            CellType::Empty => " ",
            CellType::Food => "\x1B[38;2;247;223;30m\x1B[0m", // Yellow
            CellType::SnakeHead => "\x1B[38;2;80;200;80m◆\x1B[0m", // Green
            CellType::SnakeBody => "\x1B[38;2;101;79;240m\x1B[0m", // Wasm purple
        }
    }
}

pub(crate) const BOARD_ROWS: usize = 16;
pub(crate) const BOARD_COLS: usize = 30;

/// Generate an empty board with walls around the border.
pub(crate) fn empty_board() -> [[CellType; BOARD_COLS]; BOARD_ROWS] {
    let mut board = [[CellType::Empty; BOARD_COLS]; BOARD_ROWS];

    board[0].fill(CellType::Wall);
    board[BOARD_ROWS - 1].fill(CellType::Wall);

    for row in board.iter_mut() {
        row[0] = CellType::Wall;
        row[BOARD_COLS - 1] = CellType::Wall;
    }

    board
}
