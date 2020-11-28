use tui::buffer::Buffer;
use tui::layout::Rect;
use tui::style::Style;
use tui::widgets::Widget;

#[derive(Clone, Copy)]
pub struct BoundsWidget {
    pub min_width: u16,
    pub min_height: u16,
}

impl BoundsWidget {
    pub fn can_fit(&self, area: Rect) -> bool {
        area.width >= self.min_width && area.height >= self.min_height
    }
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
