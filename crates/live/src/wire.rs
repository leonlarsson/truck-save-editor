//! How a snapshot gets from the plugin (inside the game) to the editor: over a connection on this
//! computer only, one line of JSON per snapshot, ten a second.
//!
//! Plain blocking sockets from Rust's own library, one thread on each side: nothing here needs
//! `unsafe`, and both sides are tested against each other without the game.

use std::io::{BufRead as _, BufReader, ErrorKind, Read as _, Write as _};
use std::net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::thread::JoinHandle;
use std::time::Duration;

use crate::Snapshot;

/// The port the plugin listens on, on `127.0.0.1` (this computer only).
pub const PORT: u16 = 47227;

/// How often a snapshot is sent.
const INTERVAL: Duration = Duration::from_millis(100);
/// How long a client that stopped reading is waited for before it's dropped.
const WRITE_TIMEOUT: Duration = Duration::from_millis(250);
/// How long the editor waits for a line before taking the game as gone (the plugin sends ten a
/// second).
const READ_TIMEOUT: Duration = Duration::from_secs(2);
/// How long the editor waits before looking for the game again.
const RETRY: Duration = Duration::from_secs(2);
/// The longest line the editor reads, in bytes: a snapshot is a few thousand. Whoever sends more
/// without ending the line isn't the plugin, and is hung up on.
const LONGEST_LINE: usize = 256 * 1024;
/// How many clients the plugin sends to at once, and how many it takes on in one look: more than
/// the editor ever needs, and few enough that slow or countless ones can't hold the thread up
/// (which would hold the game up when it closes and waits for the thread).
const MOST_CLIENTS: usize = 8;

/// A snapshot as one line, ending in a line break.
pub fn encode(snapshot: &Snapshot) -> String {
    // Serialising plain numbers, strings and lists can't fail; an empty line (which `decode`
    // skips) rather than a panic inside the game, should it ever.
    let mut line = serde_json::to_string(snapshot).unwrap_or_default();
    line.push('\n');
    line
}

/// A line back into a snapshot, with anything that can't be real taken out (see
/// `Snapshot::made_safe`); `None` for anything else.
pub fn decode(line: &str) -> Option<Snapshot> {
    serde_json::from_str::<Snapshot>(line.trim())
        .ok()
        .map(Snapshot::made_safe)
}

/// The plugin's side, running: sends the shared snapshot to every client until stopped.
pub struct Server {
    port: u16,
    should_stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl Server {
    /// The port it listens on.
    pub fn port(&self) -> u16 {
        self.port
    }
}

impl Drop for Server {
    /// Stops the thread and waits for it: the game unloads the plugin right after, and a thread
    /// still running code that's gone would crash the game.
    fn drop(&mut self) {
        self.should_stop.store(true, Ordering::Relaxed);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

/// Starts sending `snapshot` to whoever connects on [`PORT`].
pub fn serve(snapshot: Arc<Mutex<Snapshot>>) -> std::io::Result<Server> {
    serve_on(PORT, snapshot)
}

/// Like [`serve`], on a port of the caller's choosing (0 for any free one: tests).
pub fn serve_on(port: u16, snapshot: Arc<Mutex<Snapshot>>) -> std::io::Result<Server> {
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, port))?;
    // Not blocking, so the thread can look at `should_stop` between clients.
    listener.set_nonblocking(true)?;
    let port = listener.local_addr()?.port();
    let should_stop = Arc::new(AtomicBool::new(false));
    let thread = {
        let should_stop = should_stop.clone();
        std::thread::Builder::new()
            .name("tse-live".to_string())
            .spawn(move || run_server(listener, snapshot, should_stop))?
    };
    Ok(Server {
        port,
        should_stop,
        thread: Some(thread),
    })
}

fn run_server(listener: TcpListener, snapshot: Arc<Mutex<Snapshot>>, should_stop: Arc<AtomicBool>) {
    let mut clients: Vec<TcpStream> = Vec::new();
    while !should_stop.load(Ordering::Relaxed) {
        // Those who connected since the last look: a few at a time, and no more than there's
        // room for (the others are hung up on, by being dropped).
        for _ in 0..MOST_CLIENTS {
            let Ok((client, _)) = listener.accept() else {
                break;
            };
            if clients.len() >= MOST_CLIENTS {
                continue;
            }
            // On Windows a client starts out not blocking, like the listener; writes should wait
            // (a little) instead.
            let is_ready = client.set_nonblocking(false).is_ok()
                && client.set_write_timeout(Some(WRITE_TIMEOUT)).is_ok();
            let _ = client.set_nodelay(true);
            if is_ready {
                clients.push(client);
            }
        }
        if !clients.is_empty() {
            // A copy, so the game's thread never waits on a slow client. A poisoned lock still
            // holds a usable snapshot.
            let now = snapshot
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .clone();
            let line = encode(&now);
            // A client that's gone, or stopped reading, is dropped.
            clients.retain_mut(|client| client.write_all(line.as_bytes()).is_ok());
        }
        std::thread::sleep(INTERVAL);
    }
}

/// The editor's side: connects to the plugin on `port` and calls `on_change` with every
/// snapshot, and with `None` when the game is gone (or was never there). Keeps looking for the
/// game until `should_stop` is set. Runs on its own thread.
pub fn listen(
    port: u16,
    should_stop: Arc<AtomicBool>,
    mut on_change: impl FnMut(Option<Snapshot>) + Send + 'static,
) -> std::io::Result<JoinHandle<()>> {
    std::thread::Builder::new()
        .name("tse-live".to_string())
        .spawn(move || {
            let address = SocketAddr::from((Ipv4Addr::LOCALHOST, port));
            while !should_stop.load(Ordering::Relaxed) {
                if let Ok(stream) = TcpStream::connect_timeout(&address, READ_TIMEOUT) {
                    read_snapshots(stream, &should_stop, &mut on_change);
                    on_change(None);
                }
                // In short steps, so stopping doesn't wait the whole time.
                let mut waited = Duration::ZERO;
                while waited < RETRY && !should_stop.load(Ordering::Relaxed) {
                    std::thread::sleep(INTERVAL);
                    waited += INTERVAL;
                }
            }
        })
}

/// Reads lines until the connection ends, goes quiet, or it's time to stop.
fn read_snapshots(
    stream: TcpStream,
    should_stop: &AtomicBool,
    on_change: &mut impl FnMut(Option<Snapshot>),
) {
    if stream.set_read_timeout(Some(READ_TIMEOUT)).is_err() {
        return;
    }
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    while !should_stop.load(Ordering::Relaxed) {
        line.clear();
        // At most a line's worth and a byte: a line that long without its end isn't one.
        let mut limited = (&mut reader).take(LONGEST_LINE as u64 + 1);
        match limited.read_line(&mut line) {
            // The plugin closed the connection: the game quit.
            Ok(0) => return,
            Ok(_) if line.len() > LONGEST_LINE => return,
            Ok(_) => {
                if let Some(snapshot) = decode(&line) {
                    on_change(Some(snapshot));
                }
            }
            Err(error) if error.kind() == ErrorKind::Interrupted => {}
            Err(_) => return,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;

    #[test]
    fn a_snapshot_survives_the_line() {
        let snapshot = Snapshot {
            game: "eut2".to_string(),
            speed: 22.5,
            truck_brand: "A \"quoted\"\nname".to_string(),
            ..Snapshot::default()
        };
        let line = encode(&snapshot);
        assert_eq!(line.matches('\n').count(), 1);
        assert_eq!(decode(&line), Some(snapshot));
        assert_eq!(decode("not a snapshot"), None);
    }

    #[test]
    fn a_newer_side_s_fields_are_ignored_and_missing_ones_filled_in() {
        let snapshot = decode(r#"{"speed": 10.0, "a_field_from_the_future": [1, 2]}"#);
        assert_eq!(snapshot.map(|snapshot| snapshot.speed), Some(10.0));
    }

    #[test]
    fn an_endless_line_is_hung_up_on() {
        // Someone who isn't the plugin: sends without ever ending the line.
        let Ok(listener) = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)) else {
            return;
        };
        let Ok(port) = listener.local_addr().map(|address| address.port()) else {
            return;
        };
        let sender = std::thread::spawn(move || {
            let Ok((mut client, _)) = listener.accept() else {
                return;
            };
            let chunk = vec![b'x'; 64 * 1024];
            // Until the reader hangs up (a write then fails), or far more than the limit.
            for _ in 0..64 {
                if client.write_all(&chunk).is_err() {
                    break;
                }
            }
        });
        let Ok(stream) = TcpStream::connect((Ipv4Addr::LOCALHOST, port)) else {
            return;
        };
        let mut heard = 0;
        read_snapshots(stream, &AtomicBool::new(false), &mut |_| heard += 1);
        // It came back (rather than reading for ever), having taken nothing for a snapshot.
        assert_eq!(heard, 0);
        let _ = sender.join();
    }

    #[test]
    fn the_editor_s_side_hears_the_plugin_s() {
        let shared = Arc::new(Mutex::new(Snapshot {
            odometer_km: 1234.0,
            ..Snapshot::default()
        }));
        // Port 0: any free one, so tests never collide with a running game.
        let Ok(server) = serve_on(0, shared.clone()) else {
            return;
        };
        let (sender, receiver) = mpsc::channel();
        let should_stop = Arc::new(AtomicBool::new(false));
        let listener = listen(server.port(), should_stop.clone(), move |snapshot| {
            let _ = sender.send(snapshot);
        });

        let first = receiver.recv_timeout(Duration::from_secs(5));
        assert_eq!(
            first.ok().flatten().map(|snapshot| snapshot.odometer_km),
            Some(1234.0)
        );

        // The game quits: the editor hears that too.
        drop(server);
        let mut heard_the_end = false;
        while let Ok(snapshot) = receiver.recv_timeout(Duration::from_secs(5)) {
            if snapshot.is_none() {
                heard_the_end = true;
                break;
            }
        }
        assert!(heard_the_end);

        should_stop.store(true, Ordering::Relaxed);
        if let Ok(listener) = listener {
            let _ = listener.join();
        }
    }
}
