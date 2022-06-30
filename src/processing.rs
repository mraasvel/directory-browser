use std::{
	fs::File,
	io::Read,
	path::{PathBuf},
	sync::Arc,
};

use tokio::sync::{
	mpsc::{self, Receiver, Sender},
	Mutex,
};

use crate::app::App;

pub enum ProcessEvent {
	File(PathBuf),
}

#[derive(Debug)]
pub enum ProcessResult {
	File(Vec<u8>),
}

pub struct Processor {
	receiver: Receiver<ProcessEvent>,
	context: Arc<Mutex<App>>,
}

pub fn channel() -> (Sender<ProcessEvent>, Receiver<ProcessEvent>) {
	mpsc::channel(100)
}

impl Processor {
	pub fn new(context: Arc<Mutex<App>>, receiver: Receiver<ProcessEvent>) -> Processor {
		Processor { receiver, context }
	}

	fn process_file(&mut self, path: PathBuf) -> anyhow::Result<ProcessResult> {
		let mut file = File::open(path)?;
		std::thread::sleep(std::time::Duration::from_secs(2));
		let mut buf = Vec::new();
		file.read_to_end(&mut buf)?;
		Ok(ProcessResult::File(buf))
	}

	pub async fn run(&mut self) {
		while let Some(event) = self.receiver.recv().await {
			match event {
				ProcessEvent::File(path) => {
					let result = self.process_file(path);
					self.context.lock().await.wake(result);
				}
			}
		}
	}
}
