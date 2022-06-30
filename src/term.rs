use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use std::io::Stdout;
use tui::backend::CrosstermBackend;

pub type Backend = CrosstermBackend<Stdout>;

pub struct Terminal(pub tui::Terminal<Backend>);

// wrapper to allow cleanup in destructor
impl Terminal {
	pub fn new() -> anyhow::Result<Terminal> {
		let mut stdout = std::io::stdout();
		crossterm::execute!(stdout, EnterAlternateScreen)?;
		let backend = CrosstermBackend::new(stdout);
		let term = tui::Terminal::new(backend)?;
		crossterm::terminal::enable_raw_mode()?;
		Ok(Terminal(term))
	}
}

fn check<T, E: std::error::Error>(result: Result<T, E>) {
	if let Err(err) = result {
		log::error!("{}", err);
	}
}

impl Drop for Terminal {
	fn drop(&mut self) {
		check(crossterm::terminal::disable_raw_mode());
		check(crossterm::execute!(self.0.backend_mut(), LeaveAlternateScreen));
		check(self.0.show_cursor());
	}
}
