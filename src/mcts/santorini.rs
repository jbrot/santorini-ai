use super::{Expansion, Simulation};
use crate::santorini::{ActionResult, BuildAction, Game, Move, MoveAction, Player};
use rand::seq::SliceRandom;
use rand::Rng;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum NodeState {
    Move(Game<Move>),
    Victory(Player),
}

#[derive(Clone)]
pub struct SantoriniNode {
    pub mv: Option<MoveAction>,
    pub build: Option<BuildAction>,
    pub game: NodeState,
}

impl From<Game<Move>> for SantoriniNode {
    fn from(game: Game<Move>) -> SantoriniNode {
        SantoriniNode {
            mv: None,
            build: None,
            game: NodeState::Move(game),
        }
    }
}

impl SantoriniNode {
    pub fn matches(&self, game: Game<Move>) -> bool {
        match self.game {
            NodeState::Move(g) => g == game,
            _ => false,
        }
    }
}

pub struct SantoriniSimulation {}

enum PossibleAction {
    Victory,
    Continue(Game<Move>),
}

fn find_action<R: Rng>(game: Game<Move>, rng: &mut R) -> PossibleAction {
    let mut choice = game;
    let mut count = 0.0;
    for mv in game
        .active_pawns()
        .iter()
        .map(|pawn| pawn.actions())
        .flatten()
    {
        match game.apply(mv) {
            ActionResult::Victory(_) => return PossibleAction::Victory,
            ActionResult::Continue(game) => {
                for build in game.active_pawn().actions() {
                    match game.apply(build) {
                        ActionResult::Victory(_) => return PossibleAction::Victory,
                        ActionResult::Continue(game) => {
                            count += 1.0;
                            if rng.gen::<f64>() < 1.0 / count {
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

impl<R: Rng> Simulation<SantoriniNode, R> for SantoriniSimulation {
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
    fn simulate(&self, state: &SantoriniNode, rng: &mut R) -> f64 {
        let mut game = match state.game {
            NodeState::Victory(_) => return 1.0,
            NodeState::Move(game) => game,
        };

        let player = game.player();

        loop {
            match find_action(game, rng) {
                PossibleAction::Victory => return if game.player() == player { -1.0 } else { 1.0 },
                PossibleAction::Continue(choice) => game = choice,
            }
        }
    }
}

pub struct ExtendedSantoriniSimulation {}

fn possible_actions<'a>(
    game: &'a Game<Move>,
) -> impl Iterator<
    Item = (
        (Option<MoveAction>, Option<BuildAction>),
        ActionResult<Move>,
    ),
> + 'a {
    game.active_pawns()
        .to_vec()
        .into_iter()
        .map(|pawn| pawn.actions())
        .flatten()
        .map(move |mv| match game.apply(mv) {
            ActionResult::Victory(game) => vec![((Some(mv), None), ActionResult::Victory(game))],
            ActionResult::Continue(game) => game
                .active_pawn()
                .actions()
                .map(|build| ((Some(mv), Some(build)), game.apply(build)))
                .collect(),
        })
        .flatten()
}

impl<R: Rng> Simulation<SantoriniNode, R> for ExtendedSantoriniSimulation {
    fn simulate(&self, state: &SantoriniNode, rng: &mut R) -> f64 {
        let mut game = match state.game {
            NodeState::Victory(_) => return 1.0,
            NodeState::Move(game) => game,
        };

        let player = game.player();

        let mut previous = game;

        match find_action(game, rng) {
            PossibleAction::Victory => return if game.player() == player { -1.0 } else { 1.0 },
            PossibleAction::Continue(choice) => game = choice,
        }

        loop {
            match find_action(game, rng) {
                PossibleAction::Continue(choice) => {
                    previous = game;
                    game = choice;
                }
                PossibleAction::Victory => {
                    // Back track to see if this could be avoided
                    let mut actions: Vec<_> = possible_actions(&previous).collect();
                    &mut actions.shuffle(rng);
                    let mut found = false;
                    for (_, result) in actions {
                        // We know this can't be a winning move, otherwise we would have
                        // already taken it instead of getting here.
                        let new_game = result.unwrap();
                        match find_action(new_game, rng) {
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
                        return if game.player() == player { -1.0 } else { 1.0 };
                    }
                }
            }
        }
    }
}

pub struct SantoriniExpansion {}

impl Expansion<SantoriniNode> for SantoriniExpansion {
    fn expand(&self, state: &SantoriniNode) -> Vec<SantoriniNode> {
        match state.game {
            NodeState::Victory(_) => vec![],
            NodeState::Move(game) => possible_actions(&game)
                .map(|((mv, build), result)| SantoriniNode {
                    mv,
                    build,
                    game: match result {
                        ActionResult::Victory(game) => NodeState::Victory(game.player()),
                        ActionResult::Continue(game) => NodeState::Move(game),
                    },
                })
                .collect(),
        }
    }
}
