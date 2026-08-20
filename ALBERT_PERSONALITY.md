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

**Complete when:** Albert redraws on schedule while a deliberately slow Restic check runs, slow commands run only at their cadence, Ctrl-C still stops the program, and `cargo fmt`, `cargo check`, and `cargo test` pass without warnings.

## 2. Live CPU and disk activity

**Outcome:** The report visibly changes with real CPU and storage-disk activity, while the face remains unchanged.

**Work:**

- Parse the aggregate CPU counters from `/proc/stat` into a raw sample.
- Calculate CPU utilization from consecutive samples with safe delta arithmetic.
- Identify and parse the `/proc/diskstats` row for Albert’s storage device.
- Calculate read rate, write rate, combined throughput, and whether any I/O occurred.
- Add current CPU and disk activity to the observed dashboard state and textual report.
- Treat first samples, resets, zero elapsed work, and malformed files as unavailable.
- Add focused parser/delta tests using short fixed input strings.

**Rust lessons:** data modelling with structs, iterators and parsing, `Option`/`Result`, references versus ownership, integer arithmetic, previous/current state, pure functions, and unit tests.

**Complete when:** CPU load changes under CPU work, disk activity changes during a real transfer to `/srv/storage`, idle values settle again, invalid sample tests pass, and the dashboard never panics.

## 3. Cute reactive Albert

**Outcome:** A fixed-size cute face reacts immediately and predictably to temperature, CPU, and disk activity.

**Work:**

- Model temperature mood, CPU energy, disk focus, and the composed expression with enums or similarly explicit types.
- Derive those states using the version-one thresholds; keep policy outside collectors and rendering.
- Design fixed-width face frames for comfortable, warm, alarmed, puzzled, relaxed, energetic, focused, and intensely focused combinations.
- Render the face and one short personality message without shifting the metrics.
- Enforce composition priority: dangerous heat, disk focus, CPU energy, then relaxed behaviour.
- Test threshold boundaries and representative combinations as pure logic.

**Rust lessons:** enums and exhaustive `match`, associated data, pure transformations, borrowing shared input, expression-oriented control flow, separation of concerns, and table-driven tests.

**Complete when:** controlled CPU/disk activity and temperature fixtures select the expected face and message, real activity visibly changes Albert, the layout remains stable, and all checks pass.

## 4. Persistent behaviour and Albert-ready finish

**Outcome:** Albert feels alive rather than merely swapping icons, and the program is robust enough to run on the NAS.

**Work:**

- Add persistent character state for the idle eye cycle, high-CPU streak, and two-frame focus hold.
- Advance the state once per completed two-second observation; avoid randomness and timing drift.
- Keep thresholds, paths, device name, and cadences in clearly named constants.
- Refine concise, cute messages without hiding warnings or unavailable data.
- Split collection, personality, and rendering into modules only if this improves navigation.
- Verify low idle CPU use, command cadence, terminal redraw, failure behaviour, and operation from any working directory on Albert.
- Add tests for state transitions and create a final Git checkpoint.

**Rust lessons:** finite-state machines, state transitions, invariants, methods and modules, visibility, deterministic testing, refactoring, and release builds.

**Complete when:** Albert wanders while idle, focuses during and briefly after disk I/O, becomes energetic under CPU load, reacts clearly to heat, remains responsive during Restic checks, and runs successfully on Albert with `cargo build --release`.
