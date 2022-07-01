use std::path::PathBuf;
use std::sync::Arc;

use crossterm::event::KeyCode;
use futures::executor;
use tokio::sync::Mutex;
use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;
use tui::style::{Color, Style};
use tui::text::{Span, Spans};
use tui::widgets::{List, ListItem, ListState};
use std::ops::{Deref};

use crate::processing::{ProcessEvent, ProcessResult, Processor};
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

#[derive(Default)]
struct State {
	processor: Option<Processor>
}

impl State {
	fn new() -> State {
		State { ..Default::default() }
	}
}

pub struct Browser {
	files: Vec<DirEntry>,
	liststate: ListState,
	state: Arc<Mutex<State>>,
}

impl Browser {
	pub fn new() -> Browser {
		let liststate = ListState::default();
		Browser {
			files: Vec::new(),
			liststate,
			state: Arc::new(Mutex::new(State::new())),
		}
	}

	fn next(&mut self) {
		let index = self.liststate.selected().unwrap();
		if index == self.files.len() - 1 {
			return;
		}
		self.liststate.select(Some(index + 1));
	}

	fn prev(&mut self) {
		let index = self.liststate.selected().unwrap();
		if index == 0 {
			return;
		}
		self.liststate.select(Some(index - 1));
	}

	async fn process_file(&mut self, path: PathBuf) {
		if self.state.lock().await.processor.is_some() {
			log::warn!("already processing file");
			return;
		}
		log::info!("process file: {:?}", path);
		let state = self.state.clone();
		self.state.lock().await.processor = Some(processing::spawn(ProcessEvent::File(path), move |result| {
			log::info!("finished: {:?}", result);
			executor::block_on(state.lock()).processor = None;
		}));
	}

	fn select(&mut self) {
		let index = self.liststate.selected().unwrap();
		match self.files[index].file_type {
			FileType::Directory => {
				if let Err(err) = self.read_directory(self.files[index].path.clone()) {
					log::error!("{}", err);
				}
			}
			FileType::File => {
				let pathbuf = self.files[index].path.as_path().to_path_buf();
				executor::block_on(self.process_file(pathbuf));
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
		self.liststate.select(Some(0));
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
		frame.render_stateful_widget(list, area, &mut self.liststate);
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
