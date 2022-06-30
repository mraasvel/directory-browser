use app::App;
use crossterm::event::{self, Event, KeyEvent};
use std::cell::RefCell;
use std::rc::Rc;
use term::{Backend, Terminal};
use tui::Frame;

mod app;
mod menu;
mod term;

type FrameType<'a> = Frame<'a, Backend>;

fn handle_keypress(key: KeyEvent, app: &Rc<RefCell<App>>) {
	app.borrow_mut().keyhook(key);
}

fn handle_event(event: Event, app: &Rc<RefCell<App>>) {
	match event {
		Event::Key(event) => handle_keypress(event, app),
		_ => {}
	}
}

fn run_ui(mut term: Terminal, app: Rc<RefCell<App>>) -> anyhow::Result<()> {
	let timeout = std::time::Duration::from_millis(1);
	while !app.borrow().quit {
		term.0.draw(|frame| {
			let area = frame.size();
			app.borrow_mut().render(frame, area)
		})?;
		let event = event::read()?;
		handle_event(event, &app);
		while event::poll(timeout)? {
			let event = event::read()?;
			handle_event(event, &app);
		}
	}
	Ok(())
}

pub fn run() -> anyhow::Result<()> {
	env_logger::from_env(env_logger::Env::default().default_filter_or("info"))
		.target(env_logger::Target::Stderr)
		.format_timestamp_nanos()
		.init();
	let term = Terminal::new()?;
	let app = Rc::new(RefCell::new(App::new()));
	run_ui(term, app)?;
	Ok(())
}
