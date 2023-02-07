use crossterm::{
    event::{self},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{env, io};
use tui::{backend::CrosstermBackend, Terminal};
use tui_tabview::data_sources::CSV;
use tui_tabview::model_table::dummies::{DummyDataSource, DummyStyle};
use tui_tabview::tui_app::App;

fn main() -> Result<(), io::Error> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let command_line_args : Vec<String> = env::args().collect();

    // load data source & styler
    let csv_data_source = CSV::new(&command_line_args[1]);
    let data_source = DummyDataSource::default();
    let styler = DummyStyle::default();

    //  main loop
    let mut app = App::new(csv_data_source, styler);
    while app.is_open() {
        terminal.draw(|f| {
            app.render(f);
        })?;

        match event::read() {
            Ok(e) => app.event(e),
            Err(err) => {
                println!("Error reading event: {}", err)
            }
        }
    }

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        // DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
