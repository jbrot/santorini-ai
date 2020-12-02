use rand::seq::SliceRandom;
use rand::Rng;
use std::mem;
use std::time::{Duration, Instant};
use std::io::Write;

use crate::player::{FullPlayer, Player, StepResult};
use crate::santorini::{
    self, ActionResult, Build, BuildAction, Game, GameState, Move, MoveAction,
    NormalState, PlaceOne, PlaceTwo, Point,
};
use crate::ui::{BoardWidget, UpdateError};

/// Move for each player until the game ends according to the following policy:
///   1. If there exists a winning action, take it.
///   2. Otherwise, pick a random action.
///
/// Returns -1.0 if the active player in the provided game wins and 1.0 if the
/// other player wins.
///
/// In other words, we return 1.0 if the player who moved to get to this state
/// wins---which is what we want because in MCTS we consider Games from the
/// perspective of the previous turn.
pub fn simulate(mut game: Game<Move>) -> i32 {
    let player = game.player();

    enum PossibleAction {
        Victory,
        Continue (Game<Move>),
    }
    
    fn find_action(game: Game<Move>) -> PossibleAction {
        let mut choice = game;
        let mut count = 0.0;
        for mv in game.active_pawns().iter().map(|pawn| pawn.actions()).flatten() {
            match game.apply(mv) {
                ActionResult::Victory(_) => return PossibleAction::Victory,
                ActionResult::Continue(game) => {
                    for build in game.active_pawn().actions() {
                        match game.apply(build) {
                            ActionResult::Victory(_) => return PossibleAction::Victory,
                            ActionResult::Continue(game) => {
                                count += 1.0;
                                if rand::thread_rng().gen::<f64>() < 1.0 / count {
                                    choice = game;
                                }
                            }
                        }
                    }
                }
            }
        }
        PossibleAction::Continue(choice)
    }

    let mut previous = game;

    match find_action(game) {
        PossibleAction::Victory => return if game.player() == player { -1 } else { 1 },
        PossibleAction::Continue(choice) => game = choice,
    }

    loop {
        match find_action(game) {
            PossibleAction::Continue(choice) => {
                previous = game;
                game = choice;
            }
            PossibleAction::Victory => {
                // Back track to see if this could be avoided
                let mut actions = possible_actions(previous);
                &mut actions.shuffle(&mut rand::thread_rng());
                let mut found = false;
                for (_, result) in actions {
                    // We know this can't be a winning move, otherwise we would have
                    // already taken it instead of getting here.
                    let new_game = result.unwrap();
                    match find_action(new_game) {
                        PossibleAction::Victory => (),
                        PossibleAction::Continue(choice) => {
                            // Found a blocking move
                            previous = new_game;
                            game = choice;
                            found = true;
                            break;
                        }
                    }
                }

                // Legitimate win
                if !found {
                    return if game.player() == player { -1 } else { 1 }
                }
            }
        }
    }
}

/// List all possible actions
fn possible_actions(
    game: Game<Move>,
) -> Vec<((MoveAction, Option<BuildAction>), ActionResult<Move>)> {
    game.active_pawns()
        .iter()
        .map(|pawn| pawn.actions())
        .flatten()
        .map(|mv| match game.apply(mv) {
            ActionResult::Victory(game) => vec![((mv, None), ActionResult::Victory(game))],
            ActionResult::Continue(game) => game
                .active_pawn()
                .actions()
                .map(|build| ((mv, Some(build)), game.apply(build)))
                .collect(),
        })
        .flatten()
        .collect()
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum NodeState {
    Move(Game<Move>),
    Victory(santorini::Player),
}

impl NodeState {
    fn is_victory(&self) -> bool {
        match self {
            NodeState::Move(_) => false,
            NodeState::Victory(_) => true,
        }
    }
}

#[derive(Clone)]
pub struct Node {
    children: Option<Vec<Node>>,
    iterations: u32,
    score: i32,
    mv: Option<MoveAction>,
    build: Option<BuildAction>,
    game: NodeState,
}

impl Node {
    pub fn new(game: Game<Move>) -> Node {
        Node {
            children: None,
            iterations: 1,
            score: simulate(game),
            mv: None,
            build: None,
            game: NodeState::Move(game),
        }
    }

    fn expand(&mut self) -> (u32, i32) {
        assert!(self.children.is_none(), "Node has already been expanded!");

        if let NodeState::Move(game) = self.game {
            let mut children = Vec::new();
            let mut new_scores: i32 = 0;
            let mut new_nodes: u32 = 0;
            for ((mv, build), result) in possible_actions(game) {
                let node_state;
                let score;
                let iterations;
                match result {
                    ActionResult::Victory(won_game) => {
                        node_state = NodeState::Victory(won_game.player());
                        score = 100;
                        iterations = 100;
                    },
                    ActionResult::Continue(game) => {
                        node_state = NodeState::Move(game);
                        score = simulate(game);
                        iterations = 1;
                    },
                }
                let node = Node {
                    children: None,
                    iterations,
                    score,
                    mv: Some(mv),
                    build,
                    game: node_state,
                };
                children.push(node);
                new_scores += -1 * score;
                new_nodes += iterations;
            }

            self.score += new_scores;
            self.iterations += new_nodes;
            self.children = Some(children);

            (new_nodes, new_scores)
        } else {
            panic!("Tried to expand terminal node!");
        }
    }

    fn choose_child(&self) -> usize {
        assert!(self.children.is_some(), "Node hasn't been expanded!");
        let children = self.children.as_ref().unwrap();

        // UCB1 algorithm for choosing a child (multi-arm bandit)
        let mut best_index = None;
        let mut best_weight = None;
        for (index, child) in children.iter().enumerate() {
            let avg_value = (child.score as f64) / (child.iterations as f64);
            // Rescale to be between 0 and 1
            let avg_value = (1.0 + avg_value) / 2.0;

            let augment = 4.0 * f64::ln(self.iterations as f64);
            let augment = augment / (child.iterations as f64);
            let augment = f64::sqrt(augment);

            let weight = avg_value + augment;
            match best_weight {
                None => {
                    best_weight = Some(weight);
                    best_index = Some(index);
                },
                Some(best) => if weight > best {
                    best_weight = Some(weight);
                    best_index = Some(index);
                }
            }
        }

        best_index.expect("No children!")
    }

    pub fn step(&mut self) -> (u32, i32) {
        if self.game.is_victory() {
            return (1, 1);
        }

        match self.children {
            None => self.expand(),
            Some(_) => {
                let idx = self.choose_child();
                let (count, delta) = self.children.as_mut().unwrap()[idx].step();
                self.iterations += count;
                self.score -= delta;
                (count, -delta)
            }
        }
    }
}

static EMPTY: Vec<Point> = Vec::new();

pub struct MCTSAI {
    node: Option<Node>,
}

impl MCTSAI {
    pub fn new() -> Box<dyn FullPlayer> {
        Box::new(MCTSAI {
            node: None,
        })
    }

    pub fn simulate(&mut self, budget: Duration) {
        let mut node = mem::replace(&mut self.node, None).expect("Missing root node!");
        let mut ct = 0;
        let start = Instant::now();
        loop {
            for _ in 0..10 {
                node.step();
            }

            ct += 10;

            let elapsed = Instant::now().duration_since(start);
            if elapsed > budget {
                let mut file = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open("mcts.log")
                    .unwrap();
                writeln!(file, "Ran for {} steps in {:.2} seconds:", ct, elapsed.as_secs_f32());

                let children = node.children.as_ref().expect("Missing children");
                let mut best_score = children[0].score as f64 / children[0].iterations as f64;
                let mut best_score_idx = 0;
                let mut most_visits = children[0].iterations;
                let mut most_visits_idx = 0;

                for (index, child) in children.iter().enumerate() {
                    writeln!(file, "    {}: Visits: {} Score: {} Move: {:?} Build: {:?}", index, child.iterations, child.score, child.mv.map(|ma| ma.to()), child.build.map(|ba| ba.loc()));
                    if child.game.is_victory() {
                        best_score_idx = index;
                        most_visits_idx = index;
                        break;
                    }

                    let score = child.score as f64 / child.iterations as f64;
                    if score > best_score {
                        best_score = score;
                        best_score_idx = index;
                    }

                    if child.iterations > most_visits {
                        most_visits = child.iterations;
                        most_visits_idx = index;
                    }
                }

                if best_score_idx == most_visits_idx {
                    writeln!(file, "Choosing: {}", most_visits_idx);
                    writeln!(file, "");
                    self.node = Some(node.children.unwrap().into_iter().nth(most_visits_idx).unwrap());
                    return;
                } else {
                    writeln!(file, "Most visited: {} Best Score: {}", most_visits_idx, best_score_idx);
                    writeln!(file, "Adding cycles to get convergence...");
                }
            }
        }
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

// TODO: Add support for placement to the tree
fn random_pt() -> Point {
    let mut rng = rand::thread_rng();
    let x: i8 = rng.gen_range(1, santorini::BOARD_WIDTH.0 - 1);
    let y: i8 = rng.gen_range(1, santorini::BOARD_HEIGHT.0 - 1);
    Point::new(x.into(), y.into())
}

impl Player<PlaceOne> for MCTSAI {
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

impl Player<PlaceTwo> for MCTSAI {
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

impl Player<Move> for MCTSAI {
    fn prepare(&mut self, game: &Game<Move>) {
        let current = mem::replace(&mut self.node, None);
        if let Some(node) = current {
            let mut found = false;
            for child in node.children.expect("Unexpanded root node!") {
                if child.game == NodeState::Move(*game) {
                    self.node = Some(child);
                    found = true;
                    break;
                }
            }
            assert!(found, "Tree reset!");
        }

        if self.node.is_none() {
            self.node = Some(Node::new(*game));
        }
    }

    fn render(&self, game: &Game<Move>) -> BoardWidget {
        default_render(game)
    }

    fn step(&mut self, game: &Game<Move>) -> Result<StepResult, UpdateError> {
        if self.node.as_ref().expect("Missing node!").game == NodeState::Move(*game) {
            self.simulate(Duration::from_secs(5));

            // let mut file = std::fs::OpenOptions::new()
            //     .create(true)
            //     .append(true)
            //     .open("mcts.log")
            //     .unwrap();

            //     writeln!(file, "{}: Visits: {} Score: {} Move: {:?} Build: {:?}", index, child.iterations, child.score, child.mv.map(|ma| ma.to()), child.build.map(|ba| ba.loc()));

            // writeln!(file, "Choosing: {}", best_child);
            // writeln!(file, "");

            // writeln!(file, "Chosen: Move: {:?} Build: {:?}", self.node.as_ref().unwrap().mv, self.node.as_ref().unwrap().build);
            // writeln!(file, "");
        }

        let action = self.node.as_ref().expect("Missing node!").mv.expect("Missing move action!");
        match game.clone().apply(action) {
            ActionResult::Continue(game) => Ok(StepResult::Build(game)),
            ActionResult::Victory(game) => Ok(StepResult::Victory(game)),
        }
    }
}

impl Player<Build> for MCTSAI {
    fn prepare(&mut self, _: &Game<Build>) {}

    fn render(&self, game: &Game<Build>) -> BoardWidget {
        default_render(game)
    }

    fn step(&mut self, game: &Game<Build>) -> Result<StepResult, UpdateError> {
        let action = self.node.as_ref().expect("Missing node!").build.expect("Missing build action!");
        match game.clone().apply(action) {
            ActionResult::Continue(game) => Ok(StepResult::Move(game)),
            ActionResult::Victory(game) => Ok(StepResult::Victory(game)),
        }
    }
}
