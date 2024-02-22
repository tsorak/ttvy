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

    pub fn init(&self) -> JoinHandle<()> {
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

type Cmd<'a> = &'a str;
type Args<'a> = &'a str;

pub(crate) fn parse_command(s: &str) -> Option<(Cmd, Args)> {
    let mut words = s.split(' ').collect::<Vec<&str>>();

    match words {
        ref mut w if w.len() == 2 => {
            let cmd = w.remove(0);
            Some((cmd, words[0]))
        }
        _ => None,
    }
}
