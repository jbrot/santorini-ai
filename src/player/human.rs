use std::io;
use termion::event::{Event, Key};
use termion::input::TermRead;

use crate::player::{FullPlayer, Player, StepResult};
use crate::santorini::{
    self, ActionResult, Build, Game, GameState, Move, NormalState, Pawn, PlaceOne, PlaceTwo, Point,
};
use crate::ui::{BoardWidget, UpdateError};

pub struct HumanPlayer {
    cursor: Point,
    highlights: Vec<Point>,
    intermediate_loc: Option<Point>,
}

impl HumanPlayer {
    pub fn new() -> Box<dyn FullPlayer> {
        Box::new(HumanPlayer {
            cursor: Point::new(0.into(), 0.into()),
            highlights: vec![],
            intermediate_loc: None,
        })
    }

    fn move_with_highlights(&mut self, filter: impl Fn(Point) -> bool) {
        let mut best_pt = self.cursor;
        let mut best_distance = i8::MAX;
        for point in &self.highlights {
            if filter(*point) {
                let distance = point.taxicab(self.cursor);
                if distance < best_distance {
                    best_distance = distance;
                    best_pt = *point;
                }
            }
        }

        self.cursor = best_pt;
    }

    fn move_up(&mut self) {
        if self.highlights.is_empty() {
            self.cursor =
                Point::new_(self.cursor.x(), self.cursor.y() + (-1).into()).unwrap_or(self.cursor);
        } else {
            let cursor_y = self.cursor.y();
            self.move_with_highlights(|point| point.y() < cursor_y);
        }
    }

    fn move_down(&mut self) {
        if self.highlights.is_empty() {
            self.cursor =
                Point::new_(self.cursor.x(), self.cursor.y() + 1.into()).unwrap_or(self.cursor);
        } else {
            let cursor_y = self.cursor.y();
            self.move_with_highlights(|point| point.y() > cursor_y);
        }
    }

    fn move_left(&mut self) {
        if self.highlights.is_empty() {
            self.cursor =
                Point::new_(self.cursor.x() + (-1).into(), self.cursor.y()).unwrap_or(self.cursor);
        } else {
            let cursor_x = self.cursor.x();
            self.move_with_highlights(|point| point.x() < cursor_x);
        }
    }

    fn move_right(&mut self) {
        if self.highlights.is_empty() {
            self.cursor =
                Point::new_(self.cursor.x() + 1.into(), self.cursor.y()).unwrap_or(self.cursor);
        } else {
            let cursor_x = self.cursor.x();
            self.move_with_highlights(|point| point.x() > cursor_x);
        }
    }

    fn default_input_handler(&mut self, event: Event) -> Result<(), UpdateError> {
        match event {
            Event::Key(Key::Ctrl('c')) => return Err(UpdateError::Shutdown),
            Event::Key(Key::Up) | Event::Key(Key::Char('w')) => self.move_up(),
            Event::Key(Key::Left) | Event::Key(Key::Char('a')) => self.move_left(),
            Event::Key(Key::Down) | Event::Key(Key::Char('s')) => self.move_down(),
            Event::Key(Key::Right) | Event::Key(Key::Char('d')) => self.move_right(),
            _ => (),
        }
        Ok(())
    }

    fn default_render<T: GameState + NormalState>(&self, game: &Game<T>) -> BoardWidget {
        BoardWidget {
            board: game.board(),
            player: game.player(),
            cursor: Some(self.cursor),

            highlights: &self.highlights,
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
}

fn pawn_at<T: GameState + NormalState + Clone>(game: &Game<T>, loc: Point) -> Option<Pawn<T>> {
    for pawn in game.active_pawns().iter() {
        if pawn.pos() == loc {
            return Some(pawn.clone());
        }
    }
    None
}

impl Player<PlaceOne> for HumanPlayer {
    fn prepare(&mut self, _: &Game<PlaceOne>) {
        self.highlights = vec![];
        self.intermediate_loc = None;
    }

    fn render(&self, game: &Game<PlaceOne>) -> BoardWidget {
        BoardWidget {
            board: game.board(),
            player: game.player(),
            cursor: Some(self.cursor),

            highlights: &self.highlights,
            player1_locs: self.intermediate_loc.iter().cloned().collect(),
            player2_locs: vec![],
        }
    }

    fn step(&mut self, game: &Game<PlaceOne>) -> Result<StepResult, UpdateError> {
        match io::stdin().events().next().unwrap()? {
            Event::Key(Key::Char('q')) | Event::Key(Key::Esc) => {
                if !self.intermediate_loc.is_none() {
                    self.intermediate_loc = None;
                }
            }
            Event::Key(Key::Char('\n')) | Event::Key(Key::Char('e')) => {
                if let Some(pos1) = self.intermediate_loc {
                    if let Some(action) = game.can_place(pos1, self.cursor) {
                        return Ok(StepResult::PlaceTwo(game.clone().apply(action)));
                    }
                } else {
                    self.intermediate_loc = Some(self.cursor);
                }
            }
            event => self.default_input_handler(event)?,
        }

        Ok(StepResult::NoMove)
    }
}

impl Player<PlaceTwo> for HumanPlayer {
    fn prepare(&mut self, _: &Game<PlaceTwo>) {
        self.highlights = vec![];
        self.intermediate_loc = None;
    }

    fn render(&self, game: &Game<PlaceTwo>) -> BoardWidget {
        BoardWidget {
            board: game.board(),
            player: game.player(),
            cursor: Some(self.cursor),

            highlights: &self.highlights,
            player1_locs: game.player1_locs().to_vec(),
            player2_locs: self.intermediate_loc.iter().cloned().collect(),
        }
    }

    fn step(&mut self, game: &Game<PlaceTwo>) -> Result<StepResult, UpdateError> {
        match io::stdin().events().next().unwrap()? {
            Event::Key(Key::Char('q')) | Event::Key(Key::Esc) => {
                if !self.intermediate_loc.is_none() {
                    self.intermediate_loc = None;
                }
            }
            Event::Key(Key::Char('\n')) | Event::Key(Key::Char('e')) => {
                for pos in game.player1_locs().iter() {
                    if *pos == self.cursor {
                        return Ok(StepResult::NoMove);
                    }
                }

                if let Some(pos1) = self.intermediate_loc {
                    if let Some(action) = game.can_place(pos1, self.cursor) {
                        return Ok(StepResult::Move(game.clone().apply(action)));
                    }
                } else {
                    self.intermediate_loc = Some(self.cursor);
                }
            }
            event => self.default_input_handler(event)?,
        }

        Ok(StepResult::NoMove)
    }
}

impl Player<Move> for HumanPlayer {
    fn prepare(&mut self, game: &Game<Move>) {
        self.highlights = game.active_pawns().iter().map(|pawn| pawn.pos()).collect();
        self.cursor = self.highlights[0];
        self.intermediate_loc = None;
    }

    fn render(&self, game: &Game<Move>) -> BoardWidget {
        self.default_render(game)
    }

    fn step(&mut self, game: &Game<Move>) -> Result<StepResult, UpdateError> {
        match io::stdin().events().next().unwrap()? {
            Event::Key(Key::F(6)) => return Ok(StepResult::Victory(game.clone().resign())),
            Event::Key(Key::Char('q')) | Event::Key(Key::Esc) => {
                if !self.intermediate_loc.is_none() {
                    self.prepare(game);
                }
            }
            Event::Key(Key::Char('\n')) | Event::Key(Key::Char('e')) => {
                if let Some(pawn) = self
                    .intermediate_loc
                    .map(|loc| pawn_at(&game, loc))
                    .flatten()
                {
                    let action = pawn.can_move(self.cursor).unwrap();
                    return match game.clone().apply(action) {
                        ActionResult::Continue(game) => Ok(StepResult::Build(game)),
                        ActionResult::Victory(game) => Ok(StepResult::Victory(game)),
                    };
                } else {
                    let pawn = pawn_at(&game, self.cursor).unwrap();
                    if let Some(action) = pawn.actions().iter().next() {
                        self.intermediate_loc = Some(self.cursor);
                        self.cursor = action.to();
                        self.highlights = pawn.actions().iter().map(|pair| pair.to()).collect();
                    }
                }
            }
            event => self.default_input_handler(event)?,
        }

        Ok(StepResult::NoMove)
    }
}

impl Player<Build> for HumanPlayer {
    fn prepare(&mut self, game: &Game<Build>) {
        self.highlights = game
            .active_pawn()
            .actions()
            .map(|build| build.loc())
            .collect();
        self.cursor = self.highlights[0];
    }

    fn render(&self, game: &Game<Build>) -> BoardWidget {
        self.default_render(game)
    }

    fn step(&mut self, game: &Game<Build>) -> Result<StepResult, UpdateError> {
        match io::stdin().events().next().unwrap()? {
            Event::Key(Key::F(6)) => return Ok(StepResult::Victory(game.clone().resign())),
            Event::Key(Key::Char('\n')) | Event::Key(Key::Char('e')) => {
                let action = game.active_pawn().can_build(self.cursor).unwrap();
                return match game.clone().apply(action) {
                    ActionResult::Continue(game) => Ok(StepResult::Move(game)),
                    ActionResult::Victory(game) => Ok(StepResult::Victory(game)),
                };
            }
            event => self.default_input_handler(event)?,
        }

        Ok(StepResult::NoMove)
    }
}
