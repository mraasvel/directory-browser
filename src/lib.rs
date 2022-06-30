use app::App;
use browser::Browser;
use crossterm::event::{self, Event, KeyEvent};
use std::rc::Rc;
use std::{cell::RefCell, io::Stdout};
use term::Terminal;
use tui::layout::Rect;
use tui::{backend::CrosstermBackend, Frame};

mod app;
mod browser;
mod menu;
mod term;

type BackendType = CrosstermBackend<Stdout>;
type FrameType<'a> = Frame<'a, BackendType>;

pub trait Component {
	fn title(&self) -> String;
	fn render(&mut self, frame: &mut FrameType, area: Rect);
	fn keyhook(&mut self, _: KeyEvent) {}
	fn mounted(&mut self) -> anyhow::Result<()> {
		Ok(())
	}
	fn unmounted(&mut self) {}
}

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

fn init_components() -> Vec<Box<dyn Component>> {
	vec![Box::new(Browser::new())]
}

pub fn run() -> anyhow::Result<()> {
	env_logger::from_env(env_logger::Env::default().default_filter_or("info"))
		.target(env_logger::Target::Stderr)
		.format_timestamp_nanos()
		.init();
	let term = Terminal::new()?;
	let components = init_components();
	let app = Rc::new(RefCell::new(App::new(components)));
	run_ui(term, app)?;
	Ok(())
}
