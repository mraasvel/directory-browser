use app::App;
use browser::Browser;
use crossterm::event::{self, Event, KeyEvent};
use futures::executor;
use processing::{ProcessEvent, ProcessResult, Processor};
use std::io::Stdout;
use std::sync::Arc;
use term::Terminal;
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;
use tui::layout::Rect;
use tui::{backend::CrosstermBackend, Frame};

mod app;
mod browser;
mod menu;

mod processing;
mod term;

type BackendType = CrosstermBackend<Stdout>;
type FrameType<'a> = Frame<'a, BackendType>;

pub trait Component {
	fn title(&self) -> String;
	fn before_render(&mut self) {}
	fn render(&mut self, frame: &mut FrameType, area: Rect);
	fn keyhook(&mut self, _: KeyEvent) {}
	fn mounted(&mut self) -> anyhow::Result<()> {
		Ok(())
	}
	fn unmounted(&mut self) {}
}

fn handle_keypress(key: KeyEvent, app: &Arc<Mutex<App>>) {
	let mut app = executor::block_on(app.lock());
	app.keyhook(key);
}

fn handle_event(event: Event, app: &Arc<Mutex<App>>) {
	match event {
		Event::Key(event) => handle_keypress(event, app),
		_ => {}
	}
}

fn flush_events(app: &Arc<Mutex<App>>) -> anyhow::Result<()> {
	// handle any events in queue
	let timeout = std::time::Duration::from_millis(1);
	while event::poll(timeout)? {
		let event = event::read()?;
		handle_event(event, app);
	}
	Ok(())
}

async fn run_ui(mut term: Terminal, app: Arc<Mutex<App>>) -> anyhow::Result<()> {
	// tick duration
	let timeout = std::time::Duration::from_secs(1);
	loop {
		{
			let mut app = app.lock().await;
			if app.quit {
				break;
			}
			term.0.draw(|frame| {
				let area = frame.size();
				app.render(frame, area)
			})?;
		}
		// I don't understand the point behind some apps using a seperate thread and a channel with ticks to send events
		// this is doing the exact same thing, perhaps the other allows you to avoid rerendering on unused events
		if event::poll(timeout)? {
			let event = event::read()?;
			handle_event(event, &app);
			flush_events(&app)?;
		}
	}
	Ok(())
}

fn init_components() -> Vec<Box<dyn Component>> {
	vec![Box::new(Browser::new())]
}

pub async fn run() -> anyhow::Result<()> {
	env_logger::from_env(env_logger::Env::default().default_filter_or("info"))
		.target(env_logger::Target::Stderr)
		.format_timestamp_nanos()
		.init();
	let term = Terminal::new()?;
	let components = init_components();
	let app = Arc::new(Mutex::new(App::new(components)));
	run_ui(term, app).await?;
	Ok(())
}
