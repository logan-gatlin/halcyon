use std::collections::BTreeMap;
use std::sync::{
    Mutex,
    OnceLock,
};
use std::time::{
    Duration,
    Instant,
};

#[derive(Clone, Copy, Default)]
struct PhaseStats {
    total: Duration,
    max: Duration,
    count: u64,
}

#[derive(Clone, Copy)]
struct PhaseRow {
    name: &'static str,
    total: Duration,
    max: Duration,
    count: u64,
}

static PROFILING_ENABLED: OnceLock<bool> = OnceLock::new();
static PHASE_STATS: OnceLock<Mutex<BTreeMap<&'static str, PhaseStats>>> = OnceLock::new();

fn profiling_enabled() -> bool {
    *PROFILING_ENABLED.get_or_init(|| std::env::var_os("HALCYON_PROFILE").is_some())
}

fn phase_stats() -> &'static Mutex<BTreeMap<&'static str, PhaseStats>> {
    PHASE_STATS.get_or_init(|| Mutex::new(BTreeMap::new()))
}

pub struct PhaseScope {
    name: &'static str,
    started_at: Instant,
}

impl Drop for PhaseScope {
    fn drop(&mut self) {
        let elapsed = self.started_at.elapsed();
        let mut stats = phase_stats()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let entry = stats.entry(self.name).or_default();
        entry.total += elapsed;
        entry.count += 1;
        if elapsed > entry.max {
            entry.max = elapsed;
        }
    }
}

pub fn scope(name: &'static str) -> Option<PhaseScope> {
    profiling_enabled().then(|| {
        PhaseScope {
            name,
            started_at: Instant::now(),
        }
    })
}

fn format_millis(duration: Duration) -> String {
    format!("{:.3}", duration.as_secs_f64() * 1000.0)
}

#[allow(clippy::print_stderr)]
pub fn print_report() {
    if !profiling_enabled() {
        return;
    }

    let stats = phase_stats()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if stats.is_empty() {
        eprintln!("[profile] HALCYON_PROFILE is enabled, but no samples were recorded.");
        return;
    }

    let mut rows = stats
        .iter()
        .map(|(name, stats)| {
            PhaseRow {
                name,
                total: stats.total,
                max: stats.max,
                count: stats.count,
            }
        })
        .collect::<Vec<_>>();
    rows.sort_by(|left, right| right.total.cmp(&left.total));

    let grand_total = rows
        .iter()
        .fold(Duration::ZERO, |acc, row| acc + row.total)
        .as_secs_f64();

    eprintln!();
    eprintln!("=== Halcyon Profiling (HALCYON_PROFILE=1) ===");
    eprintln!("(inclusive timings; nested phases can overlap)");
    eprintln!(
        "{:<44} {:>12} {:>8} {:>12} {:>12} {:>8}",
        "phase", "total-ms", "count", "avg-ms", "max-ms", "%"
    );
    for row in rows {
        let count = row.count.max(1);
        let average = row.total / count as u32;
        let share = if grand_total > 0.0 {
            row.total.as_secs_f64() * 100.0 / grand_total
        } else {
            0.0
        };
        eprintln!(
            "{:<44} {:>12} {:>8} {:>12} {:>12} {:>7.2}",
            row.name,
            format_millis(row.total),
            row.count,
            format_millis(average),
            format_millis(row.max),
            share,
        );
    }
}
