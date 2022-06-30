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
	fn render(&mut self, frame: &mut FrameType, area: Rect);
	fn keyhook(&mut self, _: KeyEvent) {}
	fn mounted(&mut self) -> anyhow::Result<()> {
		Ok(())
	}
	fn unmounted(&mut self) {}
	fn wake(&mut self, _: anyhow::Result<ProcessResult>) {}
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
		if event::poll(timeout)? {
			let event = event::read()?;
			handle_event(event, &app);
			flush_events(&app)?;
		}
	}
	Ok(())
}

fn init_components(sender: Arc<Sender<ProcessEvent>>) -> Vec<Box<dyn Component + Send>> {
	vec![Box::new(Browser::new(sender.clone()))]
}

pub async fn run() -> anyhow::Result<()> {
	env_logger::from_env(env_logger::Env::default().default_filter_or("info"))
		.target(env_logger::Target::Stderr)
		.format_timestamp_nanos()
		.init();
	let term = Terminal::new()?;
	let (sender, receiver) = processing::channel();
	let components = init_components(Arc::new(sender));
	let app = Arc::new(Mutex::new(App::new(components)));
	let mut processor = Processor::new(app.clone(), receiver);
	tokio::spawn(async move { processor.run().await });
	run_ui(term, app).await?;
	Ok(())
}
