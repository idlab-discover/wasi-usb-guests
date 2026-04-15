use crate::types::{BOARD_COLS, BOARD_ROWS, CellType};
use colored::Colorize;
use std::io::{self, Write};

pub(crate) fn print_board(
    board: &[[CellType; BOARD_COLS]; BOARD_ROWS],
    score: u32,
    length: usize,
    _state: &crate::types::GamepadState,
) -> anyhow::Result<()> {
    print!("\x1B[H"); // Home cursor

    println!("{}", "==== WASM WASI-USB SNAKE ====".blue().bold());

    println!(
        "Score: {} | Length: {}",
        score.to_string().cyan(),
        length.to_string().yellow(),
    );
    println!("{}", "------------------------------".blue());

    for row in board.iter() {
        for cell in row.iter() {
            print!("{}", cell.icon());
        }
        println!();
    }

    io::stdout().flush()?;
    Ok(())
}
