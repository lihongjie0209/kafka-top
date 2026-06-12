use crossterm::event::{Event as CrosstermEvent, KeyEvent};
use tokio::sync::mpsc;
use tokio::time::{interval, Duration};

#[derive(Debug, Clone)]
pub enum Event {
    Key(KeyEvent),
    Tick,
}

pub struct EventStream {
    rx: mpsc::Receiver<Event>,
    _handle: tokio::task::JoinHandle<()>,
}

impl EventStream {
    pub fn new(refresh_interval_secs: u64) -> Self {
        let (tx, rx) = mpsc::channel(256);
        let tick_duration = Duration::from_secs(refresh_interval_secs);

        let handle = tokio::spawn(async move {
            let mut tick = interval(tick_duration);
            tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                tokio::select! {
                    result = tokio::task::spawn_blocking(crossterm::event::read) => {
                        match result {
                            Ok(Ok(crossterm_event)) => {
                                match crossterm_event {
                                    CrosstermEvent::Key(key) => {
                                        if tx.send(Event::Key(key)).await.is_err() { break; }
                                    }
                                    CrosstermEvent::Resize(_w, _h) => {
                                        // resize is handled by ratatui automatically
                                    }
                                    _ => {}
                                }
                            }
                            Ok(Err(_)) | Err(_) => {
                                // crossterm read error — continue reading
                                continue;
                            }
                        }
                    }
                    _ = tick.tick() => {
                        if tx.send(Event::Tick).await.is_err() { break; }
                    }
                }
            }
        });

        Self { rx, _handle: handle }
    }

    pub async fn next(&mut self) -> Option<Event> {
        self.rx.recv().await
    }
}
