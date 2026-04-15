pub mod types;
pub mod window;

use crate::{
    types::{BOARD_COLS, BOARD_ROWS, CellType, Direction, GamepadState, empty_board},
    window::print_board,
};
use colored::Colorize;
use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

pub fn run_game_loop<F>(mut get_input: F) -> anyhow::Result<()>
where
    F: FnMut() -> Option<GamepadState>,
{
    let mut board = empty_board();

    // Snake starts in the center, moving right, length 3
    let start_row = BOARD_ROWS / 2;
    let start_col = BOARD_COLS / 2;
    let mut snake: VecDeque<(usize, usize)> = VecDeque::new();
    snake.push_front((start_row, start_col));
    snake.push_front((start_row, start_col + 1));
    snake.push_front((start_row, start_col + 2));

    let mut direction = Direction::Right;
    let mut score: u32 = 0;

    // Place the initial snake on the board
    for (i, &(r, c)) in snake.iter().enumerate() {
        board[r][c] = if i == 0 {
            CellType::SnakeHead
        } else {
            CellType::SnakeBody
        };
    }

    // Place initial food
    place_food(&mut board, &snake);

    let mut last_move = Instant::now();
    let base_delay = Duration::from_millis(200);

    let mut last_processed_state = GamepadState::default();

    print!("\x1B[?25l"); // Hide cursor
    print!("\x1B[2J\x1B[H"); // Clear screen and home

    loop {
        // Read input and update direction
        if let Some(state) = get_input() {
            last_processed_state = state;

            let new_dir = if state.up || state.lstick_y < -0.5 {
                Some(Direction::Up)
            } else if state.down || state.lstick_y > 0.5 {
                Some(Direction::Down)
            } else if state.left || state.lstick_x < -0.5 {
                Some(Direction::Left)
            } else if state.right || state.lstick_x > 0.5 {
                Some(Direction::Right)
            } else {
                None
            };

            // Prevent 180-degree reversal
            if let Some(d) = new_dir
                && d != direction.opposite()
            {
                direction = d;
            }
        }

        // Move the snake on a fixed timer
        let now = Instant::now();

        // Speed increases as the snake grows (minimum 80ms)
        let speed_bonus = (snake.len() as u64).saturating_mul(3);
        let move_delay = base_delay
            .checked_sub(Duration::from_millis(speed_bonus))
            .unwrap_or(Duration::from_millis(80));

        if now.duration_since(last_move) >= move_delay {
            let (head_r, head_c) = snake[0];
            let (dr, dc) = direction.delta();
            let new_r = (head_r as isize + dr) as usize;
            let new_c = (head_c as isize + dc) as usize;

            // Collision detection: wall or self
            if board[new_r][new_c] == CellType::Wall || board[new_r][new_c] == CellType::SnakeBody {
                // Render one last frame before game over
                print_board(&board, score, snake.len(), &last_processed_state)?;
                println!("{}", "GAME OVER!".red().bold());
                break;
            }

            let ate_food = board[new_r][new_c] == CellType::Food;
            if ate_food {
                score += 10;
            }

            // Clear old head to body
            let (old_hr, old_hc) = snake[0];
            board[old_hr][old_hc] = CellType::SnakeBody;

            // Add new head
            snake.push_front((new_r, new_c));
            board[new_r][new_c] = CellType::SnakeHead;

            // If we didn't eat food, remove the tail
            if !ate_food {
                if let Some((tail_r, tail_c)) = snake.pop_back() {
                    board[tail_r][tail_c] = CellType::Empty;
                }
            } else {
                // Place new food
                place_food(&mut board, &snake);
            }

            last_move = now;
        }

        // Render frame
        print_board(&board, score, snake.len(), &last_processed_state)?;

        // Win condition: snake fills the entire playable area
        let playable_cells = (BOARD_ROWS - 2) * (BOARD_COLS - 2);
        if snake.len() >= playable_cells {
            println!("{}", "YOU WIN!".green().bold());
            break;
        }
    }

    print!("\x1B[?25h"); // Show cursor
    Ok(())
}

/// Place food on a random empty cell using a time-based pseudo-random approach.
fn place_food(board: &mut [[CellType; BOARD_COLS]; BOARD_ROWS], snake: &VecDeque<(usize, usize)>) {
    // Collect all empty cells
    let mut empty_cells: Vec<(usize, usize)> = Vec::new();

    for (r, row) in board.iter().enumerate().take(BOARD_ROWS - 1).skip(1) {
        for (c, cell) in row.iter().enumerate().take(BOARD_ROWS - 1).skip(1) {
            if *cell == CellType::Empty {
                // Make sure it's not occupied by the snake
                if !snake.contains(&(r, c)) {
                    empty_cells.push((r, c));
                }
            }
        }
    }

    if empty_cells.is_empty() {
        return;
    }

    // Pseudo-random selection based on current time
    let seed = Instant::now().elapsed().as_nanos() as usize;
    // Mix the seed a bit to get better distribution
    let mixed = seed.wrapping_mul(1103515245).wrapping_add(12345);

    let (r, c) = empty_cells[mixed % empty_cells.len()];
    board[r][c] = CellType::Food;
}
