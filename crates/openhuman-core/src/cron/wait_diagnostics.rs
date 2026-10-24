//! Temporary PR17 diagnostic: live stages of storage and cron requests.
use std::collections::BTreeMap;
use std::io::Write;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{LazyLock, Mutex, Once};
use std::time::{Duration, Instant};

struct Entry {
    label: String,
    phase: &'static str,
    started: Instant,
    changed: Instant,
}

static LIVE: LazyLock<Mutex<BTreeMap<u64, Entry>>> = LazyLock::new(Mutex::default);
static NEXT: AtomicU64 = AtomicU64::new(1);
static START: Once = Once::new();

pub(crate) struct Span(u64);

impl Span {
    pub(crate) fn new(label: impl Into<String>, phase: &'static str) -> Self {
        let id = NEXT.fetch_add(1, Ordering::Relaxed);
        let now = Instant::now();
        LIVE.lock().unwrap_or_else(std::sync::PoisonError::into_inner).insert(
            id,
            Entry { label: label.into(), phase, started: now, changed: now },
        );
        START.call_once(|| {
            std::thread::Builder::new()
                .name("pr17-wait-watch".into())
                .spawn(|| loop {
                    std::thread::sleep(Duration::from_secs(30));
                    let entries = {
                        let live = LIVE.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
                        live.iter()
                            .filter(|(_, entry)| entry.started.elapsed() >= Duration::from_secs(20))
                            .map(|(id, entry)| format!(
                                "PR17_WAIT id={id} age={:.1}s stage_age={:.1}s stage={} label={}\n",
                                entry.started.elapsed().as_secs_f64(),
                                entry.changed.elapsed().as_secs_f64(),
                                entry.phase,
                                entry.label,
                            ))
                            .collect::<String>()
                    };
                    if !entries.is_empty() {
                        let _ = std::io::stderr().lock().write_all(entries.as_bytes());
                    }
                })
                .expect("start PR17 wait diagnostic");
        });
        Self(id)
    }

    pub(crate) fn phase(&self, phase: &'static str) {
        if let Some(entry) = LIVE.lock().unwrap_or_else(std::sync::PoisonError::into_inner).get_mut(&self.0) {
            entry.phase = phase;
            entry.changed = Instant::now();
        }
    }
}

impl Drop for Span {
    fn drop(&mut self) {
        LIVE.lock().unwrap_or_else(std::sync::PoisonError::into_inner).remove(&self.0);
    }
}
