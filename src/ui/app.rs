use std::io;
use termion::event::{Event, Key};
use termion::input::TermRead;
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::{Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use tui::Frame;

use crate::santorini::{
    self, ActionResult, Build, Game, GameState, Move, NormalState, Pawn, PlaceOne, PlaceTwo,
    Player, Point, Victory,
};

use crate::ui::{
    self, Back, BoardWidget, Screen, Term, UpdateError, PLAYER_ONE_TEXT_STYLE,
    PLAYER_TWO_TEXT_STYLE,
};

pub struct App<T: GameState> {
    game: Game<T>,
    cursor: Point,
    intermediate_loc: Option<Point>,
}

impl<T: GameState> App<T> {
    fn current_player_name(&self) -> Span {
        if self.game.player() == Player::PlayerOne {
            Span::styled("Player One", PLAYER_ONE_TEXT_STYLE)
        } else {
            Span::styled("Player Two", PLAYER_TWO_TEXT_STYLE)
        }
    }

    fn do_draw(&self, frame: &mut Frame<Back>, widget: BoardWidget, title: Spans) -> Rect {
        let border = Block::default().title("Santorini").borders(Borders::ALL);
        frame.render_widget(border, frame.size());

        let segments = Layout::default()
            .direction(Direction::Horizontal)
            .margin(1)
            .constraints([Constraint::Min(15), Constraint::Ratio(1, 3)].as_ref())
            .split(frame.size());

        frame.render_widget(
            Paragraph::new(vec![Spans::from(vec![]), title])
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: false }),
            segments[0],
        );
        frame.render_widget(widget, segments[0]);

        let bold = Style::default().add_modifier(Modifier::BOLD);
        let instructions = vec![
            Spans::from(vec![]),
            Spans::from(vec![
                Span::raw("Use arrow keys or "),
                Span::styled("WASD", bold),
                Span::raw(" to move cursor."),
            ]),
            Spans::from(vec![]),
            Spans::from(vec![
                Span::raw("Use "),
                Span::styled("Enter", bold),
                Span::raw(" or "),
                Span::styled("e", bold),
                Span::raw(" to select."),
            ]),
            Spans::from(vec![]),
            Spans::from(vec![
                Span::raw("Use "),
                Span::styled("Esc", bold),
                Span::raw(" or "),
                Span::styled("q", bold),
                Span::raw(" to deselect."),
            ]),
            Spans::from(vec![]),
            Spans::from(vec![
                Span::raw("Use "),
                Span::styled("Ctrl C", bold),
                Span::raw(" to quit."),
            ]),
        ];
        frame.render_widget(
            Paragraph::new(instructions)
                .block(Block::default().title("Instructions").borders(Borders::ALL))
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: false }),
            segments[1],
        );

        segments[0]
    }

    fn draw(
        &self,
        terminal: &mut Term,
        widget: BoardWidget,
        title: Spans,
    ) -> Result<(), UpdateError> {
        terminal.draw(|f| {
            self.do_draw(f, widget, title);
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

pub fn new_app() -> Box<dyn Screen> {
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
            Spans::from(vec![self.current_player_name(), Span::raw(" to place")]),
        )?;

        if let Some(event) = io::stdin().events().next() {
            match event? {
                Event::Key(Key::Ctrl('c')) => Err(UpdateError::Shutdown),
                Event::Key(Key::Char('q')) | Event::Key(Key::Esc) => {
                    if self.intermediate_loc.is_none() {
                        Ok(self)
                    } else {
                        Ok(Box::new(App {
                            intermediate_loc: None,
                            ..*self
                        }))
                    }
                }
                Event::Key(Key::Char('\n')) | Event::Key(Key::Char('e')) => {
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
            Spans::from(vec![self.current_player_name(), Span::raw(" to place")]),
        )?;

        if let Some(event) = io::stdin().events().next() {
            match event? {
                Event::Key(Key::Ctrl('c')) => Err(UpdateError::Shutdown),
                Event::Key(Key::Char('q')) | Event::Key(Key::Esc) => {
                    if self.intermediate_loc.is_none() {
                        Ok(self)
                    } else {
                        Ok(Box::new(App {
                            intermediate_loc: None,
                            ..*self
                        }))
                    }
                }
                Event::Key(Key::Char('\n')) | Event::Key(Key::Char('e')) => {
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
            Spans::from(vec![self.current_player_name(), Span::raw(" to move")]),
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
                        Ok(self)
                    }
                }
                Event::Key(Key::Char('\n')) | Event::Key(Key::Char('e')) => {
                    if let Some(pawn) = self.active_pawn() {
                        let action = pawn.can_move(self.cursor).unwrap();
                        match self.game.apply(action) {
                            ActionResult::Continue(game) => Ok(Box::new(App {
                                cursor: game.active_pawn().actions()[0].loc(),
                                game,
                                intermediate_loc: None,
                            })),
                            ActionResult::Victory(game) => Ok(Box::new(App {
                                cursor: self.cursor,
                                game,
                                intermediate_loc: None,
                            })),
                        }
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

impl Screen for App<Build> {
    fn update(self: Box<Self>, terminal: &mut Term) -> Result<Box<dyn Screen>, UpdateError> {
        let highlights: Vec<Point> = self
            .game
            .active_pawn()
            .actions()
            .iter()
            .map(|pair| pair.loc())
            .collect();
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
            Spans::from(vec![self.current_player_name(), Span::raw(" to build")]),
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
                        Ok(self)
                    }
                }
                Event::Key(Key::Char('\n')) | Event::Key(Key::Char('e')) => {
                    let action = self.game.active_pawn().can_build(self.cursor).unwrap();
                    match self.game.apply(action) {
                        ActionResult::Continue(game) => Ok(Box::new(App {
                            cursor: game.active_pawns()[0].pos(),
                            game,
                            intermediate_loc: None,
                        })),
                        ActionResult::Victory(game) => Ok(Box::new(App {
                            cursor: self.cursor,
                            game,
                            intermediate_loc: None,
                        })),
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

impl Screen for App<Victory> {
    fn update(self: Box<Self>, terminal: &mut Term) -> Result<Box<dyn Screen>, UpdateError> {
        terminal.draw(|f| {
            let widget = BoardWidget {
                board: self.game.board(),
                player: self.game.player(),
                cursor: self.cursor,

                highlights: vec![],
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
            };
            let game_rect = self.do_draw(f, widget, Spans::from(vec![]));
            let announce_width = 20;
            let announce_height = 7;
            let x_off = (game_rect.width - announce_width) / 2;
            let y_off = (game_rect.height - announce_height) / 2;
            let announce_rect = Rect::new(
                game_rect.x + x_off,
                game_rect.y + y_off,
                announce_width,
                announce_height,
            );
            f.render_widget(Clear, announce_rect);

            let text = vec![
                Spans::from(vec![
                    self.current_player_name(),
                    Span::styled(" wins!", Style::default().add_modifier(Modifier::BOLD)),
                ]),
                Spans::from(vec![]),
                Spans::from(vec![]),
                Spans::from(Span::raw("Press any key to continue...")),
            ];
            f.render_widget(
                Paragraph::new(text)
                    .block(Block::default().borders(Borders::ALL))
                    .alignment(Alignment::Center)
                    .wrap(Wrap { trim: false }),
                announce_rect,
            );
        })?;

        if let Some(event) = io::stdin().events().next() {
            match event? {
                Event::Key(Key::Ctrl('c')) | Event::Key(Key::Char('q')) | Event::Key(Key::Esc) => {
                    Err(UpdateError::Shutdown)
                }
                Event::Key(_) => Ok(ui::main_menu()),
                _ => Ok(self),
            }
        } else {
            Ok(self)
        }
    }
}
