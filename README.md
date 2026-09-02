# Albert Eyes

Albert Eyes is a Rust terminal display for my NAS, Albert. It shows

- Basic info about the hard drives (temperature and health)
- Basic info about the NAS (temperature and RAM usage)
- Freshness and activity of the three Restic backups

Albert has different morning, day, evening, and night animations, and becomes excited while a backup is running.

He's so CUTE!

```text
┌─────────────────────────────────────────────────────┐
│           ╭─────────╮         ╭─────────╮           │
│           │         │         │         │           │
│           │    ●    │         │    ●    │           │
│           │         │         │         │           │
│           ╰─────────╯         ╰─────────╯           │
│                        ╰───╯                        │
│                                                     │
│        AL    31°C  ✓   831G free  █████████░        │
│        BERT  26°C  ✓   427G free  █████████░        │
│        PI    52°C  · RAM 24%                        │
│     BKP XPS→AL 25m✓  XPS→BERT 1d✓  AL→BERT 1h✓      │
└─────────────────────────────────────────────────────┘
```

DISCLAIMER: I am a total Rust beginner and mainly took this project in order to learn more. My code is probably suboptimal in many ways. The ASCII animations were done using AI.

## Intro: The NAS

Searching through my stuff I found an old Raspberry Pi that I bought when I used to be into robotics. I decided to turn it into my home NAS and named him Albert! Albert is a very simple boi. He is just a Raspberry Pi connected via LAN to my local network and accessible everywhere through Tailscale (the free tier is awesome). Two hard drives are attached:

- Al: A 1 TB HDD I got from Vinted for 30€
- Bert: A 512 GB SSD I salvaged from an old PC.

And basically that is it!
The structure of the NAS is really simple:

```text
/srv
├── storage/                         # AL is mounted here
│   ├── shared/                      # Folder where I store my files.
│   │   │                            # I access it through a Copyparty web server.
│   │   ├── photos/
│   │   └── other-files/
│   │
│   └── backups/
│       └── my-pc/                   # Backups of my PC using Restic
│
└── recovery/                        # BERT is mounted here
    ├── shared/                      # Backup of /srv/storage/shared using Restic
    └── computer-backups/            # Secondary backup using Restic
```

So my workflow is pretty basic. When I want to access the NAS I use my browser (thanks copyparty). I have configured systemd timers for my PC to back up to Al and to Bert, and a systemd on Albert (the Pi) to back up the shared folder to Bert.

I also have an offsite, offline backup (important!). I intend to make a script that automatically backs it up when plugged and prompted.

## Meet Albert

This is all right and good, but random configurable scripts tucked away in `~/.config/systemd` are not very user-friendly. I wanted to give Albert some personality, so I built this little terminal dashboard. His face changes throughout the day, while the rows underneath give me the important information at a glance. If a backup is running, Albert gets excited too.

The application checks whether each disk is present and mounted, uses `df` for its free space, and reads the Pi temperature and memory usage from Linux. For each Restic repository it looks at the newest snapshot and recent lock files, allowing it to distinguish a current, stale, running, or unavailable backup. The daily backups become stale after 48 hours, while the weekday backup becomes stale after 96 hours.

SMART information is a little different because it needs root access. `albert-eyes-smart.timer` runs `scripts/collect-smart.sh` every 15 minutes and caches the disk temperatures and health in `/run/albert-eyes/smart.json`, where Albert Eyes can read them as an unprivileged user.

This is still a personal project rather than a configurable NAS dashboard: the mount paths, Restic repositories, disk UUIDs, and SMART device IDs are hard-coded in `src/collect.rs` and `scripts/collect-smart.sh`. It is made for Linux and expects Rust, `findmnt`, `df`, and `smartctl`. It can be run with `cargo run --release`.

For testing animations, `ALBERT_EYES_PHASE` can be set to `morning`, `day`, `evening`, or `night`, and `ALBERT_EYES_BACKUP` can be set to `xps-to-al`, `xps-to-bert`, or `al-to-bert`.

## Albert's moods

Run them to see them moving!

**Morning**

```text
│                                        ( (          │
│           ╭───────╮     ╭───────╮       ) )         │
│           │   ●   │     │   ●   │    ╭────╮         │
│           ╰───────╯     ╰───────╯    │    ├╮        │
│                     ╰─╯              ╰────╯         │
```

**Day**

```text
│           ╭─────────╮         ╭─────────╮           │
│           │         │         │         │           │
│           │    ●    │         │    ●    │           │
│           │         │         │         │           │
│           ╰─────────╯         ╰─────────╯           │
│                        ╰───╯                        │
```

**Evening**

```text
│                                                     │
│               ╭───────╮     ╭───────╮               │
│               │       │     │       │               │
│               │   ●   │     │   ●   │               │
│               ╰───────╯     ╰───────╯               │
│                         ╰─╯                         │
```

**Night**

```text
│                ╭────────────────────╮               │
│               ╱        ✦             ╰───◯          │
│              ╰───────────────────────────╯          │
│               ╰───────╯     ╰───────╯               │
│                                                     │
│                          ~                          │
```
