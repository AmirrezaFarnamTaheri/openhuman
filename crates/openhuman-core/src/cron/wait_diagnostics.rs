//! Temporary PR17 diagnostic: live stages of storage and cron requests.
use std::collections::BTreeMap;
use std::io::Write;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::sync::LazyLock;
use std::time::{Duration, Instant};

const REPORT_INTERVAL: Duration = Duration::from_secs(30);
const MIN_AGE: Duration = Duration::from_secs(20);

struct Entry {
    label: String,
    thread: String,
    phase: &'static str,
    started: Instant,
    changed: Instant,
}

enum Kind {
    Start {
        id: u64,
        label: String,
        thread: String,
        phase: &'static str,
    },
    Phase {
        id: u64,
        phase: &'static str,
    },
    End {
        id: u64,
    },
}

struct Event {
    at: Instant,
    kind: Kind,
}

static NEXT: AtomicU64 = AtomicU64::new(1);
static EVENTS: LazyLock<Sender<Event>> = LazyLock::new(|| {
    let (sender, receiver) = mpsc::channel();
    std::thread::Builder::new()
        .name("pr17-wait-watch".into())
        .spawn(move || watch(receiver))
        .expect("start PR17 wait diagnostic");
    sender
});

pub(crate) struct Span {
    id: u64,
    events: &'static Sender<Event>,
}

impl Span {
    pub(crate) fn new(label: impl Into<String>, phase: &'static str) -> Self {
        let at = Instant::now();
        let id = NEXT.fetch_add(1, Ordering::Relaxed);
        let events = &*EVENTS;
        let _ = events.send(Event {
            at,
            kind: Kind::Start {
                id,
                label: label.into(),
                thread: std::thread::current()
                    .name()
                    .unwrap_or("<unnamed>")
                    .to_string(),
                phase,
            },
        });
        Self { id, events }
    }

    pub(crate) fn id(&self) -> u64 {
        self.id
    }

    pub(crate) fn phase(&self, phase: &'static str) {
        let _ = self.events.send(Event {
            at: Instant::now(),
            kind: Kind::Phase { id: self.id, phase },
        });
    }
}

impl Drop for Span {
    fn drop(&mut self) {
        let _ = self.events.send(Event {
            at: Instant::now(),
            kind: Kind::End { id: self.id },
        });
    }
}

fn apply(live: &mut BTreeMap<u64, Entry>, event: Event) {
    let Event { at, kind } = event;
    match kind {
        Kind::Start {
            id,
            label,
            thread,
            phase,
        } => {
            live.insert(
                id,
                Entry {
                    label,
                    thread,
                    phase,
                    started: at,
                    changed: at,
                },
            );
        }
        Kind::Phase { id, phase } => {
            if let Some(entry) = live.get_mut(&id) {
                // Concurrent shared borrows can send an older timestamp later.
                if at >= entry.changed {
                    entry.phase = phase;
                    entry.changed = at;
                }
            }
        }
        Kind::End { id } => {
            live.remove(&id);
        }
    }
}

fn watch(receiver: Receiver<Event>) {
    let mut live = BTreeMap::new();
    let mut next_report = Instant::now() + REPORT_INTERVAL;
    loop {
        match receiver.recv_timeout(next_report.saturating_duration_since(Instant::now())) {
            Ok(event) => apply(&mut live, event),
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => return,
        }
        if Instant::now() < next_report {
            continue;
        }

        // Apply queued phase changes and drops before reporting old spans.
        while let Ok(event) = receiver.try_recv() {
            apply(&mut live, event);
        }
        let now = Instant::now();
        let entries = live
            .iter()
            .filter(|(_, entry)| now.saturating_duration_since(entry.started) >= MIN_AGE)
            .map(|(id, entry)| {
                format!(
                    "PR17_WAIT id={id} age={:.1}s stage_age={:.1}s stage={} thread={} label={}\n",
                    now.saturating_duration_since(entry.started).as_secs_f64(),
                    now.saturating_duration_since(entry.changed).as_secs_f64(),
                    entry.phase,
                    entry.thread,
                    entry.label,
                )
            })
            .collect::<String>();
        if !entries.is_empty() {
            let _ = std::io::stderr().lock().write_all(entries.as_bytes());
        }
        next_report = Instant::now() + REPORT_INTERVAL;
    }
}
