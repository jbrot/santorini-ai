use crate::santorini::{Build, Game, GameState, Move, PlaceOne, PlaceTwo, Victory};
use crate::ui::{BoardWidget, UpdateError};

mod human;
mod random_ai;

pub use human::HumanPlayer;
pub use random_ai::RandomAI;

pub enum StepResult {
    NoMove,
    PlaceTwo(Game<PlaceTwo>),
    Move(Game<Move>),
    Build(Game<Build>),
    Victory(Game<Victory>),
}

pub trait Player<T: GameState> {
    fn prepare(&mut self, game: &Game<T>);
    fn render(&self, game: &Game<T>) -> BoardWidget;
    fn step(&mut self, game: &Game<T>) -> Result<StepResult, UpdateError>;
}

pub trait FullPlayer: Player<PlaceOne> + Player<PlaceTwo> + Player<Build> + Player<Move> {}
impl<T> FullPlayer for T where T: Player<PlaceOne> + Player<PlaceTwo> + Player<Build> + Player<Move> {}
