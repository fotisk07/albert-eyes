mod animation;
mod collect;
mod render;
use crate::collect::{ActivityTelemetry, BackupStatus, CheapTelemetry, CpStatus};

use std::io::Write;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

const RENDER_UPDATE_MSECS: u64 = 200;
const TUM_DIC_UPDATE_SECS: u64 = 1;
const STORAGE_UPDATE_SECS: u64 = 60;
const COPYPARTY_UPDATE_SECS: u64 = 120;
const BACKUP_UPDATE_SECS: u64 = 300;

struct StatusSnapshot {
    cheap_telemetry: CheapTelemetry,
    activity_telemetry: ActivityTelemetry,
    storage: Option<u8>,
    copyparty: CpStatus,
    backup: BackupStatus,
}

struct Timers {
    tum_dic: Instant, // temperature, uptime, memory, disk ,cpu
    copyparty: Instant,
    storage: Instant,
    backup: Instant,
}

fn main() {
    let mut activity_collector = collect::ActivityCollector::new();

    let mut status = StatusSnapshot {
        cheap_telemetry: collect::CheapTelemetry::collect(),
        activity_telemetry: activity_collector.collect(),
        storage: collect::storage(),
        copyparty: collect::copyparty(),
        backup: BackupStatus::Checking,
    };

    let mut timing = Timers {
        tum_dic: Instant::now(),
        copyparty: Instant::now(),
        storage: Instant::now(),
        backup: Instant::now(),
    };

    // Get backup command spawning
    let (tx, rx) = mpsc::channel();
    let worker_sender = tx.clone();
    thread::spawn(move || {
        let val = collect::backup();
        let _ = worker_sender.send(val);
    });

    let mut backup_command_running = true;

    let mut animation = animation::IdleAnimation::new();

    print!("\x1B[2J");
    loop {
        animation.update();

        if timing.tum_dic.elapsed() >= Duration::from_secs(TUM_DIC_UPDATE_SECS) {
            status.cheap_telemetry = collect::CheapTelemetry::collect();
            status.activity_telemetry = activity_collector.collect();

            timing.tum_dic = Instant::now()
        };

        if timing.copyparty.elapsed() >= Duration::from_secs(COPYPARTY_UPDATE_SECS) {
            status.copyparty = collect::copyparty();
            timing.copyparty = Instant::now();
        }
        if timing.storage.elapsed() >= Duration::from_secs(STORAGE_UPDATE_SECS) {
            status.storage = collect::storage();
            timing.storage = Instant::now();
        }

        let received = rx.try_recv();
        match received {
            Ok(v) => {
                status.backup = v;
                backup_command_running = false;
            }
            Err(_) => {
                if timing.backup.elapsed() >= Duration::from_secs(BACKUP_UPDATE_SECS)
                    && !backup_command_running
                {
                    let worker_sender = tx.clone();
                    thread::spawn(move || {
                        let val = collect::backup();
                        let _ = worker_sender.send(val);
                    });
                    backup_command_running = true;
                    timing.backup = Instant::now();
                }
            }
        }

        print!("\x1B[H{}", render::render(&status, animation.pupils()));
        let _ = std::io::stdout().flush();

        thread::sleep(Duration::from_millis(RENDER_UPDATE_MSECS));
    }
}
