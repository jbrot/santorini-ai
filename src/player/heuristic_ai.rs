use cached::proc_macro::cached;
use cached::SizedCache;
use rand::Rng;
use std::cmp::Ordering;
use std::mem;

use crate::player::{FullPlayer, Player, StepResult};
use crate::santorini::{
    self, ActionResult, Build, BuildAction, CoordLevel, Game, GameState, Move, MoveAction,
    NormalState, PlaceOne, PlaceTwo, Point,
};
use crate::ui::{BoardWidget, UpdateError};

static EMPTY: Vec<Point> = Vec::new();

pub struct HeuristicAI {
    mv: Option<MoveAction>,
    build: Option<BuildAction>,
}

impl HeuristicAI {
    pub fn new() -> Box<dyn FullPlayer> {
        Box::new(HeuristicAI {
            mv: None,
            build: None,
        })
    }
}

fn default_render<'a, T: GameState + NormalState>(game: &Game<T>) -> BoardWidget<'a> {
    BoardWidget {
        board: game.board(),
        player: game.player(),
        cursor: None,

        highlights: &EMPTY,
        player1_locs: game
            .player_pawns(santorini::Player::PlayerOne)
            .iter()
            .map(|pawn| pawn.pos())
            .collect(),
        player2_locs: game
            .player_pawns(santorini::Player::PlayerTwo)
            .iter()
            .map(|pawn| pawn.pos())
            .collect(),
    }
}

fn possible_actions(
    game: &Game<Move>,
) -> Vec<((MoveAction, Option<BuildAction>), ActionResult<Move>)> {
    game.clone()
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

fn player_score(game: &Game<Move>, player: santorini::Player) -> f64 {
    let pawn_score: f64 = game
        .player_pawns(player)
        .iter()
        .map(|pawn| height_score(game.board().level_at(pawn.pos())))
        .sum();
    let pawn_score = pawn_score / 2.0;

    let move_scores: Vec<f64> = game
        .player_pawns(player)
        .iter()
        .map(|pawn| {
            pawn.neighbors()
                .into_iter()
                .map(|loc| height_score(game.board().level_at(loc)))
        })
        .flatten()
        .collect();
    let move_sum: f64 = move_scores.iter().sum();
    let move_score: f64 = move_sum / (move_scores.len() as f64);

    pawn_score * 0.7 + move_score * 0.3
}

fn diff_score(game: &Game<Move>) -> f64 {
    let s1 = player_score(game, game.player());
    let s2 = player_score(game, game.player().other());
    s1 - s2
}

fn dist_score(game: &Game<Move>) -> f64 {
    let mut max_dist = 0;
    for p1 in game.active_pawns().iter() {
        for p2 in game.inactive_pawns().iter() {
            max_dist = i8::max(max_dist, p1.pos().distance(p2.pos()));
        }
    }
    let dist_score = 1.0 - (max_dist as f64) / 5.0;
    dist_score * dist_score
}

fn score_recurse(action: &ActionResult<Move>, active_player: bool, depth: u8) -> f64 {
    match action {
        ActionResult::Victory(_) => {
            if active_player {
                1.0
            } else {
                -1.0
            }
        }
        ActionResult::Continue(game) => {
            if depth == 0 {
                if active_player {
                    0.3 * dist_score(game) - 0.7 * diff_score(game)
                } else {
                    0.3 * dist_score(game) + 0.7 * diff_score(game)
                }
            } else {
                let scores = possible_actions(game)
                    .into_iter()
                    .map(|(_, action)| score_recurse(&action, !active_player, depth - 1));
                if active_player {
                    let mut min = f64::MAX;
                    for score in scores {
                        if score == -1.0 {
                            return -1.0;
                        }
                        min = f64::min(min, score);
                    }
                    min
                } else {
                    let mut max = f64::MIN;
                    for score in scores {
                        if score == 1.0 {
                            return 1.0;
                        }
                        max = f64::max(max, score);
                    }
                    max
                }
            }
        }
    }
}

#[cached(
    type = "SizedCache<ActionResult<Move>, f64>",
    create = "{ SizedCache::with_size(128) }",
    convert = "{ action.clone() }"
)]
fn score(action: &ActionResult<Move>) -> f64 {
    score_recurse(action, true, 2)
}

fn choose_action(game: &Game<Move>) -> (MoveAction, Option<BuildAction>) {
    possible_actions(game)
        .into_iter()
        .max_by(|a, b| {
            score(&a.1)
                .partial_cmp(&score(&b.1))
                .unwrap_or(Ordering::Equal)
        })
        .expect("No good moves found!")
        .0
}

fn random_pt() -> Point {
    let mut rng = rand::thread_rng();
    let x: i8 = rng.gen_range(1, santorini::BOARD_WIDTH.0 - 1);
    let y: i8 = rng.gen_range(1, santorini::BOARD_HEIGHT.0 - 1);
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
    fn prepare(&mut self, _: &Game<Move>) {
        self.mv = None;
        self.build = None;
    }

    fn render(&self, game: &Game<Move>) -> BoardWidget {
        default_render(game)
    }

    fn step(&mut self, game: &Game<Move>) -> Result<StepResult, UpdateError> {
        if let None = self.mv {
            let (mv, build) = choose_action(game);
            self.mv = Some(mv);
            self.build = build;
        }

        let action = mem::replace(&mut self.mv, None).expect("No move selected!");
        match game.clone().apply(action) {
            ActionResult::Continue(game) => Ok(StepResult::Build(game)),
            ActionResult::Victory(game) => Ok(StepResult::Victory(game)),
        }
    }
}

impl Player<Build> for HeuristicAI {
    fn prepare(&mut self, _: &Game<Build>) {}

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
