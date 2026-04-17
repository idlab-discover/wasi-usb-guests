pub mod types;
pub mod window;

use crate::{
    types::{DEFAULT_MAZE, GamepadState, Ghost, MazeType, Personality},
    window::print_maze,
};
use colored::Colorize;
use std::time::{Duration, Instant};

pub fn run_game_loop<F>(mut get_input: F) -> anyhow::Result<()>
where
    F: FnMut() -> Option<GamepadState>,
{
    let mut maze = DEFAULT_MAZE;

    let mut ghosts = [
        Ghost {
            pos: (5, 16),
            last_pos: (5, 16),
            under_tile: MazeType::Food,
            personality: Personality::Blinky,
            home: (5, 16),
        },
        Ghost {
            pos: (7, 28),
            last_pos: (7, 28),
            under_tile: MazeType::Food,
            personality: Personality::Pinky,
            home: (7, 28),
        },
        Ghost {
            pos: (5, 2),
            last_pos: (5, 2),
            under_tile: MazeType::Food,
            personality: Personality::Clyde,
            home: (5, 2),
        },
    ];

    for g in ghosts.iter() {
        maze[g.pos.0][g.pos.1] = g.personality.into();
    }

    let mut current_pos = (6, 11);
    let mut score = 0;
    let mut lives = 3;
    let mut power_timer = Duration::from_secs(0);

    let mut last_move = Instant::now();
    let mut last_frame = Instant::now();
    let move_delay = Duration::from_millis(150);

    let mut last_ghost_move = Instant::now();
    let ghost_delay = Duration::from_millis(400);

    let mut last_processed_state = GamepadState::default();

    print!("\x1B[?25l"); // Hide cursor
    print!("\x1B[2J\x1B[H"); // Clear screen and home

    loop {
        if let Some(state) = get_input() {
            last_processed_state = state;
            let now = Instant::now();

            // Pacman Movement
            if now.duration_since(last_move) >= move_delay {
                let mut dy = 0;
                let mut dx = 0;

                if state.up || state.lstick_y > 0.5 {
                    dy = -1;
                } else if state.down || state.lstick_y < -0.5 {
                    dy = 1;
                } else if state.left || state.lstick_x < -0.5 {
                    dx = -1;
                } else if state.right || state.lstick_x > 0.5 {
                    dx = 1;
                }

                if dy != 0 || dx != 0 {
                    let mut new_y = (current_pos.0 as i32 + dy) as usize;
                    let mut new_x = (current_pos.1 as i32 + dx) as usize;

                    if maze[new_y][new_x] != MazeType::Wall {
                        if maze[new_y][new_x] == MazeType::Food {
                            score += 10;
                        } else if maze[new_y][new_x] == MazeType::PowerPellet {
                            score += 50;
                            power_timer = Duration::from_secs(10);
                        } else if maze[new_y][new_x] == Personality::Blinky.into()
                            || maze[new_y][new_x] == Personality::Pinky.into()
                            || maze[new_y][new_x] == Personality::Clyde.into()
                        {
                            lives -= 1;

                            // Move pacman to home pos
                            new_y = 6;
                            new_x = 11;

                            // Reset all ghosts to home on death
                            for g in ghosts.iter_mut() {
                                maze[g.pos.0][g.pos.1] = g.under_tile;
                                g.pos = g.home;
                                g.last_pos = g.home;
                                g.under_tile = maze[g.pos.0][g.pos.1];
                            }
                        } else if maze[new_y][new_x] == MazeType::VulnerableGhost {
                            score += 200;

                            // Find and reset the caught ghost
                            for g in ghosts.iter_mut() {
                                if (g.pos.0, g.pos.1) == (new_y, new_x) {
                                    g.pos = g.home;
                                    g.last_pos = g.home;
                                    g.under_tile = MazeType::Empty;
                                    break;
                                }
                            }
                        }

                        maze[current_pos.0][current_pos.1] = MazeType::Empty;
                        current_pos = (new_y, new_x);
                        maze[current_pos.0][current_pos.1] = MazeType::Pacman;
                        last_move = now;
                    }
                }
            }
        }

        // Update power timer
        let now = Instant::now();
        let frame_delta = now.duration_since(last_frame);
        last_frame = now;

        if power_timer > Duration::from_secs(0) {
            if power_timer > frame_delta {
                power_timer -= frame_delta;
            } else {
                power_timer = Duration::from_secs(0);
            }
        }

        // Ghost Movement
        let is_frightened = power_timer > Duration::from_secs(0);
        let current_ghost_delay = if is_frightened {
            Duration::from_millis(800)
        } else {
            ghost_delay
        };

        if now.duration_since(last_ghost_move) >= current_ghost_delay {
            // Clear all ghosts from maze first to prevent overlap issues
            for g in ghosts.iter_mut() {
                maze[g.pos.0][g.pos.1] = g.under_tile;
            }

            // Calculate and move each ghost
            let mut collision_occurred = false;
            for (i, ghost) in ghosts.iter_mut().enumerate() {
                let target_tile;

                // Calculate target
                if is_frightened {
                    let pseudo_rand = (now.duration_since(last_move).as_millis() as usize) + i;
                    target_tile = ((pseudo_rand % 14), (pseudo_rand % 30));
                } else {
                    match ghost.personality {
                        Personality::Blinky => {
                            target_tile = (current_pos.0, current_pos.1);
                        }
                        Personality::Pinky => {
                            target_tile = (current_pos.0 + 2, current_pos.1 + 2);
                        }
                        Personality::Clyde => {
                            let row_diff = ghost.pos.0 as isize - current_pos.0 as isize;
                            let col_diff = ghost.pos.1 as isize - current_pos.1 as isize;
                            let dist_sq = row_diff * row_diff + col_diff * col_diff;

                            if dist_sq > 64 {
                                target_tile = (current_pos.0, current_pos.1);
                            } else {
                                target_tile = (12, 1);
                            }
                        }
                    }
                }

                // Decide Direction (prevent 180 and pick closest to target)
                let dirs: [(isize, isize); 4] = [(0, 1), (0, -1), (1, 0), (-1, 0)];
                let mut best_dir = None;
                let mut min_dist = f32::MAX;

                for &(dr, dc) in dirs.iter() {
                    let nr = (ghost.pos.0 as isize + dr) as usize;
                    let nc = (ghost.pos.1 as isize + dc) as usize;

                    if (0..14).contains(&nr)
                        && (0..30).contains(&nc)
                        && maze[nr][nc] != MazeType::Wall
                        && (nr, nc) != ghost.last_pos
                    {
                        let row_diff = nr as isize - target_tile.0 as isize;
                        let col_diff = nc as isize - target_tile.1 as isize;
                        let dist = ((row_diff * row_diff + col_diff * col_diff) as f32).sqrt();

                        if dist < min_dist {
                            min_dist = dist;
                            best_dir = Some((nr, nc));
                        }
                    }
                }

                if best_dir.is_none() {
                    best_dir = Some(ghost.last_pos);
                }

                if let Some((nr, nc)) = best_dir {
                    if (nr, nc) == (current_pos.0, current_pos.1) {
                        if is_frightened {
                            score += 200;
                            ghost.pos = ghost.home;
                            ghost.last_pos = ghost.home;
                            ghost.under_tile = MazeType::Empty;
                        } else {
                            lives -= 1;
                            maze[current_pos.0][current_pos.1] = MazeType::Empty;
                            current_pos = (6, 11);
                            maze[current_pos.0][current_pos.1] = MazeType::Pacman;
                            collision_occurred = true;
                            // Reset ghosts on collision later
                            break;
                        }
                    } else {
                        ghost.last_pos = ghost.pos;
                        ghost.pos = (nr, nc);
                        ghost.under_tile = maze[nr][nc];
                    }
                }
            }

            if collision_occurred {
                // Reset all ghosts to home
                for g in ghosts.iter_mut() {
                    g.pos = g.home;
                    g.last_pos = g.home;
                    g.under_tile = maze[g.pos.0][g.pos.1];
                }
            }

            // Update all ghosts visuals in maze after movements are finalized
            for ghost in ghosts.iter() {
                maze[ghost.pos.0][ghost.pos.1] = if is_frightened {
                    MazeType::VulnerableGhost
                } else {
                    ghost.personality.into()
                };
            }

            last_ghost_move = now;
        }

        // Win condition check
        let mut food_left = false;
        for row in maze.iter() {
            for el in row.iter() {
                if *el == MazeType::Food || *el == MazeType::PowerPellet {
                    food_left = true;
                    break;
                }
            }
        }

        // Render frame
        print_maze(
            &maze,
            score,
            lives,
            power_timer.as_secs(),
            &last_processed_state,
        )?;

        if !food_left {
            println!("\n{}", "YOU WIN!".green().bold());
            break;
        }

        if lives == 0 {
            println!("{}", "GAME OVER!".red().bold());
            break;
        }
    }

    print!("\x1B[?25h"); // Show cursor

    Ok(())
}
