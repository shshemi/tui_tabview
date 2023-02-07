use std::cmp::{max};
use std::collections::HashSet;
use tui::buffer::Buffer;
use tui::layout::{Rect};
use tui::style::Style;
use tui::text::{Text};
use tui::widgets::{StatefulWidget};


#[derive(Default)]
pub enum Selection {
    Cell(usize, usize),
    Row(usize),
    Col(usize),
    #[default]
    None,
}

#[derive(Default)]
pub struct State {
    pub offset: (usize, usize),
    pub selection: Selection,
    pub text_cut: (bool, bool),
    pub last_cell: (usize, usize),
    pub expanded_columns: HashSet<usize>,
}


pub struct ModelTable<'a, T, U> {
    data_source: &'a T,
    styler: &'a U,
}

pub trait DataSource {
    fn value(&self, row: usize, col: usize) -> &str;
    fn shape(&self) -> (usize, usize);
    fn max_widths(&self) -> &[u16];
}

pub trait Styler {
    fn normal(&self, row: usize, col: usize) -> Style;
    fn highlight(&self, row: usize, col: usize) -> Style;
    fn row_spacing(&self) -> u16;
    fn col_spacing(&self) -> u16;
}

impl<T, U> ModelTable<'_, T, U> where
    T: DataSource,
    U: Styler
{
    pub fn new<'a>(data_source: &'a T, styler: &'a U) -> ModelTable<'a, T, U> {
        ModelTable {
            data_source,
            styler,
        }
    }

    fn text(&self, row: usize, col: usize, highlight: bool) -> Text {
        if highlight {
            Text::styled(self.data_source.value(row, col),
                         self.styler.highlight(row, col))
        } else {
            Text::styled(self.data_source.value(row, col),
                         self.styler.normal(row, col))
        }
    }
}


impl<T, U> StatefulWidget for ModelTable<'_, T, U> where
    T: DataSource,
    U: Styler
{
    type State = State;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let ds_shape = self.data_source.shape();
        let widths = &self.data_source.max_widths()[state.offset.1..];
        let shrunk_width = 24 as u16;
        state.text_cut.0 = false;
        state.text_cut.1 = false;
        state.last_cell.0 = 0;
        state.last_cell.1 = 0;
        let mut y = area.y;
        for row in state.offset.0..ds_shape.0 {
            let mut lines = 1;
            let mut x = area.x;
            state.last_cell.0 = row;
            for (col, mut w) in (state.offset.1..ds_shape.1).zip(widths) {
                if !state.expanded_columns.contains(&col) && *w > shrunk_width {
                    w = &shrunk_width;
                }
                let highlighted = match state.selection {
                    Selection::Cell(i, j) => { i == row && j == col }
                    Selection::Row(i) => { i == row }
                    Selection::Col(j) => { j == col }
                    Selection::None => { false }
                };
                let text = self.text(row, col, highlighted);
                state.last_cell.1 = col;
                for (off, spn) in text.lines.iter().enumerate() {
                    let off = off as u16;
                    if y + off >= area.height {
                        state.text_cut.0 = true;
                        break;
                    }
                    if x + w > area.width {
                        buf.set_spans(x, y + off, spn, area.width - x);
                        state.text_cut.1 = true
                    } else {
                        buf.set_spans(x, y + off, spn, *w);
                    }
                }
                lines = max(lines, text.lines.len() as u16);
                x += w + self.styler.col_spacing();
                if x >= area.width && col != ds_shape.1 - 1 {
                    break;
                }
            }
            y += lines + self.styler.row_spacing();
            if y >= area.height && row != ds_shape.0 - 1 {
                break;
            }
        }
    }
}


pub mod dummies {
    use tui::style::{Color, Modifier, Style};
    use super::{DataSource, Styler};

    pub struct DummyDataSource {
        magic: Vec<Vec<String>>,
        widths: Vec<u16>,
    }


    #[derive(Default)]
    pub struct DummyStyle {}

    impl Default for DummyDataSource {
        fn default() -> Self {
            let mut magic = Vec::new();
            for i in 0..40 {
                let mut v = Vec::new();
                for j in 0..10 {
                    v.push(format!("Value ({}, {})\n------({})", i, j, i + j));
                }
                magic.push(v);
            }
            Self {
                magic,
                widths: vec![15].repeat(10),
            }
        }
    }

    impl DataSource for DummyDataSource {
        fn value(&self, row: usize, col: usize) -> &str {
            &self.magic[row][col]
        }

        fn shape(&self) -> (usize, usize) {
            (40, 10)
        }

        fn max_widths(&self) -> &[u16] {
            &self.widths
        }
    }

    impl Styler for DummyStyle {
        fn normal(&self, _row: usize, col: usize) -> Style {
            match col % 6 {
                0 => { Style::default().fg(Color::Red) }
                1 => { Style::default().fg(Color::Yellow) }
                2 => { Style::default().fg(Color::Green) }
                3 => { Style::default().fg(Color::Cyan) }
                4 => { Style::default().fg(Color::Blue) }
                5 => { Style::default().fg(Color::Magenta) }
                _ => { panic!("Impossible number") }
            }
        }

        fn highlight(&self, row: usize, col: usize) -> Style {
            self.normal(row, col).add_modifier(Modifier::REVERSED)
        }

        fn row_spacing(&self) -> u16 { 1 }

        fn col_spacing(&self) -> u16 { 1 }
    }
}