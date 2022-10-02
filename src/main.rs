#[macro_use]
extern crate log;
extern crate simplelog;

use std::fs::File;

use simplelog::{CombinedLogger, Config, LevelFilter, WriteLogger};

use state::*;

use crate::game_setup::{ChessMove, GameSetup};
use crate::messaging::{get_message, send_message};

pub mod bboard;
pub mod common;
pub mod debug;
pub mod engine;
pub mod evaluator;
pub mod game_setup;
pub mod magic;
pub mod messaging;
pub mod move_generator;
pub mod piece_moves;
pub mod state;

fn main() {
    CombinedLogger::init(vec![WriteLogger::new(
        LevelFilter::Info,
        Config::default(),
        File::create("chess.log").unwrap(),
    )])
    .unwrap();

    let mut setup = GameSetup::new();

    loop {
        let input = get_message();
        let command = input.0.as_str();
        let argument = input.1.as_str();

        match command {
            "xboard" => {
                setup.xboard = true;

                let input = get_message();
                let command = input.0.as_str();
                let argument = input.1.as_str();

                if command != "protover" {
                    return;
                }

                if argument != "2" {
                    println!("# unsupported version");
                    return;
                }

                send_message("");
                send_message("feature usermove=1");

                let input = get_message();
                let command = input.0.as_str();

                if command != "accepted" {
                    return;
                }

                send_message("feature ping=1");
                get_message();

                send_message("feature variants=\"normal\"");
                get_message();

                send_message("feature sigint=0");
                get_message();

                send_message("feature sigterm=1");
                get_message();

                send_message("feature colors=0");
                get_message();

                send_message("feature nps=0");
                get_message();

                send_message("feature setboard=1");
                get_message();

                send_message("feature done=1");
                get_message();

                send_message("");
            }

            "hard" => {
                setup.pondering = true;
            }

            "easy" => {
                setup.pondering = false;
            }

            "new" => {
                setup.game_state = ChessState::new_game();
                setup.computer_player.insert(Side::White, false);
                setup.computer_player.insert(Side::Black, false);
                setup.forced = false;
            }

            "setboard" => {
                setup.game_state = ChessState::from_fen(argument);
            }

            "force" => {
                // stop computer from making new moves
                setup.computer_player.insert(Side::White, false);
                setup.computer_player.insert(Side::Black, true);
                setup.forced = true;
            }

            "time" => {
                setup.time = argument.parse().unwrap();
            }

            "otim" => {
                setup.time = argument.parse().unwrap();
            }

            "quit" => {
                return;
            }

            "ping" => {
                send_message(format!("pong {}", argument).as_str());
            }

            "usermove" => {
                let user_move = ChessMove::parse(argument, &setup.game_state);

                match user_move {
                    Ok(user_move) => {
                        setup
                            .computer_player
                            .insert(setup.game_state.next_to_move, false);

                        setup.game_state.do_move(&user_move);

                        // now computer moves as the opposite color
                        setup
                            .computer_player
                            .insert(setup.game_state.next_to_move, true);

                        setup.forced = false;
                        info!("parsed user move {:?}", user_move);
                    }
                    Err(msg) => {
                        error!(
                            "error making move {} on board: {}",
                            argument,
                            setup.game_state.to_fen()
                        );
                        error!("{}", msg);

                        setup.forced = true;
                    }
                }
            }

            "go" => {
                match setup.game_state.next_to_move {
                    Side::White => {
                        send_message("# received 'go', playing as white\n");
                    }
                    Side::Black => send_message("# received 'go', playing as black\n"),
                };

                setup
                    .computer_player
                    .insert(setup.game_state.next_to_move, true);
                setup.forced = false;
            }

            _ => {
                warn!("# unknown command: {}", command);
            }
        }

        if !setup.xboard {
            return;
        }

        if setup.forced {
            continue;
        }

        let do_move = setup
            .computer_player
            .get(&setup.game_state.next_to_move)
            .unwrap();

        if !do_move {
            continue;
        }

        let next_move = setup.engine.find_best_move(&setup.game_state);

        if next_move.is_none() {
            setup.forced = true;
            info!("computer was checkmated...");
            continue;
        }

        let next_move = next_move.unwrap();

        info!(
            "computer moves as {:?}: {:?}",
            setup.game_state.next_to_move, next_move
        );

        send_message(format!("move {}", next_move.to_string()).as_str());

        setup.game_state.do_move(&next_move);
        info!("new board state {}", setup.game_state.to_fen());
    }
}
