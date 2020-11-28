use std::io;
use termion::raw::{IntoRawMode, RawTerminal};
use tui::backend::TermionBackend;
use tui::buffer::Buffer;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::{Color, Style};
use tui::widgets::{Block, Borders, Widget};
use tui::Terminal;

mod santorini;
use santorini::{
    Board, Coord, Game, GameState, PlaceOne, PlaceTwo, Player, Point, BOARD_HEIGHT, BOARD_WIDTH,
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

const PLAYER_ONE_COLOR: Color = Color::Indexed(63);
const PLAYER_ONE_CURSOR_COLOR: Color = Color::Indexed(21);
const PLAYER_ONE_HIGHLIGHT_COLOR: Color = Color::Indexed(26);

const PLAYER_TWO_COLOR: Color = Color::Indexed(161);
const PLAYER_TWO_CURSOR_COLOR: Color = Color::Indexed(196);
const PLAYER_TWO_HIGHLIGHT_COLOR: Color = Color::Indexed(204);

impl BoardWidget {
    fn color(&self, point: Point) -> Option<Color> {
        for p in &self.player1_locs {
            if point == *p {
                return Some(PLAYER_ONE_COLOR);
            }
        }

        for p in &self.player2_locs {
            if point == *p {
                return Some(PLAYER_TWO_COLOR);
            }
        }

        None
    }

    fn border_color(&self, point: Point) -> Option<Color> {
        if point == self.cursor {
            if self.player == Player::PlayerOne {
                return Some(PLAYER_ONE_CURSOR_COLOR);
            } else {
                return Some(PLAYER_TWO_CURSOR_COLOR);
            }
        }

        for p in &self.highlights {
            if *p == point {
                if self.player == Player::PlayerOne {
                    return Some(PLAYER_ONE_HIGHLIGHT_COLOR);
                } else {
                    return Some(PLAYER_TWO_HIGHLIGHT_COLOR);
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
                match self.color(point) {
                    Some(color) => block = block.style(Style::default().bg(color)),
                    None => (),
                }
                match self.border_color(point) {
                    Some(color) => block = block.border_style(Style::default().fg(color).bg(color)),
                    None => (),
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

type Term = Terminal<TermionBackend<RawTerminal<io::Stdout>>>;

trait Screen {
    fn update(self: Box<Self>, terminal: &mut Term) -> Result<Box<dyn Screen>, io::Error>;
}

struct App<T: GameState> {
    game: Game<T>,
    cursor: Point,
}

impl Screen for App<PlaceOne> {
    fn update(self: Box<Self>, terminal: &mut Term) -> Result<Box<dyn Screen>, io::Error> {
        terminal.draw(|f| {
            let border = Block::default().title("Santorini").borders(Borders::ALL);
            f.render_widget(border, f.size());

            let segments = Layout::default()
                .direction(Direction::Horizontal)
                .margin(1)
                .constraints([Constraint::Min(15), Constraint::Ratio(1, 3)].as_ref())
                .split(f.size());
            let game = BoardWidget {
                board: self.game.board(),
                player: self.game.player(),
                cursor: self.cursor,

                highlights: vec![],
                player1_locs: vec![],
                player2_locs: vec![],
            };
            f.render_widget(game, segments[0]);
            let side_text = Block::default().title("Side Text").borders(Borders::ALL);
            f.render_widget(side_text, segments[1]);
        })?;
        Ok(self)
    }
}

fn new_app() -> Box<dyn Screen> {
    Box::new(App {
        game: santorini::new_game(),
        cursor: Point::new(0.into(), 0.into()),
    })
}

fn main() -> Result<(), io::Error> {
    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut app = new_app();

    terminal.clear()?;
    loop {
        app = app.update(&mut terminal)?;
    }
}
