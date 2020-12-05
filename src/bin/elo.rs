use chrono::Local;
use santorini_ai::mcts::tree_policy::PUCT;
use santorini_ai::player::{FullPlayer, HeuristicAI, MctsSantoriniParams, RandomAI, StepResult};
use santorini_ai::santorini;
use santorini_ai::ui::UpdateError;
use std::thread::{self, JoinHandle};

struct Contestant<'a> {
    name: &'a str,
    score: f64,
    diff: f64,
    instantiation: Box<dyn Fn() -> Box<dyn FullPlayer>>,
}

impl<'a> Contestant<'a> {
    fn new(name: &'a str, instantiation: Box<dyn Fn() -> Box<dyn FullPlayer>>) -> Self {
        Contestant {
            name,
            score: 1500.0,
            diff: 0.0,
            instantiation,
        }
    }
}

macro_rules! action {
    ($name:ident, $mode:ty) => {
        fn $name<'a>(
            mut p1: &'a mut Box<dyn FullPlayer>,
            mut p2: &'a mut Box<dyn FullPlayer>,
            game: santorini::Game<$mode>,
        ) -> Result<f64, UpdateError> {
            let p = match game.player() {
                santorini::Player::PlayerOne => &mut p1,
                santorini::Player::PlayerTwo => &mut p2,
            };

            p.prepare(&game);

            loop {
                match p.step(&game)? {
                    StepResult::NoMove => (),
                    StepResult::PlaceTwo(game) => return place_two(p1, p2, game),
                    StepResult::Move(game) => return mv(p1, p2, game),
                    StepResult::Build(game) => return build(p1, p2, game),
                    StepResult::Victory(game) => {
                        return match game.player() {
                            santorini::Player::PlayerOne => Ok(1.0),
                            santorini::Player::PlayerTwo => Ok(0.0),
                        }
                    }
                }
            }
        }
    };
}

action!(place_one, santorini::PlaceOne);
action!(place_two, santorini::PlaceTwo);
action!(mv, santorini::Move);
action!(build, santorini::Build);

fn play(c1: &Contestant, c2: &Contestant) -> JoinHandle<Result<f64, UpdateError>> {
    let mut p1 = (*c1.instantiation)();
    let mut p2 = (*c2.instantiation)();

    thread::spawn(move || place_one(&mut p1, &mut p2, santorini::new_game()))
}

fn main() -> Result<(), UpdateError> {
    println!("Calculating ELO scores...");

    let mut players = [
        Contestant::new("Random", Box::new(|| RandomAI::new())),
        Contestant::new("Heuristic", Box::new(|| HeuristicAI::new())),
        Contestant::new(
            "MCTS UCT",
            Box::new(|| MctsSantoriniParams::default().boxed()),
        ),
        Contestant::new(
            "MCTS PUCT",
            Box::new(|| {
                MctsSantoriniParams::default()
                    .tree_policy(PUCT {
                        parameter: f64::sqrt(2.0),
                    })
                    .boxed()
            }),
        ),
    ];

    let mut k = 100.0;
    loop {
        println!("");
        println!("{}", Local::now().to_string());
        println!("  Scores:");
        for p in players.iter() {
            println!("    {}: {}", p.name, p.score);
        }

        let mut threads = Vec::new();
        for _ in 0..5 {
            for i1 in 0..players.len() {
                for i2 in i1 + 1..players.len() {
                    let p1 = &players[i1];
                    let p2 = &players[i2];
                    threads.push((i1, i2, play(p1, p2)));
                }
            }
        }

        for (i1, i2, thread) in threads {
            let p1 = &players[i1];
            let p2 = &players[i2];

            let ea = (p2.score - p1.score) / 400.0;
            let ea = 1.0 / (1.0 + 10.0f64.powf(ea));

            let result = thread.join().expect("Game thread panicked!")?;

            let diff = k * (result - ea);
            players[i1].diff += diff;
            players[i2].diff -= diff;
        }

        for player in players.iter_mut() {
            player.score += player.diff;
            player.diff = 0.0;
        }

        k *= 0.75;
        if k < 10.0 {
            break;
        }
    }

    Ok(())
}
