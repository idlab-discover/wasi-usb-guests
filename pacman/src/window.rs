use crate::types::{GamepadState, MazeType};
use colored::Colorize;
use std::{
    fmt::Display,
    io::{self, Write},
};

macro_rules! render_pressed {
    ($f:expr, $text:expr, $condition:expr) => {
        if $condition {
            write!($f, "{} ", $text.green().bold())
        } else {
            write!($f, "{} ", $text.truecolor(100, 100, 100))
        }
    };
}

impl Display for GamepadState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "LS X: {:>3.0}%\tLS Y: {:>3.0}%\nRS X: {:>3.0}%\tRS Y: {:>3.0}%\nLT  : {:>3.0}%\tRT  : {:>3.0}%",
            100. * self.lstick_x,
            100. * self.lstick_y,
            100. * self.rstick_x,
            100. * self.rstick_y,
            100. * self.lt,
            100. * self.rt,
        )?;

        render_pressed!(f, "A", self.a)?;
        render_pressed!(f, "B", self.b)?;
        render_pressed!(f, "X", self.x)?;
        render_pressed!(f, "Y", self.y)?;

        render_pressed!(f, "Up", self.up)?;
        render_pressed!(f, "Down", self.down)?;
        render_pressed!(f, "Left", self.left)?;
        render_pressed!(f, "Right", self.right)?;

        render_pressed!(f, "Start", self.start)?;
        render_pressed!(f, "Select", self.select)?;
        render_pressed!(f, "LB", self.lb)?;
        render_pressed!(f, "RB", self.rb)?;
        render_pressed!(f, "LS", self.lstick)?;
        render_pressed!(f, "RS", self.rstick)?;

        Ok(())
    }
}

pub(crate) fn print_maze(
    maze: &[[MazeType; 30]; 14],
    score: u32,
    lives: u32,
    power_time: u64,
    _state: &GamepadState,
) -> anyhow::Result<()> {
    print!("\x1B[H"); // Home cursor

    // Print GamepadState
    // println!("{}\n", _state);

    println!("{}", "==== WASM WASI-USB PACMAN ====".blue().bold());

    let power_status = if power_time > 0 {
        format!(" | POWER: {}s", power_time).yellow().bold()
    } else {
        "".clear()
    };

    println!(
        "Score: {} | Lives: {}{}",
        score.to_string().cyan(),
        lives.to_string().red(),
        power_status
    );
    println!("{}", "------------------------------".blue());

    for row in maze.iter() {
        for cell in row.iter() {
            print!("{}", cell.icon());
        }
        println!();
    }

    io::stdout().flush()?;
    Ok(())
}
