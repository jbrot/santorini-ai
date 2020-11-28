use tui::style::{Color, Modifier, Style};
mod board;
mod bounds;

pub use board::BoardWidget;
pub use bounds::BoundsWidget;

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
