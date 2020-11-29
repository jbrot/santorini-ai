use rand::Rng;
use crate::player::{FullPlayer, Player, StepResult};
use crate::santorini::{
    self, ActionResult, Build, Game, GameState, Move, NormalState, PlaceOne, PlaceTwo, Point,
};
use crate::ui::{BoardWidget, UpdateError};

static EMPTY: Vec<Point> = Vec::new();

pub struct RandomAI {}

impl RandomAI {
    pub fn new() -> Box<dyn FullPlayer> {
        Box::new(RandomAI {})
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

fn random_pt() -> Point {
    let mut rng = rand::thread_rng();
    let x: i8 = rng.gen_range(0, santorini::BOARD_WIDTH.0);
    let y: i8 = rng.gen_range(0, santorini::BOARD_HEIGHT.0);
    Point::new(x.into(), y.into())
}

impl Player<PlaceOne> for RandomAI {
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

impl Player<PlaceTwo> for RandomAI {
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

impl Player<Move> for RandomAI {
    fn prepare(&mut self, _: &Game<Move>) {}

    fn render(&self, game: &Game<Move>) -> BoardWidget {
        default_render(game)
    }

    fn step(&mut self, game: &Game<Move>) -> Result<StepResult, UpdateError> {
        let actions: Vec<_> = game
            .active_pawns()
            .iter()
            .map(|pawn| pawn.actions())
            .flatten()
            .collect();
        let action_idx = rand::thread_rng().gen_range(0, actions.len());
        let action = actions.into_iter().nth(action_idx).unwrap();
        match game.clone().apply(action) {
            ActionResult::Continue(game) => Ok(StepResult::Build(game)),
            ActionResult::Victory(game) => Ok(StepResult::Victory(game)),
        }
    }
}

impl Player<Build> for RandomAI {
    fn prepare(&mut self, _: &Game<Build>) { }

    fn render(&self, game: &Game<Build>) -> BoardWidget {
        default_render(game)
    }

    fn step(&mut self, game: &Game<Build>) -> Result<StepResult, UpdateError> {
        let actions: Vec<_> = game
            .active_pawns()
            .iter()
            .map(|pawn| pawn.actions())
            .flatten()
            .collect();
        let action_idx = rand::thread_rng().gen_range(0, actions.len());
        let action = actions.into_iter().nth(action_idx).unwrap();
        match game.clone().apply(action) {
            ActionResult::Continue(game) => Ok(StepResult::Move(game)),
            ActionResult::Victory(game) => Ok(StepResult::Victory(game)),
        }
    }
}
