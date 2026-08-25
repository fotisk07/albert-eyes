mod collect;
mod render;
use crate::render::render;

use std::io::Write;
use std::sync::mpsc;
use std::time::{Duration, Instant};
use std::{thread, time};

const COPYPARTY_UPDATE: u64 = 30;
const STORAGE_UPDATE: u64 = 60;
const BACKUP_UPDATE_SECS: u64 = 600;

struct Uptime {
    days: u64,
    hours: u64,
    minutes: u64,
}

struct MemoryUsage {
    used: u64,
    total: u64,
}

struct DiskSample {
    sectors_read: u64,
    sectors_written: u64,
}

enum CpStatus {
    Running,
    Stopped,
    Failed,
    Unknown,
}

enum BackupStatus {
    Checking,
    Current { age_hours: i64 },
    Stale { age_hours: i64 },
    Unavailable,
}
struct DiskActivity {
    read_mib_s: f64,
    write_mib_s: f64,
}
struct CpuSample {
    user: u64,
    nice: u64,
    system: u64,
    idle: u64,
    iowait: u64,
    irq: u64,
    softirq: u64,
    steal: u64,
}

struct StatusSnapshot {
    temperature: Option<i32>,
    uptime: Option<Uptime>,
    memory: Option<MemoryUsage>,
    storage: Option<u8>,
    cpu_usage: Option<f64>,
    copyparty: CpStatus,
    backup: BackupStatus,
    disk_activity: Option<DiskActivity>,
}

fn calculate_disk_activity(
    previous: &DiskSample,
    current: &DiskSample,
    elapsed: Duration,
) -> Option<DiskActivity> {
    const BYTES_PER_SECTOR: f64 = 512.0;
    const BYTES_PER_MIB: f64 = 1_048_576.0;

    let elapsed_secs = elapsed.as_secs_f64();

    if elapsed_secs == 0.0 {
        return None;
    }

    let read_sector_delta = current.sectors_read.checked_sub(previous.sectors_read)?;

    let write_sector_delta = current
        .sectors_written
        .checked_sub(previous.sectors_written)?;

    let read_bytes = read_sector_delta as f64 * BYTES_PER_SECTOR;
    let write_bytes = write_sector_delta as f64 * BYTES_PER_SECTOR;

    let read_mib_s = read_bytes / BYTES_PER_MIB / elapsed_secs;
    let write_mib_s = write_bytes / BYTES_PER_MIB / elapsed_secs;

    Some(DiskActivity {
        read_mib_s,
        write_mib_s,
    })
}

fn calculate_util(previous: &CpuSample, current: &CpuSample) -> Option<f64> {
    let user = current.user.checked_sub(previous.user)?;
    let nice = current.nice.checked_sub(previous.nice)?;
    let system = current.system.checked_sub(previous.system)?;
    let idle = current.idle.checked_sub(previous.idle)?;
    let iowait = current.iowait.checked_sub(previous.iowait)?;
    let irq = current.irq.checked_sub(previous.irq)?;
    let softirq = current.softirq.checked_sub(previous.softirq)?;
    let steal = current.steal.checked_sub(previous.steal)?;

    let total_delta = user
        .checked_add(nice)?
        .checked_add(system)?
        .checked_add(idle)?
        .checked_add(iowait)?
        .checked_add(irq)?
        .checked_add(softirq)?
        .checked_add(steal)?;

    if total_delta == 0 {
        return None;
    }

    let idle_delta = idle.checked_add(iowait)?;
    let busy_delta = total_delta.checked_sub(idle_delta)?;

    Some((busy_delta as f64 / total_delta as f64) * 100.0)
}

fn main() {
    // initialization
    let mut status = StatusSnapshot {
        temperature: collect::temperature(),
        uptime: collect::uptime(),
        memory: collect::memory(),
        storage: collect::storage(),
        cpu_usage: None,
        disk_activity: None,
        copyparty: collect::copyparty(),
        backup: BackupStatus::Checking,
    };

    let mut copyparty_times = Instant::now();
    let mut storage_times = Instant::now();

    let (tx, rx) = mpsc::channel();

    let worker_sender = tx.clone();
    thread::spawn(move || {
        let val = collect::backup();
        let _ = worker_sender.send(val);
    });

    let mut backup_command_running = true;
    let mut backup_times = Instant::now();

    let mut previous_cpu = None;

    // No baseline yet: first frame will show unavailable.
    let mut previous_disk: Option<(DiskSample, Instant)> = None;

    print!("\x1B[2J");
    loop {
        status.temperature = collect::temperature();
        status.uptime = collect::uptime();
        status.memory = collect::memory();

        // CPU
        let current_cpu = collect::cpu_sample();

        status.cpu_usage = match (&previous_cpu, &current_cpu) {
            (Some(previous), Some(current)) => calculate_util(previous, current),
            _ => None,
        };

        previous_cpu = current_cpu;

        // Disk
        let current_disk = collect::disk_sample();
        let current_disk_time = Instant::now();

        status.disk_activity = match (&previous_disk, &current_disk) {
            (Some((previous, previous_time)), Some(current)) => {
                let elapsed = current_disk_time.duration_since(*previous_time);
                calculate_disk_activity(previous, current, elapsed)
            }
            _ => None,
        };

        // If collection failed, this becomes None and resets the baseline.
        previous_disk = current_disk.map(|sample| (sample, current_disk_time));

        let received = rx.try_recv();
        match received {
            Ok(v) => {
                status.backup = v;
                backup_command_running = false;
            }
            Err(_) => {
                if backup_times.elapsed() >= Duration::from_secs(BACKUP_UPDATE_SECS)
                    && !backup_command_running
                {
                    let worker_sender = tx.clone();
                    thread::spawn(move || {
                        let val = collect::backup();
                        let _ = worker_sender.send(val);
                    });
                    backup_command_running = true;
                    backup_times = Instant::now();
                }
            }
        }

        if copyparty_times.elapsed() >= Duration::from_secs(COPYPARTY_UPDATE) {
            status.copyparty = collect::copyparty();
            copyparty_times = Instant::now();
        }
        if storage_times.elapsed() >= Duration::from_secs(STORAGE_UPDATE) {
            status.storage = collect::storage();
            storage_times = Instant::now();
        }

        // print!("\x1B[2J");
        print!("\x1B[H");

        render(&status);
        let _ = std::io::stdout().flush();

        let sleep_duration = time::Duration::from_secs(1);
        thread::sleep(sleep_duration);
    }
}
