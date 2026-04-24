use std::time::{Duration, Instant};

use tokio::sync::mpsc::{channel, Receiver, Sender};

const ANTI_SPAM_COOLDOWN: Duration = Duration::from_millis(1000);

pub fn channel_pair(buffer_size: usize) -> (Sender<String>, Receiver<String>) {
    let (tx, mut bottleneck_rx) = channel::<String>(buffer_size);
    let (bottleneck_tx, rx) = channel::<String>(buffer_size);

    tokio::spawn(async move {
        let mut ready_at = Instant::now();

        while let Some(msg) = bottleneck_rx.recv().await {
            let msg = msg.trim_matches(' ').to_string();
            if msg.is_empty() || Instant::now() < ready_at {
                continue;
            }
            let _ = bottleneck_tx.send(msg).await;
            ready_at = Instant::now() + ANTI_SPAM_COOLDOWN;
        }
    });

    (tx, rx)
}
