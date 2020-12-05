use derive_more::{Add, Display, From};

use std::iter::Iterator;
use std::ops::Deref;
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
    word: i8,
    nibble: i8,
}

pub const BOARD_WIDTH: Coord = Coord(5);
pub const BOARD_HEIGHT: Coord = Coord(5);

impl Point {
    pub const fn x(&self) -> Coord {
        let offset = self.word * 16 + self.nibble / 4;
        Coord(offset % BOARD_WIDTH.0)
    }

    pub const fn y(&self) -> Coord {
        let offset = self.word * 16 + self.nibble / 4;
        Coord(offset / BOARD_WIDTH.0)
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
    pub const fn new_(x: Coord, y: Coord) -> Option<Point> {
        if x.0 >= BOARD_WIDTH.0 || x.0 < 0 || y.0 >= BOARD_HEIGHT.0 || y.0 < 0 {
            None
        } else {
            let offset = BOARD_WIDTH.0 * y.0 + x.0;
            Some(Point {
                word: offset / 16,
                nibble: 4 * (offset % 16),
            })
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
    fn x_y() {
        for x in 0..5 {
            for y in 0..5 {
                let point = Point::new(Coord::from(x), Coord::from(y));
                assert_eq!(point.x(), Coord::from(x));
                assert_eq!(point.y(), Coord::from(y));
            }
        }
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

impl From<i8> for CoordLevel {
    fn from(val: i8) -> CoordLevel {
        match val {
            0 => CoordLevel::Ground,
            1 => CoordLevel::One,
            2 => CoordLevel::Two,
            3 => CoordLevel::Three,
            4 => CoordLevel::Capped,
            _ => panic!("Invalid coord level value: {}", val),
        }
    }
}

impl From<CoordLevel> for i8 {
    fn from(val: CoordLevel) -> i8 {
        match val {
            CoordLevel::Ground => 0,
            CoordLevel::One => 1,
            CoordLevel::Two => 2,
            CoordLevel::Three => 3,
            CoordLevel::Capped => 4,
        }
    }
}

impl Ord for CoordLevel {
    fn cmp(&self, other: &CoordLevel) -> std::cmp::Ordering {
        i8::from(*self).cmp(&i8::from(*other))
    }
}

impl PartialOrd for CoordLevel {
    fn partial_cmp(&self, other: &CoordLevel) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Board {
    grid: [u64; 2],
}

impl Board {
    fn new() -> Board {
        Board {
            grid: [0x1111_1111_1111_1111; 2],
        }
    }

    pub fn level_at(&self, loc: Point) -> CoordLevel {
        let data = self.grid[loc.word as usize];
        let data = (data >> loc.nibble) & 0xF;
        match data {
            0b0001 => CoordLevel::Ground,
            0b0010 => CoordLevel::One,
            0b0100 => CoordLevel::Two,
            0b1000 => CoordLevel::Three,
            0b0000 => CoordLevel::Capped,
            _ => panic!("Invalid entry at {:?}: {}", loc, data),
        }
    }

    pub fn less_than_equals(&self, loc: Point, level: CoordLevel) -> bool {
        let data = self.grid[loc.word as usize];
        let mask = match level {
            CoordLevel::Ground => 0b0001,
            CoordLevel::One => 0b0011,
            CoordLevel::Two => 0b0111,
            CoordLevel::Three => 0b1111,
            CoordLevel::Capped => return true,
        };
        let mask = mask << loc.nibble;
        return data & mask != 0;
    }

    fn build(&mut self, loc: Point) {
        let data = &mut self.grid[loc.word as usize];
        let level = (*data >> loc.nibble) & 0xF;
        let mask = match level {
            0b0001 => 0b0011,
            0b0010 => 0b0110,
            0b0100 => 0b1100,
            0b1000 => 0b1000,
            0b0000 => panic!["Invalid build action!"],
            _ => panic!("Invalid entry at {:?}: {}", loc, data),
        };
        let mask = mask << loc.nibble;
        *data ^= mask;
    }

    fn cap(&mut self, loc: Point) {
        let data = &mut self.grid[loc.word as usize];
        let mask1 = !(0xF << loc.nibble);
        *data &= mask1;
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

    #[test]
    fn cap() {
        let pt = Point::new(2.into(), 2.into());
        let mut b = Board::new();

        assert_eq!(b.level_at(pt), CoordLevel::Ground);
        b.cap(pt);
        assert_eq!(b.level_at(pt), CoordLevel::Capped);
    }

    #[test]
    fn less_than_equals() {
        let pt = Point::new(2.into(), 2.into());
        let mut b = Board::new();

        assert_eq!(b.less_than_equals(pt, CoordLevel::Ground), true);
        assert_eq!(b.less_than_equals(pt, CoordLevel::One), true);
        assert_eq!(b.less_than_equals(pt, CoordLevel::Two), true);
        assert_eq!(b.less_than_equals(pt, CoordLevel::Three), true);
        assert_eq!(b.less_than_equals(pt, CoordLevel::Capped), true);

        b.build(pt);
        assert_eq!(b.less_than_equals(pt, CoordLevel::Ground), false);
        assert_eq!(b.less_than_equals(pt, CoordLevel::One), true);
        assert_eq!(b.less_than_equals(pt, CoordLevel::Two), true);
        assert_eq!(b.less_than_equals(pt, CoordLevel::Three), true);
        assert_eq!(b.less_than_equals(pt, CoordLevel::Capped), true);

        b.build(pt);
        assert_eq!(b.less_than_equals(pt, CoordLevel::Ground), false);
        assert_eq!(b.less_than_equals(pt, CoordLevel::One), false);
        assert_eq!(b.less_than_equals(pt, CoordLevel::Two), true);
        assert_eq!(b.less_than_equals(pt, CoordLevel::Three), true);
        assert_eq!(b.less_than_equals(pt, CoordLevel::Capped), true);

        b.build(pt);
        assert_eq!(b.less_than_equals(pt, CoordLevel::Ground), false);
        assert_eq!(b.less_than_equals(pt, CoordLevel::One), false);
        assert_eq!(b.less_than_equals(pt, CoordLevel::Two), false);
        assert_eq!(b.less_than_equals(pt, CoordLevel::Three), true);
        assert_eq!(b.less_than_equals(pt, CoordLevel::Capped), true);

        b.build(pt);
        assert_eq!(b.less_than_equals(pt, CoordLevel::Ground), false);
        assert_eq!(b.less_than_equals(pt, CoordLevel::One), false);
        assert_eq!(b.less_than_equals(pt, CoordLevel::Two), false);
        assert_eq!(b.less_than_equals(pt, CoordLevel::Three), false);
        assert_eq!(b.less_than_equals(pt, CoordLevel::Capped), true);
    }
}

/// A CompositeBoard is a board where the tiles occupied by pawns
/// have been capped, allowing for quicker checking of valid moves
struct CompositeBoard {
    board: Board,
}

impl CompositeBoard {
    fn check(&self, loc: Point, max_height: CoordLevel) -> bool {
        self.board.less_than_equals(loc, max_height)
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

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
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
    fn composite_board(&self) -> CompositeBoard {
        let mut board = self.board;

        for player in Player::iter() {
            for loc in &self.state.player_locs(*player) {
                board.cap(*loc);
            }
        }

        CompositeBoard { board }
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

    pub fn resign(self) -> Game<Victory> {
        Game {
            state: Victory {
                player1_locs: self.state.player_locs(Player::PlayerOne),
                player2_locs: self.state.player_locs(Player::PlayerTwo),
            },
            board: self.board,
            player: self.player.other(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
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

    pub fn neighbors(&self) -> impl Iterator<Item = Point> {
        const OFFSETS: [(i8, i8); 8] = [
            (-1, -1),
            (0, -1),
            (1, -1),
            (-1, 0),
            (1, 0),
            (-1, 1),
            (0, 1),
            (1, 1),
        ];

        const fn neighbors_table(
        ) -> [[(usize, [Point; 8]); BOARD_HEIGHT.0 as usize]; BOARD_WIDTH.0 as usize] {
            let mut array = [[(0, [Point { word: 0, nibble: 0 }; 8]); BOARD_HEIGHT.0 as usize];
                BOARD_WIDTH.0 as usize];
            let mut x = 0;
            while x < BOARD_WIDTH.0 {
                let mut y = 0;
                while y < BOARD_HEIGHT.0 {
                    let mut count = 0;
                    let mut index = 0;
                    while index < 8 {
                        let (dx, dy) = OFFSETS[index];
                        match Point::new_(Coord(x + dx), Coord(y + dy)) {
                            Some(point) => {
                                array[x as usize][y as usize].1[count] = point;
                                count += 1;
                            }
                            None => (),
                        }
                        array[x as usize][y as usize].0 = count;
                        index += 1;
                    }
                    y += 1;
                }
                x += 1;
            }
            array
        }

        static LOOKUP_TABLE: [[(usize, [Point; 8]); BOARD_HEIGHT.0 as usize];
            BOARD_WIDTH.0 as usize] = neighbors_table();

        let x: usize = self.pos.x().into();
        let y: usize = self.pos.y().into();
        let (len, data) = &LOOKUP_TABLE[x][y];
        data[0..*len].iter().cloned()
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

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct MoveAction {
    from: Point,
    to: Point,
    #[cfg(debug_assertions)]
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

const OFFSETS: [(i8, i8); 8] = [
    (-1, -1),
    (0, -1),
    (1, -1),
    (-1, 0),
    (1, 0),
    (-1, 1),
    (0, 1),
    (1, 1),
];

const fn neighbors_table() -> [[u64; 61]; 2] {
    let mut array = [[0; 61]; 2];
    let mut word = 0;
    while word < 2 {
        let mut nibble = 0;
        while nibble < 61 {
            if nibble % 4 != 0 {
                nibble += 1;
                continue;
            }

            let point = Point {
                word: word,
                nibble: nibble,
            };
            if point.y().0 >= BOARD_HEIGHT.0 {
                break;
            }
            let mut prev = 0;
            let mut offset: u64 = 0;
            let mut index = 0;
            let mut count = 0;
            while index < 8 {
                let (dx, dy) = OFFSETS[index];
                match Point::new_(Coord(point.x().0 + dx), Coord(point.y().0 + dy)) {
                    Some(point) => {
                        let off = point.word * 64 + point.nibble;
                        let diff: i8 = off - prev;
                        let diff: u64 = (diff as u64) << (count * 8);
                        offset |= diff;
                        prev = off;
                        count += 1;
                    }
                    None => (),
                }
                index += 1;
            }
            array[word as usize][nibble as usize] = offset;
            nibble += 1;
        }
        word += 1;
    }
    array
}

static ACTION_LOOKUP_TABLE: [[u64; 61]; 2] = neighbors_table();

const fn mask_table() -> [[[[u64; 2]; 3]; 61]; 2] {
    let mut array = [[[[0; 2]; 3]; 61]; 2];
    let mut word = 0;
    while word < 2 {
        let mut nibble = 0;
        while nibble < 61 {
            if nibble % 4 != 0 {
                nibble += 1;
                continue;
            }

            let point = Point {
                word: word,
                nibble: nibble,
            };
            if point.y().0 >= BOARD_HEIGHT.0 {
                break;
            }

            let mut level = 0;
            while level < 3 {
                let mask = match level {
                    0 => 0b0011,
                    1 => 0b0111,
                    2 => 0b1111,
                    _ => 0, // We should never get here
                };

                let mut index = 0;
                let mut entry = [0; 2];
                while index < 8 {
                    let (dx, dy) = OFFSETS[index];
                    match Point::new_(Coord(point.x().0 + dx), Coord(point.y().0 + dy)) {
                        Some(point) => {
                            entry[point.word as usize] |= mask << point.nibble;
                        }
                        None => (),
                    }
                    index += 1;
                }
                array[word as usize][nibble as usize][level as usize] = entry;

                level += 1;
            }
            nibble += 1;
        }
        word += 1;
    }
    array
}

static MASK_LOOKUP_TABLE: [[[[u64; 2]; 3]; 61]; 2] = mask_table();

impl<'a> Pawn<'a, Move> {
    fn level_limit(&self) -> CoordLevel {
        match self.game.board.level_at(self.pos) {
            CoordLevel::Ground => CoordLevel::One,
            CoordLevel::One => CoordLevel::Two,
            CoordLevel::Two => CoordLevel::Three,
            level => panic!("Pawn at unreachable height: {:?}", level),
        }
    }

    pub fn can_move(&self, to: Point) -> Option<MoveAction> {
        if self.player != self.game.player {
            return None;
        }

        if self.pos.distance(to) != 1 {
            return None;
        }

        if !self.game.composite_board().check(to, self.level_limit()) {
            return None;
        }

        Some(MoveAction {
            from: self.pos,
            to,
            #[cfg(debug_assertions)]
            game: *self.game,
        })
    }

    pub fn has_actions(&self) -> bool {
        let mask = MASK_LOOKUP_TABLE[self.pos.word as usize][self.pos.nibble as usize][match self
            .game
            .board
            .level_at(self.pos)
        {
            CoordLevel::Ground => 0,
            CoordLevel::One => 1,
            CoordLevel::Two => 2,
            level => panic!("Pawn at unreachable height: {:?}", level),
        }];

        let composite = self.game.composite_board();
        if composite.board.grid[0] & mask[0] != 0 {
            return true;
        }

        if composite.board.grid[1] & mask[1] != 0 {
            return true;
        }

        return false;
    }

    pub fn actions(&self) -> impl Iterator<Item = MoveAction> {
        struct ActionsIterator {
            board: u64,
            offsets: u64,
            mask: u64,
            action: MoveAction,
        }

        impl Iterator for ActionsIterator {
            type Item = MoveAction;

            fn next(&mut self) -> Option<MoveAction> {
                loop {
                    if self.offsets == 0 {
                        return None;
                    }

                    let off = self.offsets & 0xFF;
                    self.offsets = self.offsets >> 8;
                    self.board = self.board >> off;
                    self.action.to.nibble += off as i8;
                    if self.action.to.nibble & 0b1000000 != 0 {
                        self.action.to.word = 1;
                        self.action.to.nibble &= !0b1000000;
                    }

                    if self.board & self.mask != 0 {
                        break;
                    }
                }

                Some(self.action)
            }
        }

        if self.player != self.game.player {
            return ActionsIterator {
                board: 0,
                offsets: 0,
                mask: 0,
                action: MoveAction {
                    from: self.pos,
                    to: self.pos,
                    #[cfg(debug_assertions)]
                    game: *self.game,
                },
            };
        }

        let mask = match self.game.board.level_at(self.pos) {
            CoordLevel::Ground => 0b0011,
            CoordLevel::One => 0b0111,
            CoordLevel::Two => 0b1111,
            level => panic!("Pawn at unreachable height: {:?}", level),
        };

        let offsets = ACTION_LOOKUP_TABLE[self.pos.word as usize][self.pos.nibble as usize];
        let off: u64 = offsets & 0xFF;
        let offsets = offsets & !0xFF;

        let composite = self.game.composite_board();
        let board;
        if off >= 64 {
            let off = off - 64;
            board = composite.board.grid[1] >> off;
        } else {
            let board_a = composite.board.grid[0] >> off;
            let board_b;
            if off > 0 {
                board_b = composite.board.grid[1] << (64 - off);
            } else {
                board_b = 0;
            }
            board = board_a | board_b;
        }

        ActionsIterator {
            board,
            offsets,
            mask,
            action: MoveAction {
                from: self.pos,
                to: Point {
                    word: 0,
                    nibble: off as i8,
                },
                #[cfg(debug_assertions)]
                game: *self.game,
            },
        }
    }
}

// We use a macro because we need to write this function for P1 and P2
// with minimal differences
impl Game<Move> {
    pub fn apply(self, action: MoveAction) -> ActionResult<Build> {
        #[cfg(debug_assertions)]
        assert!(
            action.game == self,
            "Game {:?} received action {:?} associated with a different game!",
            self,
            action
        );

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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct BuildAction {
    loc: Point,
    #[cfg(debug_assertions)]
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
            && self.game.composite_board().check(loc, CoordLevel::Three)
        {
            Some(BuildAction {
                loc,
                #[cfg(debug_assertions)]
                game: *self.game,
            })
        } else {
            None
        }
    }

    pub fn actions(&self) -> impl Iterator<Item = BuildAction> {
        let is_active_pawn = *self == self.game.active_pawn();
        let game = *self.game;
        let composite = game.composite_board();
        self.neighbors()
            .filter(move |_| is_active_pawn)
            .filter(move |loc| composite.check(*loc, CoordLevel::Three))
            .map(move |loc| BuildAction {
                loc,
                #[cfg(debug_assertions)]
                game,
            })
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
        #[cfg(debug_assertions)]
        assert!(
            action.game == self,
            "Game {:?} received action {:?} associated with a different game!",
            self,
            action
        );

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
        let pawns = new_game.active_pawns();
        if pawns[0].has_actions() || pawns[1].has_actions() {
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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
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
                game: *self,
            })
        } else {
            None
        }
    }

    pub fn apply(self, placement: PlaceAction<PlaceOne>) -> Game<PlaceTwo> {
        debug_assert!(
            placement.game == self,
            "Game {:?} received action {:?} associated with a different game!",
            self,
            placement
        );

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
                game: *self,
            })
        } else {
            None
        }
    }

    pub fn apply(self, placement: PlaceAction<PlaceTwo>) -> Game<Move> {
        debug_assert!(
            placement.game == self,
            "Game {:?} received action {:?} associated with a different game!",
            self,
            placement
        );

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

        assert_eq!(pawn1.neighbors().collect::<Vec<Point>>(), neighbors1);
        assert_eq!(pawn2.neighbors().collect::<Vec<Point>>(), neighbors2);
        assert_eq!(pawn3.neighbors().collect::<Vec<Point>>(), neighbors3);
        assert_eq!(pawn4.neighbors().collect::<Vec<Point>>(), neighbors4);
    }

    #[test]
    fn move_actions() {
        let g = new_game();
        let pt1 = Point::new(0.into(), 0.into());
        let pt2 = Point::new(3.into(), 1.into());
        let pt3 = Point::new(4.into(), 4.into());
        let pt4 = Point::new(0.into(), 3.into());

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
                #[cfg(debug_assertions)]
                game: g,
            },
            MoveAction {
                from: pt1,
                to: Point::new(0.into(), 1.into()),
                #[cfg(debug_assertions)]
                game: g,
            },
            MoveAction {
                from: pt1,
                to: Point::new(1.into(), 1.into()),
                #[cfg(debug_assertions)]
                game: g,
            },
        ];
        let moves2 = [
            MoveAction {
                from: pt2,
                to: Point::new(2.into(), 0.into()),
                #[cfg(debug_assertions)]
                game: g,
            },
            MoveAction {
                from: pt2,
                to: Point::new(3.into(), 0.into()),
                #[cfg(debug_assertions)]
                game: g,
            },
            MoveAction {
                from: pt2,
                to: Point::new(4.into(), 0.into()),
                #[cfg(debug_assertions)]
                game: g,
            },
            MoveAction {
                from: pt2,
                to: Point::new(2.into(), 1.into()),
                #[cfg(debug_assertions)]
                game: g,
            },
            MoveAction {
                from: pt2,
                to: Point::new(4.into(), 1.into()),
                #[cfg(debug_assertions)]
                game: g,
            },
            MoveAction {
                from: pt2,
                to: Point::new(2.into(), 2.into()),
                #[cfg(debug_assertions)]
                game: g,
            },
            MoveAction {
                from: pt2,
                to: Point::new(3.into(), 2.into()),
                #[cfg(debug_assertions)]
                game: g,
            },
            MoveAction {
                from: pt2,
                to: Point::new(4.into(), 2.into()),
                #[cfg(debug_assertions)]
                game: g,
            },
        ];

        assert_eq!(pawn1.actions().collect::<Vec<MoveAction>>(), moves1);
        assert_eq!(pawn2.actions().collect::<Vec<MoveAction>>(), moves2);
        assert_eq!(pawn3.actions().collect::<Vec<MoveAction>>(), []);
        assert_eq!(pawn4.actions().collect::<Vec<MoveAction>>(), []);

        let g = g.apply(moves1[0]).expect("Invalid victory!");
        let build = g
            .active_pawn()
            .can_build(Point::new(0.into(), 0.into()))
            .expect("Invalid build!");
        let g = g.apply(build).expect("Invalid victory!");

        let [pawn1, pawn2] = g.inactive_pawns();
        let [pawn3, pawn4] = g.active_pawns();

        let moves3 = [
            MoveAction {
                from: pt3,
                to: Point::new(3.into(), 3.into()),
                #[cfg(debug_assertions)]
                game: g,
            },
            MoveAction {
                from: pt3,
                to: Point::new(4.into(), 3.into()),
                #[cfg(debug_assertions)]
                game: g,
            },
            MoveAction {
                from: pt3,
                to: Point::new(3.into(), 4.into()),
                #[cfg(debug_assertions)]
                game: g,
            },
        ];
        let moves4 = [
            MoveAction {
                from: pt4,
                to: Point::new(0.into(), 2.into()),
                #[cfg(debug_assertions)]
                game: g,
            },
            MoveAction {
                from: pt4,
                to: Point::new(1.into(), 2.into()),
                #[cfg(debug_assertions)]
                game: g,
            },
            MoveAction {
                from: pt4,
                to: Point::new(1.into(), 3.into()),
                #[cfg(debug_assertions)]
                game: g,
            },
            MoveAction {
                from: pt4,
                to: Point::new(0.into(), 4.into()),
                #[cfg(debug_assertions)]
                game: g,
            },
            MoveAction {
                from: pt4,
                to: Point::new(1.into(), 4.into()),
                #[cfg(debug_assertions)]
                game: g,
            },
        ];

        assert_eq!(pawn1.actions().collect::<Vec<MoveAction>>(), []);
        assert_eq!(pawn2.actions().collect::<Vec<MoveAction>>(), []);
        assert_eq!(pawn3.actions().collect::<Vec<MoveAction>>(), moves3);
        assert_eq!(pawn4.actions().collect::<Vec<MoveAction>>(), moves4);
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
                #[cfg(debug_assertions)]
                game: g,
            },
            BuildAction {
                loc: Point::new(1.into(), 0.into()),
                #[cfg(debug_assertions)]
                game: g,
            },
            BuildAction {
                loc: Point::new(1.into(), 1.into()),
                #[cfg(debug_assertions)]
                game: g,
            },
            BuildAction {
                loc: Point::new(0.into(), 2.into()),
                #[cfg(debug_assertions)]
                game: g,
            },
            BuildAction {
                loc: Point::new(1.into(), 2.into()),
                #[cfg(debug_assertions)]
                game: g,
            },
        ];

        assert_eq!(pawn1.actions().collect::<Vec<BuildAction>>(), build1);
        assert_eq!(pawn2.actions().collect::<Vec<BuildAction>>(), []);
        assert_eq!(pawn3.actions().collect::<Vec<BuildAction>>(), []);
        assert_eq!(pawn4.actions().collect::<Vec<BuildAction>>(), []);
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
