use tokio::{
    io::{stdin, AsyncBufReadExt, BufReader},
    sync::mpsc::{channel, Receiver, Sender},
    task::JoinHandle,
};

pub struct Channel(pub Sender<String>, pub Receiver<String>);

impl Channel {
    pub fn new(bfr: usize) -> Self {
        let (tx, rx) = channel::<String>(bfr);
        Self(tx, rx)
    }

    pub async fn recieve(&mut self) -> Option<String> {
        self.1.recv().await.map(|s| s.trim().to_owned())
    }

    pub fn init(&self) -> (JoinHandle<()>, Sender<()>) {
        let tx = self.0.clone();

        let (shutdown_tx, mut shutdown_rx) = channel::<()>(1);

        let handle = tokio::spawn(async move {
            let stdin = stdin();
            let mut stdin = BufReader::new(stdin).lines();
            loop {
                tokio::select! {
                    _ = shutdown_rx.recv() => {
                        break;
                    }
                    Ok(Some(line)) = stdin.next_line() => {
                        let _ = tx.send(line).await;
                    }
                }
            }
        });

        (handle, shutdown_tx)
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
