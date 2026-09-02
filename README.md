# Albert Eyes

Albert Eyes is a Rust terminal display for the Albert NAS. It shows:

- Al and Bert disk presence, temperature, health, and free space
- Raspberry Pi temperature and RAM usage
- Freshness and activity of the three Restic backups

Albert has different morning, day, evening, and night animations, and becomes excited while a backup is running.

Disk SMART data is collected as root every 15 minutes by `albert-eyes-smart.timer` and cached in `/run/albert-eyes/smart.json` for the unprivileged application.
