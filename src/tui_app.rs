use std::io::Stdout;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers, ModifierKeyCode};
use tui::backend::CrosstermBackend;
use tui::Frame;
use tui::widgets::{Block, Borders};

use crate::model_table::{DataSource, Styler, ModelTable, State, Selection};

#[derive(Default)]
pub struct App<DataSource, Styler> {
    is_open: bool,
    state: State,
    data_source: DataSource,
    styler: Styler,
}


impl<T: DataSource, U: Styler> App<T, U> {
    pub fn new(data_source: T, styler: U) -> Self {
        App {
            is_open: true,
            state: State::default(),
            data_source,
            styler,
        }
    }
    pub fn event(&mut self, event: Event) {
        match self.state.selection {
            Selection::None => {
                match event {
                    Event::Key(KeyEvent { code: KeyCode::Char('q'), .. }) => { self.close() }
                    Event::Key(KeyEvent { code: KeyCode::Char('k'), .. }) => { self.up() }
                    Event::Key(KeyEvent { code: KeyCode::Char('j'), .. }) => { self.down() }
                    Event::Key(KeyEvent { code: KeyCode::Char('h'), .. }) => { self.left() }
                    Event::Key(KeyEvent { code: KeyCode::Char('l'), .. }) => { self.right() }
                    Event::Key(KeyEvent { code: KeyCode::Char('v'), .. }) => { self.switch_to_cell_select() }
                    Event::Key(KeyEvent { code: KeyCode::Char('e'), .. }) => { self.toggle_expansion() }
                    Event::Key(KeyEvent { code: KeyCode::Char('u'), modifiers: KeyModifiers::CONTROL, .. }) => { self.page_up() }
                    Event::Key(KeyEvent { code: KeyCode::Char('d'), modifiers: KeyModifiers::CONTROL, .. }) => { self.page_down() }
                    _ => {}
                }
            }
            Selection::Cell(_, _) => {
                match event {
                    Event::Key(KeyEvent { code: KeyCode::Char('q'), .. }) => { self.close() }
                    Event::Key(KeyEvent { code: KeyCode::Char('v'), .. }) => { self.switch_to_no_select() }
                    Event::Key(KeyEvent { code: KeyCode::Char('k'), .. }) => { self.up() }
                    Event::Key(KeyEvent { code: KeyCode::Char('j'), .. }) => { self.down() }
                    Event::Key(KeyEvent { code: KeyCode::Char('h'), .. }) => { self.left() }
                    Event::Key(KeyEvent { code: KeyCode::Char('l'), .. }) => { self.right() }
                    Event::Key(KeyEvent { code: KeyCode::Char('e'), .. }) => { self.toggle_expansion() }
                    _ => {}
                }
            }
            // AppMode::RowSelect(_) => {}
            // AppMode::ColSelect(_) => {}
            Selection::Row(_) => {}
            Selection::Col(_) => {}
        }
    }

    pub fn render(&mut self, frame: &mut Frame<CrosstermBackend<Stdout>>) {
        let block = Block::default()
            .borders(Borders::all())
            .title("Table");
        let tbl = ModelTable::new(&self.data_source, &self.styler);
        frame.render_stateful_widget(tbl, block.inner(frame.size()), &mut self.state);
        frame.render_widget(block, frame.size());
    }

    pub fn is_open(&self) -> bool {
        self.is_open
    }

    pub fn close(&mut self) {
        self.is_open = false;
    }
    fn up(&mut self) {
        match self.state.selection {
            Selection::Cell(mut i, j) => {
                if i > self.state.offset.0 {
                    i -= 1;
                    self.state.selection = Selection::Cell(i, j);
                }
            }
            Selection::Row(i) => {}
            Selection::Col(j) => {}
            Selection::None => {
                if self.state.offset.0 > 0 {
                    self.state.offset.0 -= 1
                }
            }
        }
    }

    fn down(&mut self) {
        match self.state.selection {
            Selection::Cell(mut i, j) => {
                if i < self.state.last_cell.0 {
                    i += 1;
                    self.state.selection = Selection::Cell(i, j);
                    if i > self.state.last_cell.0 {
                        self.state.offset.0 = i - (self.state.last_cell.0 - self.state.offset.0);
                    }
                }
            }
            Selection::Row(_) => {}
            Selection::Col(_) => {}
            Selection::None => {
                if self.state.text_cut.0 || self.state.last_cell.0 < self.data_source.shape().0 - 1 {
                    self.state.offset.0 += 1
                }
            }
        }
    }

    fn left(&mut self) {
        match self.state.selection {
            Selection::Cell(i, mut j) => {
                if j > self.state.offset.1 {
                    j -= 1;
                    self.state.selection = Selection::Cell(i, j);
                }
            }
            Selection::Row(_) => {}
            Selection::Col(_) => {}
            Selection::None => {
                if self.state.offset.1 > 0 {
                    self.state.offset.1 -= 1
                }
            }
        }
    }
    fn right(&mut self) {
        match self.state.selection {
            Selection::Cell(i, mut j) => {
                if j < self.state.last_cell.1 {
                    j += 1;
                    self.state.selection = Selection::Cell(i, j);
                }
            }
            Selection::Row(_) => {}
            Selection::Col(_) => {}
            Selection::None => {
                if self.state.text_cut.1 || self.state.last_cell.1 < self.data_source.shape().1 - 1 {
                    self.state.offset.1 += 1
                }
            }
        }
    }

    fn page_up(&mut self) {
        (0..(self.state.last_cell.0 - self.state.offset.0)).for_each(|_| { self.up(); });
    }

    fn page_down(&mut self) {
        (0..(self.state.last_cell.0 - self.state.offset.0)).for_each(|_| { self.down(); });
    }

    fn switch_to_no_select(&mut self) {
        self.state.selection = Selection::None;
    }

    fn switch_to_cell_select(&mut self) {
        self.state.selection = Selection::Cell(self.state.offset.0, self.state.offset.1);
    }
    fn toggle_expansion(&mut self) {
        match self.state.selection {
            Selection::Cell(_, j) => {
                if self.state.expanded_columns.contains(&j) {
                    self.state.expanded_columns.remove(&j);
                } else {
                    self.state.expanded_columns.insert(j);
                }
            }
            Selection::Row(_)=> {}
            Selection::Col(j) => {}
            Selection::None => {
                if (self.state.offset.1..self.state.last_cell.1).all(|v| self.state.expanded_columns.contains(&v)) {
                    // (self.state.offset.1..self.state.last_cell.1).for_each(|v| { self.state.expanded_columns.remove(&v); })
                    self.state.expanded_columns.clear();
                } else {
                    (0..self.data_source.shape().1).for_each(|v| { self.state.expanded_columns.insert(v); })
                }
            }
        };
    }
}