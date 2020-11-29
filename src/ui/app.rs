use std::io;
use termion::event::{Event, Key};
use termion::input::TermRead;
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::{Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use tui::Frame;

use crate::santorini::{self, Build, Game, GameState, Move, PlaceOne, PlaceTwo, Player, Victory};

use crate::ui::{
    self, Back, BoardWidget, Screen, Term, UpdateError, PLAYER_ONE_TEXT_STYLE,
    PLAYER_TWO_TEXT_STYLE,
};

use crate::player::{self, FullPlayer, StepResult};

pub struct App<T: GameState> {
    game: Game<T>,
    player_one: Box<dyn FullPlayer>,
    player_two: Box<dyn FullPlayer>,
}

impl<T: GameState> App<T> {
    fn current_player_name(&self) -> Span {
        match self.game.player() {
            Player::PlayerOne => Span::styled("Player One", PLAYER_ONE_TEXT_STYLE),
            Player::PlayerTwo => Span::styled("Player Two", PLAYER_TWO_TEXT_STYLE),
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
                Span::styled("F6", bold),
                Span::raw(" to resign."),
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

    fn transition<U>(mut self, game: Game<U>) -> App<U> where
        U: GameState,
        dyn FullPlayer: player::Player<U> {
        match game.player() {
            Player::PlayerOne => self.player_one.prepare(&game),
            Player::PlayerTwo => self.player_two.prepare(&game),
        };

        App {
            game,
            player_one: self.player_one,
            player_two: self.player_two,
        }
    }
}

pub fn new_app(
    player_one: Box<dyn FullPlayer>,
    player_two: Box<dyn FullPlayer>,
) -> Box<dyn Screen> {
    Box::new(App {
        game: santorini::new_game(),
        player_one,
        player_two,
    })
}

macro_rules! standard_state {
    ($state:ty) => {
        impl Screen for App<$state> {
            fn update(
                mut self: Box<Self>,
                terminal: &mut Term,
            ) -> Result<Box<dyn Screen>, UpdateError> {
                let active_player = match self.game.player() {
                    Player::PlayerOne => &self.player_one,
                    Player::PlayerTwo => &self.player_two,
                };

                terminal.draw(|f| {
                    self.do_draw(
                        f,
                        active_player.render(&self.game),
                        Spans::from(vec![self.current_player_name(), Span::raw(" to place")]),
                    );
                })?;

                let active_player = match self.game.player() {
                    Player::PlayerOne => &mut self.player_one,
                    Player::PlayerTwo => &mut self.player_two,
                };

                match active_player.step(&self.game)? {
                    StepResult::NoMove => Ok(self),
                    StepResult::PlaceTwo(game) => Ok(Box::new(self.transition(game))),
                    StepResult::Move(game) => Ok(Box::new(self.transition(game))),
                    StepResult::Build(game) => Ok(Box::new(self.transition(game))),
                    StepResult::Victory(game) => Ok(Box::new(App {
                        game,
                        player_one: self.player_one,
                        player_two: self.player_two,
                    })),
                }
            }
        }
    };
}

standard_state!(PlaceOne);
standard_state!(PlaceTwo);
standard_state!(Move);
standard_state!(Build);

impl Screen for App<Victory> {
    fn update(self: Box<Self>, terminal: &mut Term) -> Result<Box<dyn Screen>, UpdateError> {
        terminal.draw(|f| {
            let widget = BoardWidget {
                board: self.game.board(),
                player: self.game.player(),
                cursor: None,

                highlights: &vec![],
                player1_locs: self
                    .game
                    .player_pawns(santorini::Player::PlayerOne)
                    .iter()
                    .map(|pawn| pawn.pos())
                    .collect(),
                player2_locs: self
                    .game
                    .player_pawns(santorini::Player::PlayerTwo)
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
