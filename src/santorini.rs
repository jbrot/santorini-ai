use derive_more::{Add, Display, From};

use std::marker::PhantomData;
use std::ops::{Deref, Sub};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Add, Display, From)]
pub struct Coord(pub i8);

impl Deref for Coord {
    type Target = i8;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Coord> for usize {
    fn from(other: Coord) -> usize {
        other.0 as usize
    }
}

#[derive(Eq, PartialEq, Clone, Copy)]
pub struct Point {
    x: Coord,
    y: Coord,
}

const BOARD_WIDTH: Coord = Coord(5);
const BOARD_HEIGHT: Coord = Coord(5);

impl Point {
    pub fn x(&self) -> Coord {
        self.x
    }

    pub fn y(&self) -> Coord {
        self.y
    }

    /// Compute the L0 (taxicab) distance between the points
    pub fn distance(&self, other: Point) -> i8 {
        let dx = (other.x().0 - self.x().0).abs();
        let dy = (other.y().0 - self.y().0).abs();
        i8::max(dx, dy)
    }

    pub fn new(x: Coord, y: Coord) -> Point {
        match Point::new_(x, y) {
            Some(p) => p,
            None => panic!(
                "A valid point must lie between (0, 0) and ({}, {})",
                BOARD_WIDTH, BOARD_HEIGHT
            ),
        }
    }

    /// Creates a new point, returning None if the given coordinates are out of bound.
    ///
    /// An alternate to Point::new which panics on out of bounds.
    ///
    /// Not sure what the naming convention is here.
    pub fn new_(x: Coord, y: Coord) -> Option<Point> {
        if x >= BOARD_WIDTH || x < Coord::from(0) || y >= BOARD_HEIGHT || y < Coord::from(0) {
            None
        } else {
            Some(Point { x, y })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_point() {
        Point::new(Coord::from(3), Coord::from(4));
    }

    #[test]
    #[should_panic]
    fn negative_point() {
        Point::new(Coord::from(3), Coord::from(-1));
    }

    #[test]
    #[should_panic]
    fn large_point() {
        Point::new(Coord::from(5), Coord::from(2));
    }
}

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum CoordLevel {
    Ground,
    One,
    Two,
    Three,
    Capped,
}

impl CoordLevel {
    fn height(self) -> i8 {
        match self {
            Ground => 0,
            One => 1,
            Two => 2,
            Three => 3,
            Capped => 4,
        }
    }
}

impl Sub for CoordLevel {
    type Output = i8;

    fn sub(self, other: Self) -> Self::Output {
        self.height() - other.height()
    }
}

struct Board {
    pub grid: [[CoordLevel; BOARD_WIDTH.0 as usize]; BOARD_HEIGHT.0 as usize],
}

impl Board {
    fn level_at(&self, loc: Point) -> CoordLevel {
        self.grid[usize::from(loc.x())][usize::from(loc.y())]
    }
}

pub trait Player {}

pub struct PlayerOne {}
impl Player for PlayerOne {}

pub struct PlayerTwo {}
impl Player for PlayerTwo {}

pub trait GameState<T: Player> {}
pub struct Place<T: Player> {
    phantom: PhantomData<T>,
}
impl<T: Player> GameState<T> for Place<T> {}
pub struct Move<T: Player> {
    phantom: PhantomData<T>,
}
impl<T: Player> GameState<T> for Move<T> {}
pub struct Build<T: Player> {
    phantom: PhantomData<T>,
}
impl<T: Player> GameState<T> for Build<T> {}

pub struct Game<P: Player, S: GameState<P>> {
    state: S,
    board: Board,
    phantom: PhantomData<P>,
}

impl<P: Player, S: GameState<P>> Game<P, S> {
    pub fn is_open(&self, loc: Point) -> bool {
        // TODO: Check player positions
        self.board.level_at(loc) != CoordLevel::Capped
    }
}

pub struct Pawn<'a, P, C, S>
where
    P: Player,
    C: Player,
    S: GameState<C>,
{
    game: &'a Game<C, S>,
    pos: Point,

    phantom: PhantomData<P>,
}

impl<'a, P: Player, C: Player, S: GameState<C>> Pawn<'a, P, C, S> {
    pub fn pos(&self) -> Point {
        self.pos
    }

    pub fn neighbors(&self) -> Vec<Point> {
        let offsets = [
            (-1, -1),
            (0, -1),
            (1, -1),
            (-1, 0),
            (1, 0),
            (-1, 1),
            (0, 1),
            (1, 1),
        ];
        offsets
            .iter()
            .filter_map(|(x, y)| {
                Point::new_(
                    self.pos.x() + Coord::from(*x),
                    self.pos.y() + Coord::from(*y),
                )
            })
            .collect()
    }
}

pub struct MoveAction {
    from: Point,
    to: Point,
}

impl MoveAction {
    pub fn from(&self) -> Point {
        self.from
    }

    pub fn to(&self) -> Point {
        self.to
    }
} 

impl<'a, P: Player> Pawn<'a, P, P, Move<P>> {
    pub fn can_move(&self, to: Point) -> Option<MoveAction> {
        if self.pos.distance(to) == 1 && self.game.is_open(to) {
            Some(MoveAction { from: self.pos, to })
        } else {
            None
        }
    }

    pub fn actions(&self) -> Vec<MoveAction> {
        self.neighbors()
            .iter()
            .filter_map(|&to| self.can_move(to))
            .collect()
    }
}

pub struct BuildAction {
    loc: Point,
}

impl BuildAction {
    pub fn loc(&self) -> Point {
        self.loc
    }
}

impl<'a, P: Player> Pawn<'a, P, P, Place<P>> {
    pub fn can_build(&self, loc: Point) -> Option<BuildAction> {
        if self.pos.distance(loc) == 1 && self.game.is_open(loc) {
            Some(BuildAction { loc })
        } else {
            None
        }
    }

    pub fn actions(&self) -> Vec<BuildAction> {
        self.neighbors()
            .iter()
            .filter_map(|&loc| self.can_build(loc))
            .collect()
    }
}

//struct Player<'a> {
//    loc: Point,
//    board: &'a Board
//}
//
//impl<'a> Player<'a> {
//    fn can_move(&self, loc: Point) -> bool {
//        self.loc.distance(loc) == 1 && self.board.is_open(loc) && self.board.level_at(loc) - self.board.level_at(self.loc) <= 1
//    }
//
//    fn can_build(&self, loc: Point) -> bool {
//        self.loc.distance(loc) == 1 && self.board.is_open(loc)
//    }
//}
