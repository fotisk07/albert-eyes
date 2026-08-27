# Albert Personality

## Goal

Turn Albert Eyes into a cute, reactive terminal character for the Albert NAS. Albert reacts only to system activity: temperature sets his mood, disk I/O sets his focus, and CPU usage sets his energy. The dashboard remains useful, redraws every two seconds, survives missing data, and never waits for Restic during a frame.

This plan extends the completed dashboard milestones in `MILESTONES.md`. The owner writes every line of Rust; implementation should optimize for learning, clarity, and visible progress rather than speed or abstraction.

## Version-one behaviour

### Rhythm and collection

| Work | Cadence | Rule |
|---|---:|---|
| Redraw; temperature, CPU, disk, uptime, memory | 2 s | Cheap file reads may run in the refresh path. |
| Copyparty status | 30 s | Display the most recent result between checks. |
| Storage usage | 60 s | Informational; it does not animate the face. |
| Restic snapshots | 10 min | Run away from the refresh path; show `Checking…` until the first result and retain the latest completed result. Never overlap checks. |

A failed refresh becomes an explicit unavailable/unknown value; it must not erase unrelated valid data or crash the dashboard.

### Signals

**Temperature — mood**

- `< 70°C`: comfortable
- `70–79°C`: warm/concerned
- `>= 80°C`: alarmed
- unavailable: puzzled

**CPU — energy**

- `< 20%`: relaxed
- `20–70%`: awake
- `> 70%`: energetic
- `>= 90%` for three samples: overwhelmed

CPU percentage is calculated from the change between two aggregate `/proc/stat` samples. The first sample is unknown, not zero.

**Disk — focus**

- no counter change: idle
- reads or writes: focused
- combined throughput `>= 10 MiB/s`: intensely focused

Disk activity is calculated from changes in `/proc/diskstats` for the device backing `/srv/storage`. The initial device selection may be a named constant verified on Albert. Counter resets, missing devices, and malformed lines produce unknown data rather than bad arithmetic.

### Character composition

Temperature controls eyebrows, mouth, and health message. Activity controls the eyes:

1. Dangerous heat overrides normal eye behaviour with an alarmed face.
2. Disk activity centers the pupils; intense activity strengthens the focused expression.
3. Without disk activity, CPU usage controls eye openness and energy.
4. When relaxed, the pupils follow a deterministic cycle: left → center → right → center → blink → center.
5. Focus remains for two frames after disk activity ends, preventing flicker.

The face must be cute, fixed-size, and stable on screen. Slow facts—storage, backup age, and service status—remain visible but do not control the version-one face. No keyboard interaction, randomness, smooth sub-second animation, colour, or configuration file is required.

### Approved milestone-two design

Milestone 2 introduces Albert’s approved neutral appearance and the final telemetry layout. The telemetry responds to live observations, but the face stays exactly as shown: centered pupils, no blinking, no pupil movement, and no expression changes.

```text
┌─────────────────────────────────────────────────────┐
│                                                     │
│           ╭───────╮             ╭───────╮           │
│           │   ●   │             │   ●   │           │
│           ╰───────╯             ╰───────╯           │
│                          ᴗ                          │
│                        ╰───╯                        │
│                                                     │
│  TEMP 52°C · comfy        CPU 18% · relaxed         │
│  RAM  46% ████░░░░░░     STORE 63% ██████░░░░       │
│  DISK R 0.0 · W 2.4 MiB/s                           │
│  UP 6h23m        COPY OK        BACKUP 3h ago       │
└─────────────────────────────────────────────────────┘
```

The values are illustrative. The frame dimensions and field positions stay fixed while values refresh. Bars represent bounded capacity only, so RAM and storage use bars while temperature, CPU activity, and disk throughput do not. Disk focus is not written as a telemetry label; future facial behaviour may communicate it instead. Missing measurements must display an explicit compact unknown/unavailable value rather than a plausible zero.

## Design boundaries

Keep these concepts separate even if they initially share `main.rs`:

```text
collect raw counters/facts
        ↓
update cached observations and calculate rates
        ↓
advance persistent character state
        ↓
render one frame
```

Raw observations are facts; thresholds are policy; character state remembers prior frames; rendering only turns state into text. Prefer standard-library facilities. Split modules only when the working program becomes difficult to navigate.

# Milestones

## 1. Responsive multi-rate dashboard

**Outcome:** The current dashboard still works and refreshes every two seconds, but slow checks are cached and Restic can no longer freeze the display.

**Work:**

- Preserve the current visible report and two-second refresh.
- Give Copyparty, storage, and Restic independent due times based on `Instant`.
- Retain their latest results between checks instead of recollecting them every frame.
- Represent Restic’s initial/in-progress state as `Checking…`.
- Move Restic execution off the refresh path using a standard-library thread and message passing; prevent overlapping checks and apply completed results safely.
- Confirm failures stay visible as unknown/unavailable and the loop continues.

**Rust lessons:** `Instant` and `Duration`, ownership of cached state, mutable updates, threads, `move` closures, channels, non-blocking receive, and modelling pending work with enums.

**Complete when:** Albert redraws on schedule while a deliberately slow Restic check runs, slow commands run only at their cadence, Ctrl-C still stops the program, and `cargo fmt`, `cargo check`.

## 2. Static Albert with live telemetry

**Outcome:** The dashboard uses the approved fixed-size Albert design and telemetry layout. Every metric refreshes at its intended cadence, but Albert’s neutral face does not move or react yet.

**Work:**

- Parse the aggregate CPU counters from `/proc/stat` into a raw sample.
- Calculate CPU utilization from consecutive samples with safe delta arithmetic.
- Identify and parse the `/proc/diskstats` row for Albert’s storage device.
- Calculate read rate, write rate, combined throughput, and whether any I/O occurred.
- Add current CPU and disk activity to the observed dashboard state.
- Render the approved centered face and telemetry card without changing its dimensions or field positions between frames.
- Show temperature and CPU descriptors beside their live values, but do not connect them to the face.
- Use capacity bars only for RAM and storage; show disk activity as read/write throughput with no `focused` label.
- Keep the face completely static: no pupil movement, blinking, or expression changes.
- Treat first samples, resets, zero elapsed work, malformed files, and missing measurements as unavailable rather than zero.
- Also refactor the code because right now it is a bit too messy. introduce new files, one for rendering for example

**Rust lessons:** data modelling with structs, iterators and parsing, `Option`/`Result`, references versus ownership, safe integer arithmetic, previous/current state, pure functions, fixed-width formatting.

**Complete when:** Albert’s approved neutral face is centered and unchanged, all telemetry fits the stable card, CPU load changes under CPU work, disk rates change during a real transfer to `/srv/storage`, RAM and storage are the only bars, idle values settle again, unavailable values do not shift the layout, and all checks pass without panics or warnings.

## 3. Frame renderer with moving eyes

**Outcome:** Albert’s pupils move together through a deterministic left → center → right → center loop. Rendering produces one complete 55×13 frame before it is written to the terminal, while telemetry and every other facial feature continue to behave exactly as they did in Milestone 2.

**Visual rule:**

```text
left      center      right
│ ●     │  │   ●   │  │     ● │
```

Each eye keeps a seven-character interior. The animation maps its four phases to horizontal pupil positions `1 → 3 → 5 → 3`, or an equivalent bounded representation. The two center phases look identical but remain distinct points in the cycle.

**Work:**

- Refactor rendering so it builds and returns one complete frame instead of printing individual rows.
- Keep face-row construction separate from telemetry-row formatting.
- Generate each pupil row from a bounded horizontal position while leaving the eye outlines as readable static templates.
- Represent the repeating animation as four persistent phases: left, center-after-left, right, and center-after-right.
- Advance the phase once per completed redraw and move both pupils together.
- Write the cursor-positioning sequence and completed frame as one output operation, then flush.
- Preserve the approved 55×13 dimensions, telemetry positions, explicit unavailable values, collection cadences, and background Restic behaviour.
- Do not add blinking, independent eye movement, randomness, keyboard control, facial reactions to telemetry, a general-purpose drawing canvas, or an LCD driver yet.

**Rust lessons:** persistent state across loop iterations, enums or bounded indices, deterministic state transitions, string construction, `std::fmt::Write`, separating frame generation from output, borrowing observations during rendering, and keeping dynamic components inside fixed geometry.

**Complete when:** Albert repeatedly looks left, center, right, and center with both pupils moving together; every generated frame remains exactly 55 characters by 13 lines; telemetry continues to refresh without shifting; missing telemetry does not stop the animation; the terminal receives a complete frame rather than row-by-row rendering; Ctrl-C still works; and `cargo fmt`, `cargo check`, and `cargo test` pass without warnings.

## 4. TBD

To be decided after Milestone 3 is complete.
