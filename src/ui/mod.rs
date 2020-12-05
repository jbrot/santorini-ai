use std::boxed::Box;
use std::io;
use termion::input::MouseTerminal;
use termion::raw::RawTerminal;
use thiserror::Error;
use tui::backend::TermionBackend;
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui::Terminal;

use crate::player::{HumanPlayer, MctsSantoriniParams};

mod app;
mod board;
mod bounds;
mod menu;

pub use app::{new_app, App};
pub use board::BoardWidget;
pub use bounds::BoundsWidget;
pub use menu::{Menu, MenuWidget};

pub type Back = TermionBackend<MouseTerminal<RawTerminal<io::Stdout>>>;
pub type Term = Terminal<Back>;

#[derive(Error, Debug)]
pub enum UpdateError {
    #[error("issue updating display")]
    IoError(#[from] io::Error),
    #[error("normal exit")]
    Shutdown,
}

pub trait Screen {
    fn update(self: Box<Self>, terminal: &mut Term) -> Result<Box<dyn Screen>, UpdateError>;
}

pub fn main_menu<'a>() -> Box<dyn Screen> {
    Box::new(Menu::new(
        Span::styled("Santorini", Style::default().add_modifier(Modifier::BOLD)).into(),
        vec![
            (
                Spans::from("2 Player Game"),
                Box::new(|| Ok(new_app(HumanPlayer::new(), HumanPlayer::new()))),
            ),
            (
                Spans::from("1 Player Game"),
                Box::new(|| {
                    Ok(new_app(
                        HumanPlayer::new(),
                        MctsSantoriniParams::default().boxed(),
                    ))
                }),
            ),
            (Spans::from("Quit"), Box::new(|| Err(UpdateError::Shutdown))),
        ],
    ))
}

pub const PLAYER_ONE_STYLE: Style = Style {
    bg: Some(Color::Indexed(21)),
    fg: Some(Color::White),
    ..DEFAULT_STYLE
};
pub const PLAYER_ONE_TEXT_STYLE: Style = Style {
    bg: None,
    fg: Some(Color::Indexed(21)),
    add_modifier: Modifier::BOLD,
    ..DEFAULT_STYLE
};
pub const PLAYER_ONE_CURSOR_STYLE: Style = Style {
    bg: Some(Color::Indexed(45)),
    fg: Some(Color::Black),
    ..DEFAULT_STYLE
};
pub const PLAYER_ONE_HIGHLIGHT_STYLE: Style = Style {
    bg: Some(Color::Indexed(33)),
    fg: Some(Color::Indexed(33)),
    ..DEFAULT_STYLE
};

pub const PLAYER_TWO_STYLE: Style = Style {
    bg: Some(Color::Indexed(160)),
    fg: Some(Color::White),
    ..DEFAULT_STYLE
};
pub const PLAYER_TWO_TEXT_STYLE: Style = Style {
    bg: None,
    fg: Some(Color::Indexed(160)),
    add_modifier: Modifier::BOLD,
    ..DEFAULT_STYLE
};
pub const PLAYER_TWO_CURSOR_STYLE: Style = Style {
    bg: Some(Color::Indexed(213)),
    fg: Some(Color::Black),
    ..DEFAULT_STYLE
};
pub const PLAYER_TWO_HIGHLIGHT_STYLE: Style = Style {
    bg: Some(Color::Indexed(204)),
    fg: Some(Color::Indexed(204)),
    ..DEFAULT_STYLE
};

const DEFAULT_STYLE: Style = Style {
    bg: None,
    fg: None,
    add_modifier: Modifier::empty(),
    sub_modifier: Modifier::empty(),
};

pub const GROUND_LEVEL_STYLE: Style = DEFAULT_STYLE;
pub const LEVEL_ONE_STYLE: Style = Style {
    bg: Some(Color::Indexed(250)),
    fg: Some(Color::Black),
    ..DEFAULT_STYLE
};
pub const LEVEL_TWO_STYLE: Style = Style {
    bg: Some(Color::Indexed(245)),
    fg: Some(Color::White),
    ..DEFAULT_STYLE
};
pub const LEVEL_THREE_STYLE: Style = Style {
    bg: Some(Color::Indexed(240)),
    fg: Some(Color::White),
    ..DEFAULT_STYLE
};
pub const CAPPED_STYLE: Style = Style {
    bg: Some(Color::Indexed(235)),
    fg: Some(Color::Indexed(235)),
    ..DEFAULT_STYLE
};
