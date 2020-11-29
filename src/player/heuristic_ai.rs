use cached::proc_macro::cached;
use cached::SizedCache;
use rand::Rng;
use std::cmp::Ordering;
use std::mem;

use crate::player::{FullPlayer, Player, StepResult};
use crate::santorini::{
    self, ActionResult, Build, BuildAction, CoordLevel, MoveAction, Game, GameState, Move, NormalState, PlaceOne, PlaceTwo, Point,
};
use crate::ui::{BoardWidget, UpdateError};

static EMPTY: Vec<Point> = Vec::new();

pub struct HeuristicAI {
    mv: Option<MoveAction>,
    build: Option<BuildAction>,
}

impl HeuristicAI {
    pub fn new() -> Box<dyn FullPlayer> {
        Box::new(HeuristicAI { mv: None, build: None })
    }
}

fn default_render<'a, T: GameState + NormalState>(game: &Game<T>) -> BoardWidget<'a> {
    BoardWidget {
        board: game.board(),
        player: game.player(),
        cursor: None,

        highlights: &EMPTY,
        player1_locs: game.player1_pawns().iter().map(|pawn| pawn.pos()).collect(),
        player2_locs: game.player2_pawns().iter().map(|pawn| pawn.pos()).collect(),
    }
}

fn possible_actions(game: &Game<Move>) -> Vec<((MoveAction, Option<BuildAction>), ActionResult<Move>)> {
    game
        .clone()
        .active_pawns()
        .iter()
        .map(|pawn| pawn.actions())
        .flatten()
        .map(|mv| match game.clone().apply(mv.clone()) {
            ActionResult::Victory(game) => vec![((mv, None), ActionResult::Victory(game))],
            ActionResult::Continue(game) => game
                .active_pawn()
                .actions()
                .into_iter()
                .map(|build| ((mv.clone(), Some(build.clone())), game.clone().apply(build)))
                .collect(),
        })
        .flatten()
        .collect()
}

fn height_score(height: CoordLevel) -> f64 {
    match height {
        CoordLevel::Ground => 0.0,
        CoordLevel::One => 0.3,
        CoordLevel::Two => 0.8,
        CoordLevel::Three => 1.0,
        CoordLevel::Capped => 0.0,
    }
}

fn raw_score(game: &Game<Move>) -> f64 {
    let pawn_score: f64 = game
        .active_pawns()
        .iter()
        .map(|pawn| height_score(game
                                 .board()
                                 .level_at(pawn.pos())))
        .sum();
    let move_scores: Vec<f64> = game
        .active_pawns()
        .iter()
        .map(|pawn| pawn
             .actions()
             .into_iter()
             .map(|mv| height_score(game
                                    .board()
                                    .level_at(mv.to()))))
        .flatten()
        .collect();
    let move_sum: f64 = move_scores.iter().sum();
    let move_score: f64 = move_sum / (move_scores.len() as f64);
    let subtotal = pawn_score * 0.3 + move_score * 0.7;
    subtotal * 0.75
}

fn score_recurse(action: &ActionResult<Move>, active_player: bool, depth: u8) -> f64 {
    match action {
        ActionResult::Victory(_) => if active_player { 1.0 } else { -1.0 },
        ActionResult::Continue(game) => {
            if depth == 0 {
                raw_score(game) * if active_player { 1.0 } else { -1.0 }
            } else {
                let scores = possible_actions(game)
                    .into_iter()
                    .map(|(_, action)| score_recurse(&action, !active_player, depth - 1));
                if active_player {
                    scores
                        .max_by(|a, b| a
                                .partial_cmp(&b)
                                .unwrap_or(Ordering::Equal))
                        .expect("No moves found!")
                } else {
                    let len = scores.len() as f64;
                    let sum: f64 = scores.sum();
                    (sum / len).max(-1.0).min(1.0)
                }
            }
        },
    }
}

#[cached(
    type = "SizedCache<ActionResult<Move>, f64>",
    create = "{ SizedCache::with_size(128) }",
    convert = "{ action.clone() }"
)]
fn score(action: &ActionResult<Move>) -> f64 {
    score_recurse(action, false, 2)
}

fn choose_action(game: &Game<Move>) -> (MoveAction, Option<BuildAction>) {
    possible_actions(game)
        .into_iter()
        .max_by(|a, b| score(&a.1).partial_cmp(&score(&b.1)).unwrap_or(Ordering::Equal))
        .expect("No good moves found!")
        .0
}

fn random_pt() -> Point {
    let mut rng = rand::thread_rng();
    let x: i8 = rng.gen_range(0, santorini::BOARD_WIDTH.0);
    let y: i8 = rng.gen_range(0, santorini::BOARD_HEIGHT.0);
    Point::new(x.into(), y.into())
}

impl Player<PlaceOne> for HeuristicAI {
    fn prepare(&mut self, _: &Game<PlaceOne>) {}

    fn render(&self, game: &Game<PlaceOne>) -> BoardWidget {
        BoardWidget {
            board: game.board(),
            player: game.player(),
            cursor: None,

            highlights: &EMPTY,
            player1_locs: vec![],
            player2_locs: vec![],
        }
    }

    fn step(&mut self, game: &Game<PlaceOne>) -> Result<StepResult, UpdateError> {
        let pt1 = random_pt();
        let pt2 = random_pt();
        match game.can_place(pt1, pt2) {
            Some(action) => Ok(StepResult::PlaceTwo(game.clone().apply(action))),
            None => Ok(StepResult::NoMove),
        }
    }
}

impl Player<PlaceTwo> for HeuristicAI {
    fn prepare(&mut self, _: &Game<PlaceTwo>) {}

    fn render(&self, game: &Game<PlaceTwo>) -> BoardWidget {
        BoardWidget {
            board: game.board(),
            player: game.player(),
            cursor: None,

            highlights: &EMPTY,
            player1_locs: game.player1_locs().to_vec(),
            player2_locs: vec![],
        }
    }

    fn step(&mut self, game: &Game<PlaceTwo>) -> Result<StepResult, UpdateError> {
        let pt1 = random_pt();
        let pt2 = random_pt();
        match game.can_place(pt1, pt2) {
            Some(action) => Ok(StepResult::Move(game.clone().apply(action))),
            None => Ok(StepResult::NoMove),
        }
    }
}

impl Player<Move> for HeuristicAI {
    fn prepare(&mut self, game: &Game<Move>) {
        let (mv, build) = choose_action(game);
        self.mv = Some(mv);
        self.build = build;
    }

    fn render(&self, game: &Game<Move>) -> BoardWidget {
        default_render(game)
    }

    fn step(&mut self, game: &Game<Move>) -> Result<StepResult, UpdateError> {
        let action = mem::replace(&mut self.mv, None).expect("No move selected!");
        match game.clone().apply(action) {
            ActionResult::Continue(game) => Ok(StepResult::Build(game)),
            ActionResult::Victory(game) => Ok(StepResult::Victory(game)),
        }
    }
}

impl Player<Build> for HeuristicAI {
    fn prepare(&mut self, _: &Game<Build>) { }

    fn render(&self, game: &Game<Build>) -> BoardWidget {
        default_render(game)
    }

    fn step(&mut self, game: &Game<Build>) -> Result<StepResult, UpdateError> {
        let action = mem::replace(&mut self.build, None).expect("No build selected!");
        match game.clone().apply(action) {
            ActionResult::Continue(game) => Ok(StepResult::Move(game)),
            ActionResult::Victory(game) => Ok(StepResult::Victory(game)),
        }
    }
}
