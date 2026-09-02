mod collect;
mod status;

const AL_DEVICE: &str = "/dev/disk/by-uuid/ba307f60-44e2-42d0-b7af-589753384ebd";
const BERT_DEVICE: &str = "/dev/disk/by-uuid/f71b55fd-9bd2-427e-bc54-d1a00cc5d6ce";

fn main() {
    let status = status::AlbertStatus {
        al: collect::collect_disk_status(AL_DEVICE, "/srv/storage"),
        bert: collect::collect_disk_status(BERT_DEVICE, "/srv/recovery"),
        pi: collect::collect_pi_status(),
        backups: collect::collect_backup_statuses(),
    };
    dbg!(status);
}
