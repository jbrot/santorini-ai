use std::io;
use termion::event::{Event, Key};
use termion::input::{MouseTerminal, TermRead};
use termion::raw::{IntoRawMode, RawTerminal};
use thiserror::Error;
use tui::backend::TermionBackend;
use tui::buffer::Buffer;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, Borders, Widget};
use tui::Terminal;

mod santorini;
use santorini::{
    ActionResult, Board, Coord, Game, GameState, Move, NormalState, Pawn, PlaceOne, PlaceTwo,
    Player, Point, BOARD_HEIGHT, BOARD_WIDTH,
};

struct BoundsWidget {
    min_width: u16,
    min_height: u16,
}

impl Widget for BoundsWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < self.min_width {
            let msg = "Expand ->";
            let len = msg.len() as u16;
            if len > area.width {
                buf.set_string(
                    area.left(),
                    area.top(),
                    &msg[(len - area.width).into()..],
                    Style::default(),
                );
            } else {
                buf.set_string(area.right() - len, area.top(), msg, Style::default());
            }
        } else if area.height < self.min_height {
            let msgs = ["Expand", "  |", "  V"];
            for o in (3 - u16::min(3, area.height))..3 {
                buf.set_string(
                    area.left(),
                    area.bottom() + o - 3,
                    msgs[usize::from(o)],
                    Style::default(),
                );
            }
        }
    }
}

struct BoardWidget {
    board: Board,

    player: Player,
    cursor: Point,
    highlights: Vec<Point>,

    player1_locs: Vec<Point>,
    player2_locs: Vec<Point>,
}

const SQUARE_SIZE: u16 = 5;
const BOARD_WIDGET_WIDTH: u16 = (BOARD_WIDTH.0 as u16) * SQUARE_SIZE;
const BOARD_WIDGET_HEIGHT: u16 = (BOARD_HEIGHT.0 as u16) * SQUARE_SIZE;

const DEFAULT_STYLE: Style = Style {
    bg: None,
    fg: None,
    add_modifier: Modifier::empty(),
    sub_modifier: Modifier::empty(),
};

const PLAYER_ONE_STYLE: Style = Style {
    bg: Some(Color::Indexed(21)),
    fg: Some(Color::White),
    ..DEFAULT_STYLE
};
const PLAYER_ONE_CURSOR_STYLE: Style = Style {
    bg: Some(Color::Indexed(45)),
    fg: Some(Color::White),
    ..DEFAULT_STYLE
};
const PLAYER_ONE_HIGHLIGHT_STYLE: Style = Style {
    bg: Some(Color::Indexed(33)),
    fg: Some(Color::Indexed(33)),
    ..DEFAULT_STYLE
};

const PLAYER_TWO_STYLE: Style = Style {
    bg: Some(Color::Indexed(160)),
    fg: Some(Color::White),
    ..DEFAULT_STYLE
};
const PLAYER_TWO_CURSOR_STYLE: Style = Style {
    bg: Some(Color::Indexed(204)),
    fg: Some(Color::White),
    ..DEFAULT_STYLE
};
const PLAYER_TWO_HIGHLIGHT_STYLE: Style = Style {
    bg: Some(Color::Indexed(204)),
    fg: Some(Color::Indexed(204)),
    ..DEFAULT_STYLE
};

impl BoardWidget {
    fn style(&self, point: Point) -> Option<Style> {
        for p in &self.player1_locs {
            if point == *p {
                return Some(PLAYER_ONE_STYLE);
            }
        }

        for p in &self.player2_locs {
            if point == *p {
                return Some(PLAYER_TWO_STYLE);
            }
        }

        None
    }

    fn border_style(&self, point: Point) -> Option<Style> {
        if point == self.cursor {
            if self.player == Player::PlayerOne {
                return Some(PLAYER_ONE_CURSOR_STYLE);
            } else {
                return Some(PLAYER_TWO_CURSOR_STYLE);
            }
        }

        for p in &self.highlights {
            if *p == point {
                if self.player == Player::PlayerOne {
                    return Some(PLAYER_ONE_HIGHLIGHT_STYLE);
                } else {
                    return Some(PLAYER_TWO_HIGHLIGHT_STYLE);
                }
            }
        }

        None
    }
}

impl Widget for BoardWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < BOARD_WIDGET_WIDTH || area.height < BOARD_WIDGET_HEIGHT {
            BoundsWidget {
                min_width: BOARD_WIDGET_WIDTH,
                min_height: BOARD_WIDGET_HEIGHT,
            }
            .render(area, buf);
            return;
        }

        let left = area.left() + (area.width - BOARD_WIDGET_WIDTH) / 2;
        let top = area.top() + (area.height - BOARD_WIDGET_HEIGHT) / 2;
        for x in 0..BOARD_WIDTH.0 as u16 {
            for y in 0..BOARD_HEIGHT.0 as u16 {
                let area = Rect {
                    x: left + x * SQUARE_SIZE,
                    y: top + y * SQUARE_SIZE,
                    width: SQUARE_SIZE,
                    height: SQUARE_SIZE,
                };
                let point = Point::new(Coord::from(x as i8), Coord::from(y as i8));
                let mut block = Block::default().borders(Borders::ALL);
                if let Some(style) = self.style(point) {
                    block = block.style(style);
                }
                if let Some(style) = self.border_style(point) {
                    block = block.border_style(style);
                }
                block.render(area, buf);

                buf.set_string(
                    area.left() + (area.width / 2),
                    area.top() + (area.height / 2),
                    format!("{}", self.board.level_at(point).height()),
                    Style::default(),
                );
            }
        }
    }
}

type Term = Terminal<TermionBackend<MouseTerminal<RawTerminal<io::Stdout>>>>;

#[derive(Error, Debug)]
enum UpdateError {
    #[error("issue updating display")]
    IoError(#[from] io::Error),
    #[error("normal exit")]
    Shutdown,
}

trait Screen {
    fn update(self: Box<Self>, terminal: &mut Term) -> Result<Box<dyn Screen>, UpdateError>;
}

struct App<T: GameState> {
    game: Game<T>,
    cursor: Point,
    intermediate_loc: Option<Point>,
}

impl<T: GameState> App<T> {
    fn draw(&self, terminal: &mut Term, widget: BoardWidget) -> Result<(), UpdateError> {
        terminal.draw(|f| {
            let border = Block::default().title("Santorini").borders(Borders::ALL);
            f.render_widget(border, f.size());

            let segments = Layout::default()
                .direction(Direction::Horizontal)
                .margin(1)
                .constraints([Constraint::Min(15), Constraint::Ratio(1, 3)].as_ref())
                .split(f.size());
            f.render_widget(widget, segments[0]);
            let side_text = Block::default().title("Side Text").borders(Borders::ALL);
            f.render_widget(side_text, segments[1]);
        })?;
        Ok(())
    }

    fn move_into_list(self, list: Vec<Point>, filter: impl Fn(Point) -> bool) -> Self {
        let mut best_pt = self.cursor;
        let mut best_distance = i8::MAX;
        for point in list {
            if filter(point) {
                let distance = point.taxicab(self.cursor);
                if distance < best_distance {
                    best_distance = distance;
                    best_pt = point;
                }
            }
        }
        App {
            cursor: best_pt,
            ..self
        }
    }

    fn move_up(self, options: Option<Vec<Point>>) -> Self {
        match options {
            Some(options) => {
                let cursor_y = self.cursor.y();
                self.move_into_list(options, |point| point.y() < cursor_y)
            }
            None => {
                let cursor = Point::new_(self.cursor.x(), self.cursor.y() + (-1).into())
                    .unwrap_or(self.cursor);
                App { cursor, ..self }
            }
        }
    }

    fn move_down(self, options: Option<Vec<Point>>) -> Self {
        match options {
            Some(options) => {
                let cursor_y = self.cursor.y();
                self.move_into_list(options, |point| point.y() > cursor_y)
            }
            None => {
                let cursor =
                    Point::new_(self.cursor.x(), self.cursor.y() + 1.into()).unwrap_or(self.cursor);
                App { cursor, ..self }
            }
        }
    }

    fn move_left(self, options: Option<Vec<Point>>) -> Self {
        match options {
            Some(options) => {
                let cursor_x = self.cursor.x();
                self.move_into_list(options, |point| point.x() < cursor_x)
            }
            None => {
                let cursor = Point::new_(self.cursor.x() + (-1).into(), self.cursor.y())
                    .unwrap_or(self.cursor);
                App { cursor, ..self }
            }
        }
    }

    fn move_right(self, options: Option<Vec<Point>>) -> Self {
        match options {
            Some(options) => {
                let cursor_x = self.cursor.x();
                self.move_into_list(options, |point| point.x() > cursor_x)
            }
            None => {
                let cursor =
                    Point::new_(self.cursor.x() + 1.into(), self.cursor.y()).unwrap_or(self.cursor);
                App { cursor, ..self }
            }
        }
    }
}

impl<T: GameState + NormalState + Clone> App<T> {
    fn pawn_at(&self, loc: Point) -> Option<Pawn<T>> {
        for pawn in self.game.active_pawns().iter() {
            if pawn.pos() == loc {
                return Some(pawn.clone());
            }
        }
        None
    }
}

fn new_app() -> Box<dyn Screen> {
    Box::new(App {
        game: santorini::new_game(),
        cursor: Point::new(0.into(), 0.into()),
        intermediate_loc: None,
    })
}

impl Screen for App<PlaceOne> {
    fn update(self: Box<Self>, terminal: &mut Term) -> Result<Box<dyn Screen>, UpdateError> {
        self.draw(
            terminal,
            BoardWidget {
                board: self.game.board(),
                player: self.game.player(),
                cursor: self.cursor,

                highlights: vec![],
                player1_locs: self.intermediate_loc.iter().cloned().collect(),
                player2_locs: vec![],
            },
        )?;

        if let Some(event) = io::stdin().events().next() {
            match event? {
                Event::Key(Key::Ctrl('c')) => Err(UpdateError::Shutdown),
                Event::Key(Key::Char('q')) | Event::Key(Key::Esc) => {
                    if self.intermediate_loc.is_none() {
                        Err(UpdateError::Shutdown)
                    } else {
                        Ok(Box::new(App {
                            intermediate_loc: None,
                            ..*self
                        }))
                    }
                }
                Event::Key(Key::Char('\n')) => {
                    if let Some(pos1) = self.intermediate_loc {
                        if pos1 != self.cursor {
                            let position = self.game.can_place(pos1, self.cursor).unwrap();
                            Ok(Box::new(App {
                                game: self.game.apply(position),
                                intermediate_loc: None,
                                cursor: self.cursor,
                            }))
                        } else {
                            Ok(self)
                        }
                    } else {
                        Ok(Box::new(App {
                            intermediate_loc: Some(self.cursor),
                            ..*self
                        }))
                    }
                }
                Event::Key(Key::Up) | Event::Key(Key::Char('w')) => {
                    Ok(Box::new(self.move_up(None)))
                }
                Event::Key(Key::Left) | Event::Key(Key::Char('a')) => {
                    Ok(Box::new(self.move_left(None)))
                }
                Event::Key(Key::Down) | Event::Key(Key::Char('s')) => {
                    Ok(Box::new(self.move_down(None)))
                }
                Event::Key(Key::Right) | Event::Key(Key::Char('d')) => {
                    Ok(Box::new(self.move_right(None)))
                }
                _ => Ok(self),
            }
        } else {
            Ok(self)
        }
    }
}

impl Screen for App<PlaceTwo> {
    fn update(self: Box<Self>, terminal: &mut Term) -> Result<Box<dyn Screen>, UpdateError> {
        self.draw(
            terminal,
            BoardWidget {
                board: self.game.board(),
                player: self.game.player(),
                cursor: self.cursor,

                highlights: vec![],
                player1_locs: self.game.player1_locs().to_vec(),
                player2_locs: self.intermediate_loc.iter().cloned().collect(),
            },
        )?;

        if let Some(event) = io::stdin().events().next() {
            match event? {
                Event::Key(Key::Ctrl('c')) => Err(UpdateError::Shutdown),
                Event::Key(Key::Char('q')) | Event::Key(Key::Esc) => {
                    if self.intermediate_loc.is_none() {
                        Err(UpdateError::Shutdown)
                    } else {
                        Ok(Box::new(App {
                            intermediate_loc: None,
                            ..*self
                        }))
                    }
                }
                Event::Key(Key::Char('\n')) => {
                    for pos in self.game.player1_locs().iter() {
                        if *pos == self.cursor {
                            return Ok(self);
                        }
                    }

                    if let Some(pos1) = self.intermediate_loc {
                        if pos1 != self.cursor {
                            let position = self.game.can_place(pos1, self.cursor).unwrap();
                            let new_game = self.game.apply(position);
                            Ok(Box::new(App {
                                cursor: new_game.active_pawns()[0].pos(),
                                game: new_game,
                                intermediate_loc: None,
                            }))
                        } else {
                            Ok(self)
                        }
                    } else {
                        Ok(Box::new(App {
                            intermediate_loc: Some(self.cursor),
                            ..*self
                        }))
                    }
                }
                Event::Key(Key::Up) | Event::Key(Key::Char('w')) => {
                    Ok(Box::new(self.move_up(None)))
                }
                Event::Key(Key::Left) | Event::Key(Key::Char('a')) => {
                    Ok(Box::new(self.move_left(None)))
                }
                Event::Key(Key::Down) | Event::Key(Key::Char('s')) => {
                    Ok(Box::new(self.move_down(None)))
                }
                Event::Key(Key::Right) | Event::Key(Key::Char('d')) => {
                    Ok(Box::new(self.move_right(None)))
                }
                _ => Ok(self),
            }
        } else {
            Ok(self)
        }
    }
}

impl App<Move> {
    fn active_pawn(&self) -> Option<Pawn<Move>> {
        if let Some(loc) = self.intermediate_loc {
            self.pawn_at(loc)
        } else {
            None
        }
    }
}

impl Screen for App<Move> {
    fn update(self: Box<Self>, terminal: &mut Term) -> Result<Box<dyn Screen>, UpdateError> {
        let highlights: Vec<Point> = if let Some(pawn) = self.active_pawn() {
            pawn.actions().iter().map(|pair| pair.to()).collect()
        } else {
            self.game
                .active_pawns()
                .iter()
                .map(|pawn| pawn.pos())
                .collect()
        };
        self.draw(
            terminal,
            BoardWidget {
                board: self.game.board(),
                player: self.game.player(),
                cursor: self.cursor,

                highlights: highlights.clone(),
                player1_locs: self
                    .game
                    .player1_pawns()
                    .iter()
                    .map(|pawn| pawn.pos())
                    .collect(),
                player2_locs: self
                    .game
                    .player2_pawns()
                    .iter()
                    .map(|pawn| pawn.pos())
                    .collect(),
            },
        )?;

        if let Some(event) = io::stdin().events().next() {
            match event? {
                Event::Key(Key::Ctrl('c')) => Err(UpdateError::Shutdown),
                Event::Key(Key::Char('q')) | Event::Key(Key::Esc) => {
                    if let Some(pawn_loc) = self.intermediate_loc {
                        Ok(Box::new(App {
                            intermediate_loc: None,
                            cursor: pawn_loc,
                            ..*self
                        }))
                    } else {
                        Err(UpdateError::Shutdown)
                    }
                }
                Event::Key(Key::Char('\n')) => {
                    if let Some(pawn) = self.active_pawn() {
                        let action = pawn.can_move(self.cursor).unwrap();
                        //match self.game.apply(action) {
                        //    ActionResult::Continue(game) => Ok(Box::new(App {
                        //            cursor: game.active_pawns()[0].pos(),
                        //            game,
                        //            intermediate_loc: None,
                        //        })),
                        //    ActionResult::Victory(game) => Ok(Box::new(App {
                        //            cursor: self.cursor,
                        //            game,
                        //            intermediate_loc: None,
                        //        })),
                        //}
                        Ok(self)
                    } else {
                        let pawn = self.pawn_at(self.cursor).unwrap();
                        if let Some(action) = pawn.actions().iter().next() {
                            Ok(Box::new(App {
                                intermediate_loc: Some(self.cursor),
                                cursor: action.to(),
                                ..*self
                            }))
                        } else {
                            Ok(self)
                        }
                    }
                }
                Event::Key(Key::Up) | Event::Key(Key::Char('w')) => {
                    Ok(Box::new(self.move_up(Some(highlights))))
                }
                Event::Key(Key::Left) | Event::Key(Key::Char('a')) => {
                    Ok(Box::new(self.move_left(Some(highlights))))
                }
                Event::Key(Key::Down) | Event::Key(Key::Char('s')) => {
                    Ok(Box::new(self.move_down(Some(highlights))))
                }
                Event::Key(Key::Right) | Event::Key(Key::Char('d')) => {
                    Ok(Box::new(self.move_right(Some(highlights))))
                }
                _ => Ok(self),
            }
        } else {
            Ok(self)
        }
    }
}

fn main() -> Result<(), UpdateError> {
    let stdout = MouseTerminal::from(io::stdout().into_raw_mode()?);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut app = new_app();

    terminal.clear()?;
    loop {
        app = app.update(&mut terminal)?;
    }
}
