use tui::buffer::Buffer;
use tui::layout::Rect;
use tui::style::Style;
use tui::widgets::{Block, Borders, Clear, Widget};

use crate::santorini::{Board, Coord, CoordLevel, Player, Point, BOARD_HEIGHT, BOARD_WIDTH};

use crate::ui::{
    BoundsWidget, CAPPED_STYLE, GROUND_LEVEL_STYLE, LEVEL_ONE_STYLE, LEVEL_THREE_STYLE,
    LEVEL_TWO_STYLE, PLAYER_ONE_CURSOR_STYLE, PLAYER_ONE_HIGHLIGHT_STYLE, PLAYER_ONE_STYLE,
    PLAYER_TWO_CURSOR_STYLE, PLAYER_TWO_HIGHLIGHT_STYLE, PLAYER_TWO_STYLE,
};

pub struct BoardWidget<'a> {
    pub board: Board,

    pub player: Player,
    pub cursor: Option<Point>,
    pub highlights: &'a Vec<Point>,

    pub player1_locs: Vec<Point>,
    pub player2_locs: Vec<Point>,
}

const SQUARE_SIZE: u16 = 5;
const BOARD_WIDGET_WIDTH: u16 = (BOARD_WIDTH.0 as u16) * SQUARE_SIZE;
const BOARD_WIDGET_HEIGHT: u16 = (BOARD_HEIGHT.0 as u16) * SQUARE_SIZE;

impl<'a> BoardWidget<'a> {
    fn style(&self, point: Point) -> Style {
        for p in &self.player1_locs {
            if point == *p {
                return PLAYER_ONE_STYLE;
            }
        }

        for p in &self.player2_locs {
            if point == *p {
                return PLAYER_TWO_STYLE;
            }
        }

        match self.board.level_at(point) {
            CoordLevel::Ground => GROUND_LEVEL_STYLE,
            CoordLevel::One => LEVEL_ONE_STYLE,
            CoordLevel::Two => LEVEL_TWO_STYLE,
            CoordLevel::Three => LEVEL_THREE_STYLE,
            CoordLevel::Capped => CAPPED_STYLE,
        }
    }

    fn border_style(&self, point: Point) -> Option<Style> {
        if Some(point) == self.cursor {
            if self.player == Player::PlayerOne {
                return Some(PLAYER_ONE_CURSOR_STYLE);
            } else {
                return Some(PLAYER_TWO_CURSOR_STYLE);
            }
        }

        for p in self.highlights {
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

impl<'a> Widget for BoardWidget<'a> {
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
        Clear.render(
            Rect::new(left, top, BOARD_WIDGET_WIDTH, BOARD_WIDGET_HEIGHT),
            buf,
        );

        for x in 0..BOARD_WIDTH.0 as u16 {
            for y in 0..BOARD_HEIGHT.0 as u16 {
                let area = Rect {
                    x: left + x * SQUARE_SIZE,
                    y: top + y * SQUARE_SIZE,
                    width: SQUARE_SIZE,
                    height: SQUARE_SIZE,
                };
                let point = Point::new(Coord::from(x as i8), Coord::from(y as i8));
                let mut block = Block::default()
                    .borders(Borders::ALL)
                    .style(self.style(point));
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
