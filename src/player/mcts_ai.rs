use crate::player::{FullPlayer, Player, StepResult};
use crate::santorini::{
    self, ActionResult, Build, Game, GameState, Move, NormalState, PlaceOne, PlaceTwo, Point,
};
use crate::ui::{BoardWidget, UpdateError};
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

use crate::mcts::santorini::{SantoriniExpansion, SantoriniNode, SantoriniSimulation};
use crate::mcts::{Mcts, MctsParams};

pub enum MctsOrParams<T, R: Rng> {
    Params(MctsParams<T, R>),
    Tree(Mcts<T, R>),
}

impl<T, R: Rng> From<MctsParams<T, R>> for MctsOrParams<T, R> {
    fn from(params: MctsParams<T, R>) -> MctsOrParams<T, R> {
        MctsOrParams::Params(params)
    }
}

impl<T, R: Rng> MctsOrParams<T, R> {
    fn params(&mut self) -> &mut MctsParams<T, R> {
        match self {
            MctsOrParams::Tree(tree) => &mut tree.params,
            MctsOrParams::Params(params) => params,
        }
    }

    fn tree(&mut self, node: T) -> &mut Mcts<T, R> {
        take_mut::take(self, move |mcts_or_params| match mcts_or_params {
            MctsOrParams::Params(params) => MctsOrParams::Tree(Mcts::new(params, node)),
            MctsOrParams::Tree(_) => mcts_or_params,
        });

        match self {
            MctsOrParams::Tree(tree) => tree,
            // Params branch will be replaced with a Tree branch above
            MctsOrParams::Params(_) => unsafe { std::hint::unreachable_unchecked() },
        }
    }

    fn expect<S: 'static + Send>(&self, message: S) -> &Mcts<T, R> {
        match self {
            MctsOrParams::Tree(tree) => tree,
            MctsOrParams::Params(_) => panic!(message),
        }
    }
}

pub type MctsSantoriniParams = MctsParams<SantoriniNode, SmallRng>;
impl MctsSantoriniParams {
    pub fn default() -> Self {
        MctsSantoriniParams::new(
            SantoriniSimulation {},
            SantoriniExpansion {},
            SmallRng::from_entropy(),
        )
    }

    pub fn boxed(self) -> Box<dyn FullPlayer> {
        MctsAI::from(self).boxed()
    }
}

pub type MctsAI = MctsOrParams<SantoriniNode, SmallRng>;

impl MctsAI {
    fn boxed(self) -> Box<dyn FullPlayer> {
        Box::new(self)
    }
}

static EMPTY: Vec<Point> = Vec::new();

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

// TODO: Add support for placement to the tree
fn random_pt<R: Rng>(rng: &mut R) -> Point {
    let x: i8 = rng.gen_range(1, santorini::BOARD_WIDTH.0 - 1);
    let y: i8 = rng.gen_range(1, santorini::BOARD_HEIGHT.0 - 1);
    Point::new(x.into(), y.into())
}

impl Player<PlaceOne> for MctsAI {
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
        let pt1 = random_pt(&mut self.params().rng);
        let pt2 = random_pt(&mut self.params().rng);
        match game.can_place(pt1, pt2) {
            Some(action) => Ok(StepResult::PlaceTwo(game.clone().apply(action))),
            None => Ok(StepResult::NoMove),
        }
    }
}

impl Player<PlaceTwo> for MctsAI {
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
        let pt1 = random_pt(&mut self.params().rng);
        let pt2 = random_pt(&mut self.params().rng);
        match game.can_place(pt1, pt2) {
            Some(action) => Ok(StepResult::Move(game.clone().apply(action))),
            None => Ok(StepResult::NoMove),
        }
    }
}

impl Player<Move> for MctsAI {
    fn prepare(&mut self, game: &Game<Move>) {
        let tree = self.tree((*game).into());
        let node = &tree.root_node;
        if node.state.matches(*game) {
            return;
        }

        take_mut::take(&mut tree.root_node, |node| {
            for child in node.children.expect("Unexpanded root node!") {
                if child.state.matches(*game) {
                    return child;
                }
            }

            panic!("Current game state not in tree!");
        });
    }

    fn render(&self, game: &Game<Move>) -> BoardWidget {
        default_render(game)
    }

    fn step(&mut self, game: &Game<Move>) -> Result<StepResult, UpdateError> {
        let tree = self.tree((*game).into());
        if tree.root_node.state.matches(*game) {
            tree.advance();
        }

        let action = tree.root_node.state.mv.expect("Missing move action!");
        match game.clone().apply(action) {
            ActionResult::Continue(game) => Ok(StepResult::Build(game)),
            ActionResult::Victory(game) => Ok(StepResult::Victory(game)),
        }
    }
}

impl Player<Build> for MctsAI {
    fn prepare(&mut self, _: &Game<Build>) {}

    fn render(&self, game: &Game<Build>) -> BoardWidget {
        default_render(game)
    }

    fn step(&mut self, game: &Game<Build>) -> Result<StepResult, UpdateError> {
        let action = self
            .expect("Unitialized tree!")
            .root_node
            .state
            .build
            .expect("Missing build action!");
        match game.clone().apply(action) {
            ActionResult::Continue(game) => Ok(StepResult::Move(game)),
            ActionResult::Victory(game) => Ok(StepResult::Victory(game)),
        }
    }
}
