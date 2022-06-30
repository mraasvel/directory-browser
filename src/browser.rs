use std::path::PathBuf;

use crossterm::event::KeyCode;
use tui::style::{Color, Style};
use tui::text::{Span, Spans};
use tui::widgets::{List, ListItem, ListState};

use crate::Component;

enum FileType {
	File,
	Directory,
	Symlink,
	Other,
}

struct DirEntry {
	path: PathBuf,
	name: String,
	file_type: FileType,
}

fn special_directories(path: PathBuf) -> anyhow::Result<Vec<DirEntry>> {
	let mut files = Vec::new();
	let parent = path.parent().map(|parent| DirEntry {
		path: parent.to_path_buf(),
		name: "..".into(),
		file_type: FileType::Directory,
	});
	let cwd = DirEntry {
		path,
		name: ".".into(),
		file_type: FileType::Directory,
	};
	files.push(cwd);
	if let Some(parent) = parent {
		files.push(parent);
	}
	Ok(files)
}

pub struct Browser {
	files: Vec<DirEntry>,
	state: ListState,
}

impl Browser {
	pub fn new() -> Browser {
		let state = ListState::default();
		Browser {
			files: Vec::new(),
			state,
		}
	}

	fn next(&mut self) {
		let index = self.state.selected().unwrap();
		if index == self.files.len() - 1 {
			return;
		}
		self.state.select(Some(index + 1));
	}

	fn prev(&mut self) {
		let index = self.state.selected().unwrap();
		if index == 0 {
			return;
		}
		self.state.select(Some(index - 1));
	}

	fn select(&mut self) {
		let index = self.state.selected().unwrap();
		match self.files[index].file_type {
			FileType::Directory => {
				if let Err(err) = self.read_directory(self.files[index].path.clone()) {
					log::error!("{}", err);
				}
			},
			_ => {}
		}
	}

	fn read_directory(&mut self, path: PathBuf) -> anyhow::Result<()> {
		let rd = std::fs::read_dir(&path)?;
		let mut files = special_directories(path)?;
		files.append(
			&mut rd
				.map(|file| {
					let file = file?;
					let file_type = file.file_type()?;
					let file_type = if file_type.is_dir() {
						FileType::Directory
					} else if file_type.is_file() {
						FileType::File
					} else if file_type.is_symlink() {
						FileType::Symlink
					} else {
						FileType::Other
					};
					let name = file
						.file_name()
						.into_string()
						.map_err(|_| anyhow::anyhow!("bad unicode filename"))?;
					Ok(DirEntry {
						name,
						file_type,
						path: file.path(),
					})
				})
				.collect::<anyhow::Result<_>>()?,
		);
		assert!(files.len() != 0);
		self.state.select(Some(0));
		self.files = files;
		Ok(())
	}
}

impl Component for Browser {
	fn title(&self) -> String {
		"directory listing".to_string()
	}

	fn render(&mut self, frame: &mut crate::FrameType, area: tui::layout::Rect) {
		let names: Vec<ListItem> = self
			.files
			.iter()
			.map(|file| {
				let span = Spans::from(Span::styled(file.name.as_str(), Style::default()));
				ListItem::new(span)
			})
			.collect();
		let list = List::new(names).highlight_style(Style::default().bg(Color::Blue));
		frame.render_stateful_widget(list, area, &mut self.state);
	}

	fn mounted(&mut self) -> anyhow::Result<()> {
		if self.files.len() > 0 {
			return Ok(());
		}
		self.read_directory(std::env::current_dir()?)
	}

	fn keyhook(&mut self, event: crossterm::event::KeyEvent) {
		log::info!("browser keyhook");
		match event.code {
			KeyCode::Up => self.prev(),
			KeyCode::Down => self.next(),
			KeyCode::Enter => self.select(),
			_ => {}
		}
	}
}
