//! Dev-mode file watcher. Posts `UserEvent::HotReload` to the winit event
//! loop whenever a relevant source file under the app root changes.
//!
//! The watcher debounces native FS events (~150 ms quiet period) so a save
//! that fires Modify+Modify+Modify, or a `git checkout` touching dozens of
//! files, coalesces into one reload.

use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;
use std::time::{Duration, Instant};

use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use winit::event_loop::EventLoopProxy;

use crate::app::UserEvent;
use crate::terminal_colors;

const DEBOUNCE: Duration = Duration::from_millis(150);

/// File extensions that trigger a reload. Other writes (lockfiles, images,
/// editor swap files) are ignored.
const RELEVANT_EXTENSIONS: &[&str] =
    &["ts", "tsx", "js", "jsx", "mjs", "cjs", "mts", "cts", "json"];

/// Path components that mark a directory we never want to watch — vendor
/// installs, build outputs, VCS metadata.
const IGNORED_COMPONENTS: &[&str] = &[
    "node_modules",
    ".git",
    ".hg",
    ".svn",
    "dist",
    "target",
    ".turbo",
    ".next",
    ".cache",
];

/// Holds the underlying `notify` watcher and the dispatcher thread. Drop
/// shuts both down: the channel sender is closed (worker thread exits) and
/// the watcher releases its OS handles.
pub struct HmrWatcher {
    _watcher: RecommendedWatcher,
}

impl HmrWatcher {
    /// Start watching `app_root` recursively. FS events are filtered down to
    /// real source-file edits and posted as `UserEvent::HotReload`.
    pub(crate) fn start(app_root: &Path, proxy: EventLoopProxy<UserEvent>) -> notify::Result<Self> {
        let (tx, rx): (Sender<Event>, Receiver<Event>) = channel();

        let mut watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
            if let Ok(event) = res {
                let _ = tx.send(event);
            }
        })?;

        watcher.watch(app_root, RecursiveMode::Recursive)?;

        let proxy = proxy.clone();
        let app_root = app_root.to_path_buf();
        thread::Builder::new()
            .name("uzumaki-hmr-watcher".into())
            .spawn(move || dispatcher_loop(rx, proxy, app_root))
            .expect("failed to spawn hmr watcher thread");

        Ok(Self { _watcher: watcher })
    }
}

fn dispatcher_loop(rx: Receiver<Event>, proxy: EventLoopProxy<UserEvent>, app_root: PathBuf) {
    eprintln!(
        "{} watching {}",
        terminal_colors::cyan_bold("[hmr]"),
        app_root.display()
    );

    loop {
        // Block until the next event. Disconnected = parent dropped the
        // watcher; exit cleanly.
        let Ok(first) = rx.recv() else {
            return;
        };
        if !is_relevant(&first) {
            continue;
        }

        // Coalesce: keep draining until DEBOUNCE has elapsed without any new
        // event arriving.
        let mut last_seen = Instant::now();
        loop {
            match rx.recv_timeout(DEBOUNCE) {
                Ok(event) => {
                    if is_relevant(&event) {
                        last_seen = Instant::now();
                    }
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    if last_seen.elapsed() >= DEBOUNCE {
                        break;
                    }
                }
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => return,
            }
        }

        // Surface a single line so users see HMR is firing.
        eprintln!("{} change detected", terminal_colors::cyan_bold("[hmr]"));

        if proxy.send_event(UserEvent::HotReload).is_err() {
            // Event loop is gone — nothing left to do.
            return;
        }
    }
}

fn is_relevant(event: &Event) -> bool {
    if !matches!(
        event.kind,
        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
    ) {
        return false;
    }

    event.paths.iter().any(path_is_watchable)
}

fn path_is_watchable(path: &PathBuf) -> bool {
    let ext_ok = path
        .extension()
        .and_then(|s| s.to_str())
        .is_some_and(|ext| RELEVANT_EXTENSIONS.iter().any(|allowed| *allowed == ext));
    if !ext_ok {
        return false;
    }
    !path.components().any(|component| {
        component
            .as_os_str()
            .to_str()
            .is_some_and(|s| IGNORED_COMPONENTS.iter().any(|ignored| *ignored == s))
    })
}
