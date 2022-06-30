use crossterm::event::{KeyCode, KeyEvent};
use tui::{
	layout::{Constraint, Direction, Layout, Rect},
	style::{Color, Modifier, Style},
	text::{Span, Spans},
	widgets::{Block, Borders, List, ListItem, ListState},
};

use crate::FrameType;

pub struct Menu {
	state: ListState,
	options: Vec<&'static str>,
	index: usize,
}

fn init_list(options: &[&'static str]) -> List<'static> {
	let items: Vec<ListItem> = options
		.iter()
		.map(|option| {
			let span = Spans::from(Span::styled(
				*option,
				Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
			));
			ListItem::new(span)
		})
		.collect();
	List::new(items)
		.highlight_style(Style::default().bg(Color::Blue))
		.block(Block::default().title("Menu"))
}

impl Menu {
	pub fn new() -> Menu {
		let options = vec!["A", "B", "Quit"];
		let mut state = ListState::default();
		state.select(Some(0));
		Menu {
			options,
			state,
			index: 0,
		}
	}

	pub fn flush(&mut self) -> usize {
		let index = self.index;
		self.index = 0;
		index
	}

	pub fn keyhook(&mut self, event: KeyEvent) {
		match event.code {
			KeyCode::Up => self.prev(),
			KeyCode::Down => self.next(),
			_ => {}
		}
	}

	fn next(&mut self) {
		if self.index < self.options.len() - 1 {
			self.index += 1;
			self.state.select(Some(self.index));
		}
	}

	fn prev(&mut self) {
		if self.index > 0 {
			self.index -= 1;
			self.state.select(Some(self.index));
		}
	}

	pub fn render(&mut self, frame: &mut FrameType, area: Rect) {
		let chunks = Layout::default()
			.direction(Direction::Horizontal)
			.constraints([Constraint::Length(40), Constraint::Min(0)])
			.split(area);
		let list = init_list(self.options.as_slice());
		frame.render_stateful_widget(list, chunks[0], &mut self.state);
	}
}
