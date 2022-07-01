use std::{
	fs::File,
	io::Read,
	path::{PathBuf},
	sync::{Arc, atomic::AtomicBool},
};
use std::ops::DerefMut;

use tokio::{sync::{
	mpsc::{self, Receiver, Sender},
	Mutex,
}, task::JoinHandle};

pub enum ProcessEvent {
	File(PathBuf),
}

#[derive(Debug)]
pub enum ProcessResult {
	File(Vec<u8>),
}

pub struct Processor {
	cancelled: Arc<AtomicBool>,
}

impl Processor {
	fn new() -> Processor {
		Processor { cancelled: Arc::new(AtomicBool::new(false)) }
	}

	pub fn cancel(&mut self) {
		self.cancelled.store(true, std::sync::atomic::Ordering::Relaxed);
	}
}

pub fn spawn<F>(event: ProcessEvent, callback: F) -> Processor
where
	F: Fn(anyhow::Result<ProcessResult>) + Send + Sync + 'static
{
	let processor = Processor::new();
	let cancellled = processor.cancelled.clone();
	// let cancelled = self.cancelled.clone();
	tokio::spawn(async move {
		let result = process(event).await;
		callback(result);
	});
	processor
}

fn process_file(file: PathBuf) -> anyhow::Result<ProcessResult> {
	std::thread::sleep(std::time::Duration::from_secs(2));
	let mut file = std::fs::File::open(file)?;
	let mut buf = Vec::new();
	file.read_to_end(&mut buf)?;
	Ok(ProcessResult::File(buf))
}

async fn process(event: ProcessEvent) -> anyhow::Result<ProcessResult> {
	match event {
		ProcessEvent::File(file) => {
			process_file(file)
		}
	}
}
