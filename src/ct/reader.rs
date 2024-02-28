use tokio::sync::mpsc::{channel, Receiver, Sender};

use crossterm as ct;
use ct::event::Event;

pub struct Reader {
    tx: Sender<Event>,
    rx: Option<Receiver<Event>>,
}

impl Default for Reader {
    fn default() -> Self {
        Self::new()
    }
}

impl Reader {
    pub fn new() -> Self {
        let (tx, rx) = channel::<Event>(10);
        Self { tx, rx: Some(rx) }
    }

    pub fn init(&mut self) -> Receiver<Event> {
        let tx = self.tx.clone();
        tokio::spawn(async move {
            loop {
                if let Ok(e) = ct::event::read() {
                    let _ = tx.send(e).await;
                }
            }
        });

        self.rx.take().unwrap()
    }
}
