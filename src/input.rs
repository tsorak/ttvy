use std::io::stdin;

use tokio::{
    sync::mpsc::{channel, Receiver, Sender},
    task::JoinHandle,
};

pub struct Channel(pub Sender<String>, pub Receiver<String>);

impl Channel {
    pub fn new(bfr: usize) -> Self {
        let (tx, rx) = channel(bfr);
        Self(tx, rx)
    }

    pub async fn recieve(&mut self) -> Option<String> {
        self.1.recv().await.map(|s| s.trim().to_owned())
    }

    pub fn init_stdin_read_loop(&self) -> JoinHandle<()> {
        let tx = self.0.clone();

        tokio::spawn(async move {
            loop {
                let mut buf: String = "".to_owned();
                stdin().read_line(&mut buf).unwrap();

                let _ = tx.send(buf).await;
            }
        })
    }
}
