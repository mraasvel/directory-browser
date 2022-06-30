use std::path::PathBuf;
use std::sync::Arc;

use crossterm::event::KeyCode;
use futures::executor;
use tokio::sync::mpsc::Sender;
use tui::style::{Color, Style};
use tui::text::{Span, Spans};
use tui::widgets::{List, ListItem, ListState};

use crate::processing::ProcessEvent;
use crate::{processing, Component};

#[derive(Debug, Copy, Clone)]
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
	sender: Arc<Sender<ProcessEvent>>,
}

impl Browser {
	pub fn new(sender: Arc<Sender<ProcessEvent>>) -> Browser {
		let state = ListState::default();
		Browser {
			files: Vec::new(),
			state,
			sender,
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
			}
			FileType::File => {
				let pathbuf = self.files[index].path.as_path().to_path_buf();
				let future = self.sender.send(ProcessEvent::File(pathbuf));
				if let Err(e) = executor::block_on(future) {
					log::error!("{}", e);
				}
			}
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

fn make_style(file_type: FileType) -> Style {
	match file_type {
		FileType::Directory => Style::default().fg(Color::LightBlue),
		FileType::File => Style::default(),
		FileType::Symlink => Style::default().fg(Color::LightMagenta),
		FileType::Other => Style::default().fg(Color::LightRed),
	}
}

impl Component for Browser {
	fn title(&self) -> String {
		"Directory Browser".to_string()
	}

	fn wake(&mut self, _: anyhow::Result<processing::ProcessResult>) {
		log::info!("browser wake called");
	}

	fn render(&mut self, frame: &mut crate::FrameType, area: tui::layout::Rect) {
		let names: Vec<ListItem> = self
			.files
			.iter()
			.map(|file| {
				let style = make_style(file.file_type);
				let span = Spans::from(Span::styled(file.name.as_str(), style));
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
		match event.code {
			KeyCode::Up => self.prev(),
			KeyCode::Down => self.next(),
			KeyCode::Enter => self.select(),
			_ => {}
		}
	}
}
