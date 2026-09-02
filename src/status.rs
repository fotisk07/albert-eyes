#[derive(Debug)]
pub struct AlbertStatus {
    pub al: DiskStatus,
    pub bert: DiskStatus,
    pub pi: PiStatus,
    pub backups: BackupStatuses,
}

#[derive(Debug)]
pub struct DiskStatus {
    pub availability: DiskAvailability,
    pub temperature_c: Option<u8>,
    pub available_gib: Option<u16>,
    pub total_gib: Option<u16>,
    pub health: Option<DiskHealth>,
}

#[derive(Debug)]
pub struct PiStatus {
    pub temperature_c: Option<u8>,
    pub ram_percent: Option<u8>,
}

#[derive(Debug)]
pub struct BackupStatuses {
    pub xps_to_al: BackupStatus,
    pub xps_to_bert: BackupStatus,
    pub al_to_bert: BackupStatus,
}

#[derive(Debug)]
pub enum DiskAvailability {
    Mounted,
    Unmounted,
    Missing,
    Unknown,
}
#[derive(Debug)]
pub enum DiskHealth {
    Healthy,
    Sick,
}
#[derive(Debug)]
pub enum BackupStatus {
    Current,
    Stale,
}
