use std::io;
use std::rc::Rc;
use termion::event::{Event, Key};
use termion::input::TermRead;
use tui::buffer::Buffer;
use tui::layout::{Alignment, Constraint, Direction, Layout, Margin, Rect};
use tui::style::{Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, Borders, Clear, Paragraph, Widget, Wrap};
use tui::Frame;

use crate::santorini::{
    self, ActionResult, Build, Game, GameState, Move, NormalState, Pawn, PlaceOne, PlaceTwo,
    Player, Point, Victory,
};

use crate::ui::{
    Back, BoardWidget, BoundsWidget, Screen, Term, UpdateError, PLAYER_ONE_TEXT_STYLE, PLAYER_TWO_TEXT_STYLE,
};

#[derive(Clone)]
pub struct MenuWidget<'a> {
    items: Vec<Spans<'a>>,
    cursor: usize,
    bounds: BoundsWidget,
}

impl<'a> MenuWidget<'a> {
    pub fn new(items: Vec<Spans<'a>>) -> MenuWidget<'a> {
        let len = items.len();
        assert!(len > 0);
        let min_width = items.iter().map(|item| item.width()).max().unwrap();
        MenuWidget {
            items,
            cursor: 0,
            bounds: BoundsWidget {
                min_width: min_width as u16,
                min_height: len as u16,
            }
        }
    }

    pub fn move_up(&mut self) {
        if self.cursor > 0 {
            self.cursor = self.cursor - 1;
        } else {
            self.cursor = self.items.len() - 1;
        }
    }

    pub fn move_down(&mut self) {
        let cursor = self.cursor + 1;
        if cursor == self.items.len() {
            self.cursor = 0;
        } else {
            self.cursor = cursor;
        }
    }

    pub fn selected(&self) -> usize {
        self.cursor
    }

    pub fn selected_item(&self) -> &Spans {
        &self.items[self.cursor]
    }

    pub fn move_menu_widget<'b>(self) -> MenuWidget<'b> {
        MenuWidget {
            items: self.items.into_iter().map(|spans| move_spans(spans)).collect(),
            cursor: self.cursor,
            bounds: self.bounds,
        }
    }
}

impl<'a> Widget for MenuWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Clear.render(area, buf);

        if !self.bounds.can_fit(area) {
            self.bounds.render(area, buf);
            return;
        }

        let double = area.height > 2 * self.bounds.min_height;
        let height = if double { 2 * self.bounds.min_height } else { self.bounds.min_height };
        let mut text = Vec::new();
        for (index, entry) in self.items.into_iter().enumerate() {
            if index == self.cursor {
                let new_spans: Vec<Span> = entry.0
                    .into_iter()
                    .map(|span| Span::styled(span.content, span.style.add_modifier(Modifier::REVERSED)))
                    .collect();
                text.push(Spans::from(new_spans));
            } else {
                text.push(entry.clone());
            }

            if double {
                text.push(Spans::from(vec![]));
            }
        }

        let top = area.top() + (area.height - height) / 2;
        let text_area = Rect { x: area.left(), y: top, width: area.width, height };
        Paragraph::new(text)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true })
            .render(text_area, buf);
    }
}

pub struct Menu<'a, T> {
    menu_widget: MenuWidget<'a>,
    actions: Vec<Box<dyn FnOnce() -> T>>,
}

impl<'a, T> Menu<'a, T> {
    pub fn new(items: Vec<(Spans<'a>, Box<dyn FnOnce() -> T>)>) -> Menu<'a, T> {
        let (items, actions): (Vec<_>, Vec<_>) = items.into_iter().unzip();
        Menu {
            menu_widget: MenuWidget::new(items),
            actions
        }
    }

    pub fn move_up(&mut self) {
        self.menu_widget.move_up()
    }

    pub fn move_down(&mut self) {
        self.menu_widget.move_down()
    }

    pub fn select(self) -> T {
        self.actions
            .into_iter()
            .nth(self.menu_widget.selected())
            .unwrap()()
    }

    pub fn move_menu<'b>(self) -> Menu<'b, T> {
        Menu {
            menu_widget: self.menu_widget.move_menu_widget(),
            actions: self.actions,
        }
    }
}

fn move_span<'a, 'b>(span: Span<'a>) -> Span<'b> {
    Span {
        content: span.content.into_owned().into(),
        style: span.style,
    }
}

fn move_spans<'a, 'b>(spans: Spans<'a>) -> Spans<'b> {
    let new_spans: Vec<_> = spans.0
        .into_iter()
        .map(|span| move_span(span))
        .collect();
    Spans::from(new_spans)
}

impl<'a> Screen for Menu<'a, Result<Box<dyn Screen>, UpdateError>> {
    fn update(mut self: Box<Self>, terminal: &mut Term) -> Result<Box<dyn Screen>, UpdateError> {
        terminal.draw(|f| {
            let border = Block::default().title("Santorini").borders(Borders::ALL);
            f.render_widget(border, f.size());
            let menu_area = f.size().inner(&Margin { horizontal: 1, vertical: 1 });
            f.render_widget(self.menu_widget.clone(), menu_area)
        })?;
        if let Some(event) = io::stdin().events().next() {
            match event? {
                Event::Key(Key::Ctrl('c')) => Err(UpdateError::Shutdown),
                Event::Key(Key::Char('\n')) => self.select(),
                Event::Key(Key::Up) | Event::Key(Key::Char('w')) => {
                    self.move_up();
                    Ok(Box::new(self.move_menu()))
                }
                Event::Key(Key::Down) | Event::Key(Key::Char('s')) => {
                    self.move_down();
                    Ok(Box::new(self.move_menu()))
                }
                _ => Ok(Box::new(self.move_menu()))
            }
        } else {
            Ok(Box::new(self.move_menu()))
        }
    }
}
