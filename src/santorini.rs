use derive_more::{Add, Display, From};

use std::ops::{Deref, Sub};
use std::slice::Iter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Add, Display, From)]
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

#[derive(Debug, Eq, PartialEq, Clone, Copy, Hash)]
pub struct Point {
    x: Coord,
    y: Coord,
}

pub const BOARD_WIDTH: Coord = Coord(5);
pub const BOARD_HEIGHT: Coord = Coord(5);

impl Point {
    pub fn x(&self) -> Coord {
        self.x
    }

    pub fn y(&self) -> Coord {
        self.y
    }

    /// Compute the L\infty (supremum) distance between the points
    pub fn distance(&self, other: Point) -> i8 {
        let dx = (other.x().0 - self.x().0).abs();
        let dy = (other.y().0 - self.y().0).abs();
        i8::max(dx, dy)
    }

    /// Compute the L0 (taxicab) distance between the points
    pub fn taxicab(&self, other: Point) -> i8 {
        (other.x().0 - self.x().0).abs() + (other.y().0 - self.y().0).abs()
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

#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
pub enum CoordLevel {
    Ground,
    One,
    Two,
    Three,
    Capped,
}

impl CoordLevel {
    pub fn height(self) -> i8 {
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

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Board {
    pub grid: [[CoordLevel; BOARD_WIDTH.0 as usize]; BOARD_HEIGHT.0 as usize],
}

impl Board {
    fn new() -> Board {
        Board {
            grid: [[CoordLevel::Ground; BOARD_WIDTH.0 as usize]; BOARD_HEIGHT.0 as usize],
        }
    }

    pub fn level_at(&self, loc: Point) -> CoordLevel {
        self.grid[usize::from(loc.x())][usize::from(loc.y())]
    }

    fn build(&mut self, loc: Point) {
        let c = &mut self.grid[usize::from(loc.x())][usize::from(loc.y())];
        match c {
            CoordLevel::Ground => *c = CoordLevel::One,
            CoordLevel::One => *c = CoordLevel::Two,
            CoordLevel::Two => *c = CoordLevel::Three,
            CoordLevel::Three => *c = CoordLevel::Capped,
            CoordLevel::Capped => panic!["Invalid build action!"],
        }
    }
}

#[cfg(test)]
mod board_tests {
    use super::*;

    #[test]
    fn level_at() {
        let b = Board::new();
        assert_eq!(
            b.level_at(Point::new(0.into(), 0.into())),
            CoordLevel::Ground
        );
        assert_eq!(
            b.level_at(Point::new(4.into(), 0.into())),
            CoordLevel::Ground
        );
        assert_eq!(
            b.level_at(Point::new(0.into(), 4.into())),
            CoordLevel::Ground
        );
        assert_eq!(
            b.level_at(Point::new(4.into(), 4.into())),
            CoordLevel::Ground
        );
        assert_eq!(
            b.level_at(Point::new(2.into(), 2.into())),
            CoordLevel::Ground
        );
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

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum Player {
    PlayerOne,
    PlayerTwo,
}

impl Player {
    pub fn other(&self) -> Player {
        match self {
            Player::PlayerOne => Player::PlayerTwo,
            Player::PlayerTwo => Player::PlayerOne,
        }
    }

    pub fn iter() -> Iter<'static, Player> {
        static PLAYERS: [Player; 2] = [Player::PlayerOne, Player::PlayerTwo];
        PLAYERS.iter()
    }
}

pub trait GameState {}

pub trait NormalState {
    fn player_locs(&self, player: Player) -> [Point; 2];
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Game<S: GameState> {
    state: S,
    board: Board,
    player: Player,
}

impl<S: GameState> Game<S> {
    pub fn board(&self) -> Board {
        self.board
    }

    pub fn player(&self) -> Player {
        self.player
    }
}

pub fn new_game() -> Game<PlaceOne> {
    Game {
        state: PlaceOne {},
        board: Board::new(),
        player: Player::PlayerOne,
    }
}

impl<S: GameState + NormalState> Game<S> {
    pub fn is_open(&self, loc: Point) -> bool {
        if self.board.level_at(loc) == CoordLevel::Capped {
            return false;
        }

        for player in Player::iter() {
            for pos in self.state.player_locs(*player).iter() {
                if *pos == loc {
                    return false;
                }
            }
        }

        true
    }

    pub fn player_pawns(&self, player: Player) -> [Pawn<S>; 2] {
        // TODO: Use map (currently nightly only)
        let [l1, l2] = self.state.player_locs(player);
        [
            Pawn {
                game: self,
                pos: l1,
                player,
            },
            Pawn {
                game: self,
                pos: l2,
                player,
            },
        ]
    }

    pub fn active_pawns(&self) -> [Pawn<S>; 2] {
        self.player_pawns(self.player)
    }

    pub fn inactive_pawns(&self) -> [Pawn<S>; 2] {
        self.player_pawns(self.player.other())
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Pawn<'a, S: GameState> {
    game: &'a Game<S>,
    pos: Point,
    player: Player,
}

impl<'a, S: GameState> Pawn<'a, S> {
    pub fn pos(&self) -> Point {
        self.pos
    }

    pub fn player(&self) -> Player {
        self.player
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

// Victory

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Victory {
    player1_locs: [Point; 2],
    player2_locs: [Point; 2],
}
impl GameState for Victory {}
impl NormalState for Victory {
    fn player_locs(&self, player: Player) -> [Point; 2] {
        match player {
            Player::PlayerOne => self.player1_locs,
            Player::PlayerTwo => self.player2_locs,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum ActionResult<T: GameState> {
    Continue(Game<T>),
    Victory(Game<Victory>),
}

impl<T: GameState> ActionResult<T> {
    pub fn unwrap(self) -> Game<T> {
        match self {
            ActionResult::Continue(g) => g,
            ActionResult::Victory(_) => panic!("Unexpected game termination!"),
        }
    }

    pub fn expect(self, msg: &str) -> Game<T> {
        match self {
            ActionResult::Continue(g) => g,
            ActionResult::Victory(_) => panic!("{}", msg),
        }
    }
}

// Moving

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Move {
    player1_locs: [Point; 2],
    player2_locs: [Point; 2],
}
impl GameState for Move {}
impl NormalState for Move {
    fn player_locs(&self, player: Player) -> [Point; 2] {
        match player {
            Player::PlayerOne => self.player1_locs,
            Player::PlayerTwo => self.player2_locs,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct MoveAction {
    from: Point,
    to: Point,
    game: Game<Move>,
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
        if self.player != self.game.player {
            return None;
        }

        if self.pos.distance(to) != 1 {
            return None;
        }

        let board = &self.game.board;
        if board.level_at(to).height() - board.level_at(self.pos).height() > 1 {
            return None;
        }

        if !self.game.is_open(to) {
            return None;
        }

        Some(MoveAction {
            from: self.pos,
            to,
            game: self.game.clone(),
        })
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
    pub fn apply(self, action: MoveAction) -> ActionResult<Build> {
        if action.game != self {
            panic!(
                "Game {:?} received action {:?} associated with a different game!",
                self, action
            );
        }

        let mut state = Build {
            player1_locs: self.state.player1_locs,
            player2_locs: self.state.player2_locs,
            active_loc: action.to,
        };
        let locs = match self.player {
            Player::PlayerOne => &mut state.player1_locs,
            Player::PlayerTwo => &mut state.player2_locs,
        };
        let source = locs
            .iter_mut()
            .find(|loc| **loc == action.from)
            .expect("Invalid MoveAction");
        *source = action.to;

        if self.board.level_at(action.to) == CoordLevel::Three {
            ActionResult::Victory(Game {
                state: Victory {
                    player1_locs: state.player1_locs,
                    player2_locs: state.player2_locs,
                },
                board: self.board,
                player: self.player,
            })
        } else {
            ActionResult::Continue(Game {
                state,
                board: self.board,
                player: self.player,
            })
        }
    }
}

// Building

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Build {
    player1_locs: [Point; 2],
    player2_locs: [Point; 2],

    active_loc: Point,
}
impl GameState for Build {}
impl NormalState for Build {
    fn player_locs(&self, player: Player) -> [Point; 2] {
        match player {
            Player::PlayerOne => self.player1_locs,
            Player::PlayerTwo => self.player2_locs,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BuildAction {
    loc: Point,
    game: Game<Build>,
}

impl BuildAction {
    pub fn loc(&self) -> Point {
        self.loc
    }
}

impl<'a> Pawn<'a, Build> {
    pub fn can_build(&self, loc: Point) -> Option<BuildAction> {
        if self.pos == self.game.state.active_loc
            && self.pos.distance(loc) == 1
            && self.game.is_open(loc)
        {
            Some(BuildAction {
                loc,
                game: self.game.clone(),
            })
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
    pub fn active_pawn(&self) -> Pawn<Build> {
        Pawn {
            game: self,
            pos: self.state.active_loc,
            player: self.player,
        }
    }

    pub fn apply(self, action: BuildAction) -> ActionResult<Move> {
        if action.game != self {
            panic!(
                "Game {:?} received action {:?} associated with a different game!",
                self, action
            );
        }

        let mut board = self.board;
        board.build(action.loc);
        let new_game = Game {
            state: Move {
                player1_locs: self.state.player1_locs,
                player2_locs: self.state.player2_locs,
            },
            board,
            player: self.player.other(),
        };

        // Note that after a move, there is always at least one valid build
        // location (the place the pawn moved from), so we just need to check
        // moves and not builds to determine a stalemate.
        if new_game
            .active_pawns()
            .iter()
            .find(|pawn| pawn.actions().len() > 0)
            .is_some()
        {
            ActionResult::Continue(new_game)
        } else {
            // New player can't move so the current player wins!
            ActionResult::Victory(Game {
                state: Victory {
                    player1_locs: new_game.state.player1_locs,
                    player2_locs: new_game.state.player2_locs,
                },
                board: new_game.board,
                player: self.player,
            })
        }
    }
}

// Placement

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PlaceAction<T: GameState> {
    pos1: Point,
    pos2: Point,
    game: Game<T>,
}

impl<T: GameState> PlaceAction<T> {
    pub fn pos1(&self) -> Point {
        self.pos1
    }

    pub fn pos2(&self) -> Point {
        self.pos2
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct PlaceOne {}
impl GameState for PlaceOne {}

impl Game<PlaceOne> {
    pub fn can_place(&self, pos1: Point, pos2: Point) -> Option<PlaceAction<PlaceOne>> {
        if pos1 != pos2 {
            Some(PlaceAction {
                pos1,
                pos2,
                game: self.clone(),
            })
        } else {
            None
        }
    }

    pub fn apply(self, placement: PlaceAction<PlaceOne>) -> Game<PlaceTwo> {
        if placement.game != self {
            panic!(
                "Game {:?} received action {:?} associated with a different game!",
                self, placement
            );
        }

        Game {
            state: PlaceTwo {
                player1_locs: [placement.pos1, placement.pos2],
            },
            board: self.board,
            player: Player::PlayerTwo,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct PlaceTwo {
    player1_locs: [Point; 2],
}
impl GameState for PlaceTwo {}

impl Game<PlaceTwo> {
    pub fn player1_locs(&self) -> [Point; 2] {
        self.state.player1_locs
    }

    pub fn can_place(&self, pos1: Point, pos2: Point) -> Option<PlaceAction<PlaceTwo>> {
        for pos in self.state.player1_locs.iter() {
            if pos1 == *pos || pos2 == *pos {
                return None;
            }
        }

        if pos1 != pos2 {
            Some(PlaceAction {
                pos1,
                pos2,
                game: self.clone(),
            })
        } else {
            None
        }
    }

    pub fn apply(self, placement: PlaceAction<PlaceTwo>) -> Game<Move> {
        if placement.game != self {
            panic!(
                "Game {:?} received action {:?} associated with a different game!",
                self, placement
            );
        }

        Game {
            state: Move {
                player1_locs: self.state.player1_locs,
                player2_locs: [placement.pos1, placement.pos2],
            },
            board: self.board,
            player: Player::PlayerOne,
        }
    }
}

#[cfg(test)]
mod game_tests {
    use super::*;

    #[test]
    fn place_one() {
        let g = new_game();
        assert_eq!(Player::PlayerOne, g.player());

        let pt1 = Point::new(0.into(), 0.into());
        assert_eq!(None, g.can_place(pt1, pt1));

        let pt2 = Point::new(1.into(), 1.into());
        assert_ne!(None, g.can_place(pt1, pt2));
    }

    #[test]
    fn place_two() {
        let g = new_game();
        let pt1 = Point::new(0.into(), 3.into());
        let pt2 = Point::new(1.into(), 2.into());
        let pt3 = Point::new(2.into(), 1.into());
        let pt4 = Point::new(3.into(), 0.into());

        let placement = g.can_place(pt1, pt2).expect("Invalid placement!");
        let g = g.apply(placement);
        assert_eq!(Player::PlayerTwo, g.player());

        assert_eq!(None, g.can_place(pt3, pt3));
        assert_eq!(None, g.can_place(pt1, pt3));
        assert_eq!(None, g.can_place(pt3, pt2));
        assert_ne!(None, g.can_place(pt3, pt4));
    }

    #[test]
    fn pawn_reporting() {
        let g = new_game();
        let pt1 = Point::new(0.into(), 0.into());
        let pt2 = Point::new(4.into(), 4.into());
        let pt3 = Point::new(2.into(), 4.into());
        let pt4 = Point::new(3.into(), 1.into());

        let action = g.can_place(pt1, pt2).expect("Invalid placement!");
        let g = g.apply(action);
        let action = g.can_place(pt3, pt4).expect("Invalid placement!");
        let g = g.apply(action);
        assert_eq!(Player::PlayerOne, g.player());

        let [pawn1, pawn2] = g.active_pawns();
        assert_eq!(pawn1.pos(), pt1);
        assert_eq!(pawn2.pos(), pt2);
        let [pawn3, pawn4] = g.inactive_pawns();
        assert_eq!(pawn3.pos(), pt3);
        assert_eq!(pawn4.pos(), pt4);

        let [pawn1, pawn2] = g.active_pawns();
        assert_eq!(pawn1.pos(), pt1);
        assert_eq!(pawn2.pos(), pt2);
        let [pawn3, pawn4] = g.inactive_pawns();
        assert_eq!(pawn3.pos(), pt3);
        assert_eq!(pawn4.pos(), pt4);

        let pt5 = Point::new(0.into(), 1.into());
        let action = pawn1.can_move(pt5).expect("Invalid move!");
        let g = g.apply(action).expect("Invalid victory!");
        assert_eq!(Player::PlayerOne, g.player());

        let [pawn1, pawn2] = g.active_pawns();
        assert_eq!(pawn1.pos(), pt5);
        assert_eq!(pawn2.pos(), pt2);
        let [pawn3, pawn4] = g.inactive_pawns();
        assert_eq!(pawn3.pos(), pt3);
        assert_eq!(pawn4.pos(), pt4);

        let [pawn1, pawn2] = g.active_pawns();
        assert_eq!(pawn1.pos(), pt5);
        assert_eq!(pawn2.pos(), pt2);
        let [pawn3, pawn4] = g.inactive_pawns();
        assert_eq!(pawn3.pos(), pt3);
        assert_eq!(pawn4.pos(), pt4);

        let action = pawn1.can_build(pt1).expect("Invalid build!");
        let g = g.apply(action).expect("Invalid victory!");
        assert_eq!(Player::PlayerTwo, g.player());

        let [pawn1, pawn2] = g.inactive_pawns();
        assert_eq!(pawn1.pos(), pt5);
        assert_eq!(pawn2.pos(), pt2);
        let [pawn3, pawn4] = g.active_pawns();
        assert_eq!(pawn3.pos(), pt3);
        assert_eq!(pawn4.pos(), pt4);

        let [pawn3, pawn4] = g.active_pawns();
        assert_eq!(pawn3.pos(), pt3);
        assert_eq!(pawn4.pos(), pt4);
        let [pawn1, pawn2] = g.inactive_pawns();
        assert_eq!(pawn1.pos(), pt5);
        assert_eq!(pawn2.pos(), pt2);
    }

    #[test]
    fn neighbors() {
        let g = new_game();
        let pt1 = Point::new(0.into(), 0.into());
        let pt2 = Point::new(4.into(), 4.into());
        let pt3 = Point::new(2.into(), 4.into());
        let pt4 = Point::new(3.into(), 1.into());

        let action = g.can_place(pt1, pt2).expect("Invalid placement!");
        let g = g.apply(action);
        let action = g.can_place(pt3, pt4).expect("Invalid placement!");
        let g = g.apply(action);

        let [pawn1, pawn2] = g.active_pawns();
        let [pawn3, pawn4] = g.inactive_pawns();

        let neighbors1 = [
            Point::new(1.into(), 0.into()),
            Point::new(0.into(), 1.into()),
            Point::new(1.into(), 1.into()),
        ];
        let neighbors2 = [
            Point::new(3.into(), 3.into()),
            Point::new(4.into(), 3.into()),
            Point::new(3.into(), 4.into()),
        ];
        let neighbors3 = [
            Point::new(1.into(), 3.into()),
            Point::new(2.into(), 3.into()),
            Point::new(3.into(), 3.into()),
            Point::new(1.into(), 4.into()),
            Point::new(3.into(), 4.into()),
        ];
        let neighbors4 = [
            Point::new(2.into(), 0.into()),
            Point::new(3.into(), 0.into()),
            Point::new(4.into(), 0.into()),
            Point::new(2.into(), 1.into()),
            Point::new(4.into(), 1.into()),
            Point::new(2.into(), 2.into()),
            Point::new(3.into(), 2.into()),
            Point::new(4.into(), 2.into()),
        ];

        assert_eq!(pawn1.neighbors(), neighbors1);
        assert_eq!(pawn2.neighbors(), neighbors2);
        assert_eq!(pawn3.neighbors(), neighbors3);
        assert_eq!(pawn4.neighbors(), neighbors4);
    }

    #[test]
    fn move_actions() {
        let g = new_game();
        let pt1 = Point::new(0.into(), 0.into());
        let pt2 = Point::new(3.into(), 1.into());
        let pt3 = Point::new(4.into(), 4.into());
        let pt4 = Point::new(2.into(), 4.into());

        let action = g.can_place(pt1, pt2).expect("Invalid placement!");
        let g = g.apply(action);
        let action = g.can_place(pt3, pt4).expect("Invalid placement!");
        let g = g.apply(action);

        let [pawn1, pawn2] = g.active_pawns();
        let [pawn3, pawn4] = g.inactive_pawns();

        let moves1 = [
            MoveAction {
                from: pt1,
                to: Point::new(1.into(), 0.into()),
                game: g.clone(),
            },
            MoveAction {
                from: pt1,
                to: Point::new(0.into(), 1.into()),
                game: g.clone(),
            },
            MoveAction {
                from: pt1,
                to: Point::new(1.into(), 1.into()),
                game: g.clone(),
            },
        ];
        let moves2 = [
            MoveAction {
                from: pt2,
                to: Point::new(2.into(), 0.into()),
                game: g.clone(),
            },
            MoveAction {
                from: pt2,
                to: Point::new(3.into(), 0.into()),
                game: g.clone(),
            },
            MoveAction {
                from: pt2,
                to: Point::new(4.into(), 0.into()),
                game: g.clone(),
            },
            MoveAction {
                from: pt2,
                to: Point::new(2.into(), 1.into()),
                game: g.clone(),
            },
            MoveAction {
                from: pt2,
                to: Point::new(4.into(), 1.into()),
                game: g.clone(),
            },
            MoveAction {
                from: pt2,
                to: Point::new(2.into(), 2.into()),
                game: g.clone(),
            },
            MoveAction {
                from: pt2,
                to: Point::new(3.into(), 2.into()),
                game: g.clone(),
            },
            MoveAction {
                from: pt2,
                to: Point::new(4.into(), 2.into()),
                game: g.clone(),
            },
        ];

        assert_eq!(pawn1.actions(), moves1);
        assert_eq!(pawn2.actions(), moves2);
        assert_eq!(pawn3.actions(), []);
        assert_eq!(pawn4.actions(), []);
    }

    #[test]
    fn build_actions() {
        let g = new_game();
        let pt1 = Point::new(0.into(), 0.into());
        let pt2 = Point::new(3.into(), 1.into());
        let pt3 = Point::new(4.into(), 4.into());
        let pt4 = Point::new(2.into(), 4.into());

        let action = g.can_place(pt1, pt2).expect("Invalid placement!");
        let g = g.apply(action);
        let action = g.can_place(pt3, pt4).expect("Invalid placement!");
        let g = g.apply(action);

        let pt1a = Point::new(0.into(), 1.into());
        let [pawn1, _] = g.active_pawns();
        let action = pawn1.can_move(pt1a).expect("Invalid move!");
        let g = g.apply(action).expect("Invalid victory!");

        let [pawn1, pawn2] = g.active_pawns();
        let [pawn3, pawn4] = g.inactive_pawns();

        let build1 = [
            BuildAction {
                loc: Point::new(0.into(), 0.into()),
                game: g.clone(),
            },
            BuildAction {
                loc: Point::new(1.into(), 0.into()),
                game: g.clone(),
            },
            BuildAction {
                loc: Point::new(1.into(), 1.into()),
                game: g.clone(),
            },
            BuildAction {
                loc: Point::new(0.into(), 2.into()),
                game: g.clone(),
            },
            BuildAction {
                loc: Point::new(1.into(), 2.into()),
                game: g.clone(),
            },
        ];

        assert_eq!(pawn1.actions(), build1);
        assert_eq!(pawn2.actions(), []);
        assert_eq!(pawn3.actions(), []);
        assert_eq!(pawn4.actions(), []);
    }

    #[test]
    fn can_move() {
        let g = new_game();
        let pt1 = Point::new(1.into(), 1.into());
        let pt2 = Point::new(2.into(), 2.into());
        let pt3 = Point::new(2.into(), 1.into());
        let pt4 = Point::new(1.into(), 2.into());

        let action = g.can_place(pt1, pt2).expect("Invalid placement!");
        let g = g.apply(action);
        let action = g.can_place(pt3, pt4).expect("Invalid placement!");
        let g = g.apply(action);

        let [pawn1, pawn2] = g.active_pawns();
        let [pawn3, _] = g.inactive_pawns();

        assert_eq!(None, pawn1.can_move(pt2));
        assert_eq!(None, pawn1.can_move(pt3));
        assert_eq!(None, pawn1.can_move(pt4));
        assert_eq!(None, pawn1.can_move(Point::new(0.into(), 3.into())));
        assert_ne!(None, pawn1.can_move(Point::new(0.into(), 2.into())));

        assert_ne!(None, pawn2.can_move(Point::new(2.into(), 3.into())));
        assert_eq!(None, pawn2.can_move(pt3));

        assert_eq!(None, pawn3.can_move(Point::new(3.into(), 1.into())));
    }

    #[test]
    fn can_build() {
        let g = new_game();
        let pt1 = Point::new(1.into(), 1.into());
        let pt2 = Point::new(2.into(), 2.into());
        let pt3 = Point::new(2.into(), 1.into());
        let pt4 = Point::new(1.into(), 2.into());

        let action = g.can_place(pt1, pt2).expect("Invalid placement!");
        let g = g.apply(action);
        let action = g.can_place(pt3, pt4).expect("Invalid placement!");
        let g = g.apply(action);

        let pt1a = Point::new(1.into(), 0.into());
        let [pawn1, _] = g.active_pawns();
        let action = pawn1.can_move(pt1a).expect("Invalid movement!");
        let g = g.apply(action).expect("Invalid victory!");

        let [pawn1, pawn2] = g.active_pawns();
        let [pawn3, _] = g.inactive_pawns();

        assert_eq!(pawn1, g.active_pawn());

        assert_eq!(None, pawn1.can_build(pt3));
        assert_ne!(None, pawn1.can_build(pt1));

        assert_eq!(None, pawn2.can_build(pt1));
        assert_eq!(None, pawn3.can_build(pt1));
    }

    #[test]
    fn can_move_height() {
        let g = new_game();
        let pt1 = Point::new(1.into(), 1.into());
        let pt2 = Point::new(2.into(), 2.into());
        let pt3 = Point::new(2.into(), 1.into());
        let pt4 = Point::new(1.into(), 2.into());

        let action = g.can_place(pt1, pt2).expect("Invalid placement!");
        let g = g.apply(action);
        let action = g.can_place(pt3, pt4).expect("Invalid placement!");
        let g = g.apply(action);

        // [0  ][0  ][0  ][0  ][0  ]
        // [0  ][0P1][0P3][0  ][0  ]
        // [0  ][0P4][0P2][0  ][0  ]
        // [0  ][0  ][0  ][0  ][0  ]
        // [0  ][0  ][0  ][0  ][0  ]

        let pt1a = Point::new(1.into(), 0.into());
        let [pawn1, _] = g.active_pawns();
        let action = pawn1.can_move(pt1a).expect("Invalid movement!");
        let g = g.apply(action).expect("Invalid victory!");

        let pawn1 = g.active_pawn();
        let action = pawn1.can_build(pt1).expect("Invalid build");
        let g = g.apply(action).expect("Invalid victory!");

        // [0  ][0P1][0  ][0  ][0  ]
        // [0  ][1  ][0P3][0  ][0  ]
        // [0  ][0P4][0P2][0  ][0  ]
        // [0  ][0  ][0  ][0  ][0  ]
        // [0  ][0  ][0  ][0  ][0  ]

        let pt3a = Point::new(2.into(), 0.into());
        let [pawn3, _] = g.active_pawns();
        let action = pawn3.can_move(pt3a).expect("Invalid movement!");
        let g = g.apply(action).expect("Invalid victory!");

        let pawn3 = g.active_pawn();
        let action = pawn3.can_build(pt1).expect("Invalid build");
        let g = g.apply(action).expect("Invalid victory!");

        // [0  ][0P1][0P3][0  ][0  ]
        // [0  ][2  ][0  ][0  ][0  ]
        // [0  ][0P4][0P2][0  ][0  ]
        // [0  ][0  ][0  ][0  ][0  ]
        // [0  ][0  ][0  ][0  ][0  ]

        let [pawn1, _] = g.active_pawns();
        assert_eq!(None, pawn1.can_move(pt1));

        let pt1b = Point::new(0.into(), 0.into());
        let action = pawn1.can_move(pt1b).expect("Invalid movement!");
        let g = g.apply(action).expect("Invalid victory!");

        let pawn1 = g.active_pawn();
        let action = pawn1.can_build(pt1a).expect("Invalid build");
        let g = g.apply(action).expect("Invalid victory!");

        // [0P1][1  ][0P3][0  ][0  ]
        // [0  ][2  ][0  ][0  ][0  ]
        // [0  ][0P4][0P2][0  ][0  ]
        // [0  ][0  ][0  ][0  ][0  ]
        // [0  ][0  ][0  ][0  ][0  ]

        let [pawn3, _] = g.active_pawns();
        let action = pawn3.can_move(pt1a).expect("Invalid movement!");
        let g = g.apply(action).expect("Invalid victory!");

        let pt1c = Point::new(0.into(), 1.into());
        let pawn3 = g.active_pawn();
        let action = pawn3.can_build(pt1c).expect("Invalid build");
        let g = g.apply(action).expect("Invalid victory!");

        // [0P1][1P3][0  ][0  ][0  ]
        // [1  ][2  ][0  ][0  ][0  ]
        // [0  ][0P4][0P2][0  ][0  ]
        // [0  ][0  ][0  ][0  ][0  ]
        // [0  ][0  ][0  ][0  ][0  ]

        let [pawn1, _] = g.active_pawns();
        let action = pawn1.can_move(pt1c).expect("Invalid movement!");
        let g = g.apply(action).expect("Invalid victory!");

        let pawn1 = g.active_pawn();
        let action = pawn1.can_build(pt1b).expect("Invalid build");
        let g = g.apply(action).expect("Invalid victory!");

        // [1  ][1P3][0  ][0  ][0  ]
        // [1P1][2  ][0  ][0  ][0  ]
        // [0  ][0P4][0P2][0  ][0  ]
        // [0  ][0  ][0  ][0  ][0  ]
        // [0  ][0  ][0  ][0  ][0  ]

        let [pawn3, _] = g.active_pawns();
        let action = pawn3.can_move(pt1).expect("Invalid movement!");
        let g = g.apply(action).expect("Invalid victory!");

        let pawn3 = g.active_pawn();
        let action = pawn3.can_build(pt1b).expect("Invalid build");
        let g = g.apply(action).expect("Invalid victory!");

        // [2  ][1  ][0  ][0  ][0  ]
        // [1P1][2P3][0  ][0  ][0  ]
        // [0  ][0P4][0P2][0  ][0  ]
        // [0  ][0  ][0  ][0  ][0  ]
        // [0  ][0  ][0  ][0  ][0  ]

        let [pawn1, _] = g.active_pawns();
        let action = pawn1.can_move(pt1a).expect("Invalid movement!");
        let g = g.apply(action).expect("Invalid victory!");

        let pawn1 = g.active_pawn();
        let action = pawn1.can_build(pt1b).expect("Invalid build");
        let g = g.apply(action).expect("Invalid victory!");

        // [3  ][1P1][0  ][0  ][0  ]
        // [1  ][2P3][0  ][0  ][0  ]
        // [0  ][0P4][0P2][0  ][0  ]
        // [0  ][0  ][0  ][0  ][0  ]
        // [0  ][0  ][0  ][0  ][0  ]

        let [pawn3, _] = g.active_pawns();
        assert_ne!(None, pawn3.can_move(pt1b));
        assert_eq!(None, pawn3.can_move(pt1a));
        assert_ne!(None, pawn3.can_move(pt3a));
        assert_ne!(None, pawn3.can_move(pt1c));
        assert_eq!(None, pawn3.can_move(pt1));
        assert_ne!(None, pawn3.can_move(pt3));
        assert_ne!(None, pawn3.can_move(Point::new(0.into(), 2.into())));
        assert_eq!(None, pawn3.can_move(pt4));
        assert_eq!(None, pawn3.can_move(pt2));
    }

    #[test]
    fn can_build_capped() {
        let g = new_game();
        let pt1 = Point::new(1.into(), 1.into());
        let pt2 = Point::new(2.into(), 2.into());
        let pt3 = Point::new(2.into(), 1.into());
        let pt4 = Point::new(1.into(), 2.into());

        let action = g.can_place(pt1, pt2).expect("Invalid placement!");
        let g = g.apply(action);
        let action = g.can_place(pt3, pt4).expect("Invalid placement!");
        let g = g.apply(action);

        // [0  ][0  ][0  ][0  ][0  ]
        // [0  ][0P1][0P3][0  ][0  ]
        // [0  ][0P4][0P2][0  ][0  ]
        // [0  ][0  ][0  ][0  ][0  ]
        // [0  ][0  ][0  ][0  ][0  ]

        let pt1a = Point::new(1.into(), 0.into());
        let [pawn1, _] = g.active_pawns();
        let action = pawn1.can_move(pt1a).expect("Invalid movement!");
        let g = g.apply(action).expect("Invalid victory!");

        let pawn1 = g.active_pawn();
        let action = pawn1.can_build(pt1).expect("Invalid build");
        let g = g.apply(action).expect("Invalid victory!");

        // [0  ][0P1][0  ][0  ][0  ]
        // [0  ][1  ][0P3][0  ][0  ]
        // [0  ][0P4][0P2][0  ][0  ]
        // [0  ][0  ][0  ][0  ][0  ]
        // [0  ][0  ][0  ][0  ][0  ]

        let pt3a = Point::new(2.into(), 0.into());
        let [pawn3, _] = g.active_pawns();
        let action = pawn3.can_move(pt3a).expect("Invalid movement!");
        let g = g.apply(action).expect("Invalid victory!");

        let pawn3 = g.active_pawn();
        let action = pawn3.can_build(pt1).expect("Invalid build");
        let g = g.apply(action).expect("Invalid victory!");

        // [0  ][0P1][0P3][0  ][0  ]
        // [0  ][2  ][0  ][0  ][0  ]
        // [0  ][0P4][0P2][0  ][0  ]
        // [0  ][0  ][0  ][0  ][0  ]
        // [0  ][0  ][0  ][0  ][0  ]

        let [pawn1, _] = g.active_pawns();
        let action = pawn1.can_move(pt3).expect("Invalid movement!");
        let g = g.apply(action).expect("Invalid victory!");

        let pawn1 = g.active_pawn();
        let action = pawn1.can_build(pt1).expect("Invalid build");
        let g = g.apply(action).expect("Invalid victory!");

        // [0  ][0  ][0P3][0  ][0  ]
        // [0  ][3  ][0P1][0  ][0  ]
        // [0  ][0P4][0P2][0  ][0  ]
        // [0  ][0  ][0  ][0  ][0  ]
        // [0  ][0  ][0  ][0  ][0  ]

        let [pawn3, _] = g.active_pawns();
        let action = pawn3.can_move(pt1a).expect("Invalid movement!");
        let g = g.apply(action).expect("Invalid victory!");

        let pawn3 = g.active_pawn();
        let action = pawn3.can_build(pt1).expect("Invalid build");
        let g = g.apply(action).expect("Invalid victory!");

        // [0  ][0P3][0  ][0  ][0  ]
        // [0  ][4  ][0P1][0  ][0  ]
        // [0  ][0P4][0P2][0  ][0  ]
        // [0  ][0  ][0  ][0  ][0  ]
        // [0  ][0  ][0  ][0  ][0  ]

        let [pawn1, _] = g.active_pawns();
        let action = pawn1.can_move(pt3a).expect("Invalid movement!");
        let g = g.apply(action).expect("Invalid victory!");

        let pawn1 = g.active_pawn();
        assert_eq!(None, pawn1.can_build(pt1a));
        assert_eq!(None, pawn1.can_build(pt3a));
        assert_ne!(None, pawn1.can_build(Point::new(3.into(), 0.into())));
        assert_eq!(None, pawn1.can_build(pt1));
        assert_ne!(None, pawn1.can_build(pt3));
        assert_ne!(None, pawn1.can_build(Point::new(3.into(), 1.into())));
    }

    #[test]
    fn victory() {
        let g = new_game();
        let pt1 = Point::new(1.into(), 1.into());
        let pt2 = Point::new(2.into(), 2.into());
        let pt3 = Point::new(2.into(), 1.into());
        let pt4 = Point::new(1.into(), 2.into());

        let action = g.can_place(pt1, pt2).expect("Invalid placement!");
        let g = g.apply(action);
        let action = g.can_place(pt3, pt4).expect("Invalid placement!");
        let g = g.apply(action);

        // [0  ][0  ][0  ][0  ][0  ]
        // [0  ][0P1][0P3][0  ][0  ]
        // [0  ][0P4][0P2][0  ][0  ]
        // [0  ][0  ][0  ][0  ][0  ]
        // [0  ][0  ][0  ][0  ][0  ]

        let pt1a = Point::new(1.into(), 0.into());
        let [pawn1, _] = g.active_pawns();
        let action = pawn1.can_move(pt1a).expect("Invalid movement!");
        let g = g.apply(action).expect("Invalid victory!");

        let pawn1 = g.active_pawn();
        let action = pawn1.can_build(pt1).expect("Invalid build");
        let g = g.apply(action).expect("Invalid victory!");

        // [0  ][0P1][0  ][0  ][0  ]
        // [0  ][1  ][0P3][0  ][0  ]
        // [0  ][0P4][0P2][0  ][0  ]
        // [0  ][0  ][0  ][0  ][0  ]
        // [0  ][0  ][0  ][0  ][0  ]

        let pt3a = Point::new(2.into(), 0.into());
        let [pawn3, _] = g.active_pawns();
        let action = pawn3.can_move(pt3a).expect("Invalid movement!");
        let g = g.apply(action).expect("Invalid victory!");

        let pawn3 = g.active_pawn();
        let action = pawn3.can_build(pt3).expect("Invalid build");
        let g = g.apply(action).expect("Invalid victory!");

        // [0  ][0P1][0P3][0  ][0  ]
        // [0  ][1  ][1  ][0  ][0  ]
        // [0  ][0P4][0P2][0  ][0  ]
        // [0  ][0  ][0  ][0  ][0  ]
        // [0  ][0  ][0  ][0  ][0  ]

        let [_, pawn2] = g.active_pawns();
        let action = pawn2.can_move(pt3).expect("Invalid movement!");
        let g = g.apply(action).expect("Invalid victory!");

        let pawn2 = g.active_pawn();
        let action = pawn2.can_build(pt2).expect("Invalid build");
        let g = g.apply(action).expect("Invalid victory!");

        // [0  ][0P1][0P3][0  ][0  ]
        // [0  ][1  ][1P2][0  ][0  ]
        // [0  ][0P4][1  ][0  ][0  ]
        // [0  ][0  ][0  ][0  ][0  ]
        // [0  ][0  ][0  ][0  ][0  ]

        let [_, pawn4] = g.active_pawns();
        let action = pawn4.can_move(pt2).expect("Invalid movement!");
        let g = g.apply(action).expect("Invalid victory!");

        let pawn4 = g.active_pawn();
        let action = pawn4.can_build(pt1).expect("Invalid build");
        let g = g.apply(action).expect("Invalid victory!");

        // [0  ][0P1][0P3][0  ][0  ]
        // [0  ][2  ][1P2][0  ][0  ]
        // [0  ][0  ][1P4][0  ][0  ]
        // [0  ][0  ][0  ][0  ][0  ]
        // [0  ][0  ][0  ][0  ][0  ]

        let [_, pawn2] = g.active_pawns();
        let action = pawn2.can_move(pt1).expect("Invalid movement!");
        let g = g.apply(action).expect("Invalid victory!");

        let pawn2 = g.active_pawn();
        let action = pawn2.can_build(pt3).expect("Invalid build");
        let g = g.apply(action).expect("Invalid victory!");

        // [0  ][0P1][0P3][0  ][0  ]
        // [0  ][2P2][2  ][0  ][0  ]
        // [0  ][0  ][1P4][0  ][0  ]
        // [0  ][0  ][0  ][0  ][0  ]
        // [0  ][0  ][0  ][0  ][0  ]

        let [_, pawn4] = g.active_pawns();
        let action = pawn4.can_move(pt4).expect("Invalid movement!");
        let g = g.apply(action).expect("Invalid victory!");

        let pawn4 = g.active_pawn();
        let action = pawn4.can_build(pt3).expect("Invalid build");
        let g = g.apply(action).expect("Invalid victory!");

        // [0  ][0P1][0P3][0  ][0  ]
        // [0  ][2P2][3  ][0  ][0  ]
        // [0  ][0P4][1  ][0  ][0  ]
        // [0  ][0  ][0  ][0  ][0  ]
        // [0  ][0  ][0  ][0  ][0  ]

        let [_, pawn2] = g.active_pawns();
        let action = pawn2.can_move(pt3).expect("Invalid movement!");
        let g = g.apply(action);

        // [0  ][0P1][0P3][0  ][0  ]
        // [0  ][2  ][3P2][0  ][0  ]
        // [0  ][0P4][1  ][0  ][0  ]
        // [0  ][0  ][0  ][0  ][0  ]
        // [0  ][0  ][0  ][0  ][0  ]

        if let ActionResult::Victory(g) = g {
            assert_eq!(g.player(), Player::PlayerOne);
        } else {
            panic!("Victory not detected!");
        }
    }

    #[test]
    fn victory_stalemate() {
        let g = new_game();
        let pt1 = Point::new(1.into(), 1.into());
        let pt2 = Point::new(1.into(), 2.into());
        let pt3 = Point::new(0.into(), 2.into());
        let pt4 = Point::new(2.into(), 2.into());

        let action = g.can_place(pt1, pt2).expect("Invalid placement!");
        let g = g.apply(action);
        let action = g.can_place(pt3, pt4).expect("Invalid placement!");
        let g = g.apply(action);

        // [0  ][0  ][0  ][0  ][0  ]
        // [0  ][0P1][0  ][0  ][0  ]
        // [0P3][0P2][0P4][0  ][0  ]
        // [0  ][0  ][0  ][0  ][0  ]
        // [0  ][0  ][0  ][0  ][0  ]

        let pt1a = Point::new(0.into(), 0.into());
        let [pawn1, _] = g.active_pawns();
        let action = pawn1.can_move(pt1a).expect("Invalid movement!");
        let g = g.apply(action).expect("Invalid victory!");

        let wall_top = Point::new(1.into(), 0.into());
        let pawn1 = g.active_pawn();
        let action = pawn1.can_build(wall_top).expect("Invalid build");
        let g = g.apply(action).expect("Invalid victory!");

        // [0P1][1  ][0  ][0  ][0  ]
        // [0  ][0  ][0  ][0  ][0  ]
        // [0P3][0P2][0P4][0  ][0  ]
        // [0  ][0  ][0  ][0  ][0  ]
        // [0  ][0  ][0  ][0  ][0  ]

        let pt4a = Point::new(2.into(), 1.into());
        let [_, pawn4] = g.active_pawns();
        let action = pawn4.can_move(pt4a).expect("Invalid movement!");
        let g = g.apply(action).expect("Invalid victory!");

        let pawn4 = g.active_pawn();
        let action = pawn4.can_build(wall_top).expect("Invalid build");
        let g = g.apply(action).expect("Invalid victory!");

        // [0P1][2  ][0  ][0  ][0  ]
        // [0  ][0  ][0P4][0  ][0  ]
        // [0P3][0P2][0  ][0  ][0  ]
        // [0  ][0  ][0  ][0  ][0  ]
        // [0  ][0  ][0  ][0  ][0  ]

        let pt2a = Point::new(0.into(), 1.into());
        let [_, pawn2] = g.active_pawns();
        let action = pawn2.can_move(pt2a).expect("Invalid movement!");
        let g = g.apply(action).expect("Invalid victory!");

        let pawn2 = g.active_pawn();
        let action = pawn2.can_build(pt1).expect("Invalid build");
        let g = g.apply(action).expect("Invalid victory!");

        // [0P1][2  ][0  ][0  ][0  ]
        // [0P2][1  ][0P4][0  ][0  ]
        // [0P3][0  ][0  ][0  ][0  ]
        // [0  ][0  ][0  ][0  ][0  ]
        // [0  ][0  ][0  ][0  ][0  ]

        let [_, pawn4] = g.active_pawns();
        let action = pawn4.can_move(pt2).expect("Invalid movement!");
        let g = g.apply(action).expect("Invalid victory!");

        let pawn4 = g.active_pawn();
        let action = pawn4.can_build(pt1).expect("Invalid build");
        let g = g.apply(action);

        // [0P1][2  ][0  ][0  ][0  ]
        // [0P2][2  ][0  ][0  ][0  ]
        // [0P3][0P4][0  ][0  ][0  ]
        // [0  ][0  ][0  ][0  ][0  ]
        // [0  ][0  ][0  ][0  ][0  ]

        if let ActionResult::Victory(g) = g {
            assert_eq!(g.player(), Player::PlayerTwo);
        } else {
            panic!("Victory not detected!");
        }
    }
}
