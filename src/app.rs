use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tui::layout::Rect;

use crate::{menu::Menu, FrameType};

#[derive(Debug, Eq, PartialEq)]
enum State {
	Menu,
	Component,
}

pub struct App {
	pub quit: bool,
	menu: Menu,
	state: State,
}

impl App {
	pub fn new() -> App {
		App {
			quit: false,
			menu: Menu::new(),
			state: State::Menu,
		}
	}

	pub fn keyhook(&mut self, event: KeyEvent) {
		match self.state {
			State::Menu => self.menu.keyhook(event),
			State::Component => {}
		}
		match event.code {
			KeyCode::Enter if self.state == State::Menu => {
				let index = self.menu.flush();
				self.quit = true;
			}
			KeyCode::Char('c') if event.modifiers == KeyModifiers::CONTROL => self.quit = true,
			_ => {}
		}
	}

	pub fn render(&mut self, frame: &mut FrameType, area: Rect) {
		match self.state {
			State::Menu => self.menu.render(frame, area),
			State::Component => {}
		}
	}
}
