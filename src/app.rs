use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tui::layout::Rect;

use crate::{menu::Menu, processing::ProcessResult, Component, FrameType};

#[derive(Debug, Eq, PartialEq)]
enum State {
	Menu,
	Component,
}

pub struct App {
	pub quit: bool,
	menu: Menu,
	components: Vec<Box<dyn Component + Send>>,
	active: usize,
	state: State,
}

impl App {
	pub fn new(components: Vec<Box<dyn Component + Send>>) -> App {
		let mut titles: Vec<String> = components.iter().map(|c| c.title()).collect();
		titles.push("Quit".into());
		App {
			quit: false,
			menu: Menu::new(titles),
			components,
			active: 0,
			state: State::Menu,
		}
	}

	pub fn wake(&mut self, result: anyhow::Result<ProcessResult>) {
		log::info!("finished processing with: {:?}", result);
		self.component().wake(result);
	}

	fn select_tab(&mut self) {
		let index = self.menu.flush();
		if index == self.components.len() {
			self.quit = true;
		} else {
			self.active = index;
			self.mount_component();
		}
	}

	fn mount_component(&mut self) {
		match self.component().mounted() {
			Err(err) => {
				log::error!("{}", err);
				// maybe error state
			}
			Ok(_) => self.state = State::Component,
		}
	}

	fn component(&mut self) -> &mut Box<dyn Component + Send> {
		&mut self.components[self.active]
	}

	pub fn keyhook(&mut self, event: KeyEvent) {
		match self.state {
			State::Menu => self.menu.keyhook(event),
			State::Component => self.component().keyhook(event),
		}
		match event.code {
			KeyCode::Enter if self.state == State::Menu => self.select_tab(),
			KeyCode::Esc if self.state != State::Menu => self.state = State::Menu,
			KeyCode::Char('c') if event.modifiers == KeyModifiers::CONTROL => self.quit = true,
			_ => {}
		}
	}

	pub fn render(&mut self, frame: &mut FrameType, area: Rect) {
		match self.state {
			State::Menu => self.menu.render(frame, area),
			State::Component => self.component().render(frame, area),
		}
	}
}
