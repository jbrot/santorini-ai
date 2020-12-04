use std::io;
use termion::input::MouseTerminal;
use termion::raw::IntoRawMode;
use tui::backend::TermionBackend;
use tui::Terminal;

use santorini_ai::ui::{self, UpdateError};
use santorini_ai::player::{FullPlayer, StepResult, HeuristicAI, MCTSAI, RandomAI};
use santorini_ai::santorini;

struct Contestant<'a> {
    name: &'a str,
    score: f64,
    instantiation: Box<dyn Fn() -> Box<dyn FullPlayer>>,
}

impl<'a> Contestant<'a> {
    fn new(name: &'a str, instantiation: Box<dyn Fn() -> Box<dyn FullPlayer>>) -> Self {
        Contestant {
            name,
            score: 1500.0,
            instantiation,
        }
    }
}

macro_rules! action {
    ($name:ident, $mode:ty) => {
        fn $name (p1: &mut Box<dyn FullPlayer>, p2: &mut Box<dyn FullPlayer>, game: santorini::Game<$mode>) -> Result<f64, UpdateError> {
            let p = match game.player() {
                santorini::Player::PlayerOne => p1,
                santorini::Player::PlayerOne => p2,
            };
        
            p.prepare(&game);
        
            loop {
                match p.step(&game)? {
                    StepResult::NoMove => (),
                    StepResult::PlaceTwo(game) => return place_two(p1, p2, game),
                    StepResult::Move(game) => return mv(p1, p2, game),
                    StepResult::Build(game) => return build(p1, p2, game),
                    StepResult::Victory(game) => return match game.player() {
                        santorini::Player::PlayerOne => Ok(1.0),
                        santorini::Player::PlayerOne => Ok(0.0),
                    },
                }
            }
        }
    }
}

action!(place_one, santorini::PlaceOne);
action!(place_two, santorini::PlaceTwo);
action!(mv, santorini::Move);
action!(build, santorini::Build);

fn play(c1: &Contestant, c2: &Contestant) -> Result<f64, UpdateError> {
    let mut p1 = (*c1.instantiation)();
    let mut p2 = (*c2.instantiation)();
    
    let game = santorini::new_game();

    place_one(&mut p1, &mut p2, santorini::new_game())
}

fn main() -> Result<(), UpdateError> {
    println!("Calculating ELO scores...");

    let mut players = [ 
        Contestant::new("Random", Box::new(|| { RandomAI::new() })),
        Contestant::new("Heuristic", Box::new(|| { HeuristicAI::new() })),
        Contestant::new("MCTS", Box::new(|| { MCTSAI::new() })),
    ];

    let mut k = 100.0;
    loop {
        println!("Scores:");
        for p in players.iter() {
            println!("  {}: {}", p.name, p.score);
        }

        for _ in 0..5 {
            for i1 in 0..players.len() {
                for i2 in i1 + 1..players.len() {
                    let p1 = &players[i1];
                    let p2 = &players[i2];

                    let ea = (p2.score - p1.score) / 400.0;
                    let ea = 1.0 / (1.0 + 10.0f64.powf(ea));

                    let result = play(p1, p2)?;

                    let diff = k * (result - ea);
                    players[i1].score += diff;
                    players[i2].score -= diff;
                }
            }
        }

        k *= 0.9;
        if k < 10.0 {
            break;
        }
    }

    Ok(())
}
