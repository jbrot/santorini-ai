use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Coord(pub u8);
impl fmt::Display for Coord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub struct Point {
    x: Coord,
    y: Coord,
}

const BOARD_WIDTH: Coord = Coord(5);
const BOARD_HEIGHT: Coord = Coord(5);

impl Point {
    pub fn new(x: Coord, y: Coord) -> Point {
        if x >= BOARD_WIDTH || y >= BOARD_HEIGHT {
            panic!(
                "Coord must have values less than ({}, {})",
                BOARD_WIDTH, BOARD_HEIGHT
            )
        }

        Point { x, y }
    }

    pub fn x(&self) -> Coord {
        self.x
    }

    pub fn y(&self) -> Coord {
        self.y
    }
}

pub enum CoordLevel {
    Ground,
    One,
    Two,
    Three,
    Capped,
}

struct Board {
    pub grid: [[CoordLevel; BOARD_WIDTH.0 as usize]; BOARD_HEIGHT.0 as usize],
    pub player1a: Point,
    pub player1b: Point,
    pub player2a: Point,
    pub player2b: Point,
}
