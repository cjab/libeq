use std::sync::mpsc::{self};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::thread;
use std::time::Duration;

use crossterm::event::{self, KeyCode, KeyEventKind};

pub enum Event<I> {
    Input(I),
    Tick,
}

/// A small event handler that wrap termion input and tick events. Each event
/// type is handled in its own thread and returned to a common `Receiver`
pub struct Events {
    rx: mpsc::Receiver<Event<event::Event>>,
    input_handle: thread::JoinHandle<()>,
    ignore_exit_key: Arc<AtomicBool>,
    tick_handle: thread::JoinHandle<()>,
}

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub exit_key: KeyCode,
    pub tick_rate: Duration,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            exit_key: KeyCode::Char('q'),
            tick_rate: Duration::from_millis(500),
        }
    }
}

impl Events {
    pub fn new() -> Events {
        Events::with_config(Config::default())
    }

    pub fn with_config(config: Config) -> Events {
        let (tx, rx) = mpsc::channel();
        let ignore_exit_key = Arc::new(AtomicBool::new(false));
        let input_handle = {
            let tx = tx.clone();
            let ignore_exit_key = ignore_exit_key.clone();

            thread::spawn(move || {
                loop {
                    if event::poll(config.tick_rate).is_ok()
                        && let Ok(evt) = event::read()
                    {
                        match evt {
                            event::Event::Key(key_event) => {
                                if key_event.kind != KeyEventKind::Release
                                    && tx.send(Event::Input(evt)).is_err()
                                {
                                    return;
                                }

                                if !ignore_exit_key.load(Ordering::Relaxed)
                                    && key_event.code == config.exit_key
                                {
                                    return;
                                }
                            }
                            event::Event::Mouse(_) => {
                                if tx.send(Event::Input(evt)).is_err() {
                                    return;
                                }
                            }
                            _ => {}
                        }
                    }
                }
            })
        };
        let tick_handle = {
            thread::spawn(move || {
                loop {
                    if tx.send(Event::Tick).is_err() {
                        break;
                    }
                    thread::sleep(config.tick_rate);
                }
            })
        };
        Events {
            rx,
            ignore_exit_key,
            input_handle,
            tick_handle,
        }
    }

    pub fn next(&self) -> Result<Event<event::Event>, mpsc::RecvError> {
        self.rx.recv()
    }

    pub fn disable_exit_key(&mut self) {
        self.ignore_exit_key.store(true, Ordering::Relaxed);
    }

    pub fn enable_exit_key(&mut self) {
        self.ignore_exit_key.store(false, Ordering::Relaxed);
    }
}
