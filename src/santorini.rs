use derive_more::{Add, Display, From};

use std::ops::{Deref, Sub};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Add, Display, From)]
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

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
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
mod point_tests {
    use super::*;

    #[test]
    fn valid_point() {
        Point::new(Coord::from(0), Coord::from(0));
        Point::new(Coord::from(4), Coord::from(4));
        Point::new_(Coord::from(3), Coord::from(1)).unwrap();
        Point::new_(Coord::from(2), Coord::from(0)).unwrap();
    }

    #[test]
    #[should_panic]
    fn negative_point() {
        Point::new(Coord::from(3), Coord::from(-1));
    }

    #[test]
    fn negative_point_() {
        assert_eq!(Point::new_(Coord::from(0), Coord::from(-1)), None);
        assert_eq!(Point::new_(Coord::from(-1), Coord::from(0)), None);
        assert_eq!(Point::new_(Coord::from(-4), Coord::from(-8)), None);
    }

    #[test]
    #[should_panic]
    fn large_point() {
        Point::new(Coord::from(5), Coord::from(2));
    }

    #[test]
    fn large_point_() {
        assert_eq!(Point::new_(Coord::from(5), Coord::from(4)), None);
        assert_eq!(Point::new_(Coord::from(4), Coord::from(5)), None);
        assert_eq!(Point::new_(Coord::from(7), Coord::from(9)), None);
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
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
            CoordLevel::Ground => 0,
            CoordLevel::One => 1,
            CoordLevel::Two => 2,
            CoordLevel::Three => 3,
            CoordLevel::Capped => 4,
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
    fn new() -> Board {
        Board {
            grid: [[CoordLevel::Ground; BOARD_WIDTH.0 as usize]; BOARD_HEIGHT.0 as usize]
        }
    }

    pub fn level_at(&self, loc: Point) -> CoordLevel {
        self.grid[usize::from(loc.x())][usize::from(loc.y())]
    }

    fn build(&mut self, loc: Point)  {
        let c = &mut self.grid[usize::from(loc.x())][usize::from(loc.y())];
        match c {
            CoordLevel::Ground => *c = CoordLevel::One,
            CoordLevel::One => *c = CoordLevel::Two,
            CoordLevel::Two => *c = CoordLevel::Three,
            CoordLevel::Three => *c = CoordLevel::Capped,
            CoordLevel::Capped => panic!["Invalid build action!"]
        }
    }
}

#[cfg(test)]
mod board_tests {
    use super::*;

    #[test]
    fn level_at() {
        let b = Board::new();
        assert_eq!(b.level_at(Point::new(0.into(), 0.into())), CoordLevel::Ground);
        assert_eq!(b.level_at(Point::new(4.into(), 0.into())), CoordLevel::Ground);
        assert_eq!(b.level_at(Point::new(0.into(), 4.into())), CoordLevel::Ground);
        assert_eq!(b.level_at(Point::new(4.into(), 4.into())), CoordLevel::Ground);
        assert_eq!(b.level_at(Point::new(2.into(), 2.into())), CoordLevel::Ground);
    }

    #[test]
    fn build() {
        let pt = Point::new(2.into(), 2.into());
        let mut b = Board::new();

        assert_eq!(b.level_at(pt), CoordLevel::Ground);
        b.build(pt);
        assert_eq!(b.level_at(pt), CoordLevel::One);
        b.build(pt);
        assert_eq!(b.level_at(pt), CoordLevel::Two);
        b.build(pt);
        assert_eq!(b.level_at(pt), CoordLevel::Three);
        b.build(pt);
        assert_eq!(b.level_at(pt), CoordLevel::Capped);
    }

    #[test]
    #[should_panic]
    fn build_over() {
        let pt = Point::new(2.into(), 2.into());
        let mut b = Board::new();

        b.build(pt);
        b.build(pt);
        b.build(pt);
        b.build(pt);
        assert_eq!(b.level_at(pt), CoordLevel::Capped);
        b.build(pt);
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Player {
    PlayerOne,
    PlayerTwo
}

impl Player {
    pub fn other(&self) -> Player  {
        match self {
            Player::PlayerOne => Player::PlayerTwo,
            Player::PlayerTwo => Player::PlayerOne
        }
    }
}

pub trait GameState {}

pub trait NormalState {
    fn player1_locs(&self) -> [Point; 2];
    fn player2_locs(&self) -> [Point; 2];
}

pub struct Game<S: GameState> {
    state: S,
    board: Board,
    player: Player
}

impl<S: GameState + NormalState> Game<S> {
    pub fn new() -> Game<PlaceOne> {
        Game {
            state: PlaceOne {},
            board: Board::new(),
            player: Player::PlayerOne
        }
    }

    pub fn is_open(&self, loc: Point) -> bool {
        if self.board.level_at(loc) == CoordLevel::Capped {
            return false
        }

        for pos in self.state.player1_locs().iter() {
            if *pos == loc {
                return false
            }
        }

        for pos in self.state.player2_locs().iter() {
            if *pos == loc {
                return false
            }
        }

        true
    }

    pub fn player1_pawns(&self) -> [Pawn<S>; 2] {
        // TODO: Use map (currently nightly only)
        let [l1, l2] = self.state.player1_locs();
        [Pawn{game: self, pos: l1, player: Player::PlayerOne}, Pawn{game: self, pos: l2, player: Player::PlayerOne}]
    }

    pub fn player2_pawns(&self) -> [Pawn<S>; 2] {
        // TODO: Use map (currently nightly only)
        let [l1, l2] = self.state.player2_locs();
        [Pawn{game: self, pos: l1, player: Player::PlayerTwo}, Pawn{game: self, pos: l2, player: Player::PlayerTwo}]
    }

    pub fn active_pawns(&self) -> [Pawn<S>; 2] {
        match self.player {
            PlayerOne => self.player1_pawns(),
            PlayerTwo => self.player2_pawns()
        }
    }

    pub fn inactive_pawns(&self) -> [Pawn<S>; 2] {
        match self.player {
            PlayerOne => self.player2_pawns(),
            PlayerTwo => self.player1_pawns()
        }
    }
}

pub struct Pawn<'a, S: GameState> {
    game: &'a Game<S>,
    pos: Point,
    player: Player
}

impl<'a, S: GameState> Pawn<'a, S> {
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

// Moving

pub struct Move {
    player1_locs: [Point; 2],
    player2_locs: [Point; 2],
}
impl GameState for Move {}
impl NormalState for Move {
    fn player1_locs(&self) -> [Point; 2] {
        self.player1_locs
    }

    fn player2_locs(&self) -> [Point; 2] {
        self.player1_locs
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

impl<'a> Pawn<'a, Move> {
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

// We use a macro because we need to write this function for P1 and P2
// with minimal differences
impl Game<Move> {
    pub fn apply(self, action: MoveAction) -> Game<Build> {
        let mut state = Build {
            player1_locs: self.state.player1_locs,
            player2_locs: self.state.player2_locs
        };
        let locs = match self.player {
            Player::PlayerOne => &mut state.player1_locs,
            Player::PlayerTwo => &mut state.player2_locs
        };
        let source = locs.iter_mut().find(|loc| **loc == action.from).expect("Invalid MoveAction");
        *source = action.to;

        Game {
            state,
            board: self.board,
            player: self.player 
        }
    }
}

// Building

pub struct Build {
    player1_locs: [Point; 2],
    player2_locs: [Point; 2],
}
impl GameState for Build {}
impl NormalState for Build {
    fn player1_locs(&self) -> [Point; 2] {
        self.player1_locs
    }

    fn player2_locs(&self) -> [Point; 2] {
        self.player1_locs
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

impl<'a> Pawn<'a, Build> {
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

impl Game<Build> {
    pub fn apply(self, action: BuildAction) -> Game<Move> {
        let mut board = self.board;
        board.build(action.loc);
        Game {
            state: Move {
                player1_locs: self.state.player1_locs,
                player2_locs: self.state.player2_locs,
            },
            board,
            player: self.player.other()
        }
    }
}

// Placement

pub struct PlaceAction {
    pos1: Point,
    pos2: Point
}

impl PlaceAction {
    pub fn pos1(&self) -> Point {
        self.pos1
    }

    pub fn pos2(&self) -> Point {
        self.pos2
    }
}

pub struct PlaceOne {}
impl GameState for PlaceOne {}


impl Game<PlaceOne> {
    pub fn can_place(&self, pos1: Point, pos2: Point) -> Option<PlaceAction> {
        if pos1 != pos2 {
            Some(PlaceAction { pos1, pos2 })
        } else {
            None
        }
    }

    pub fn apply(self, placement: PlaceAction) -> Game<PlaceTwo> {
        Game {
            state: PlaceTwo { player1_locs: [placement.pos1, placement.pos2] },
            board: self.board,
            player: Player::PlayerTwo
        }
    }
}


pub struct PlaceTwo {
    player1_locs: [Point; 2],
}
impl GameState for PlaceTwo {}

impl Game<PlaceTwo> {
    pub fn can_place(&self, pos1: Point, pos2: Point) -> Option<PlaceAction> {
        for pos in self.state.player1_locs.iter() {
            if pos1 == *pos || pos2 == *pos {
                return None
            }
        }

        if pos1 == pos2 {
            Some(PlaceAction { pos1, pos2 })
        } else {
            None
        }
    }

    pub fn apply(self, placement: PlaceAction) -> Game<Move> {
        Game {
            state: Move {
                player1_locs: self.state.player1_locs,
                player2_locs: [placement.pos1, placement.pos2],
            },
            board: self.board,
            player: Player::PlayerOne
        }
    }
}
