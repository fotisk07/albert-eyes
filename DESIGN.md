# Albert Eyes — design and implementation direction

## Vision

Albert Eyes is the face of Albert, not a general-purpose monitoring dashboard.

Albert consists of two storage roles:

- **Al** is the primary storage disk.
- **Bert** is the recovery disk.

The display should make their condition understandable at a glance while still giving Albert a recognisable personality. It currently renders in a terminal, but the design should remain suitable for a small dedicated screen later.

The normal view should answer four questions without requiring interaction:

1. Are Al and Bert present and healthy?
2. Are their temperatures safe?
3. How much storage remains on each disk?
4. Are all three backup paths current, and is one running now?

CPU, RAM, and Pi temperature are useful supporting information, but the disks and backups are the main subject.

## Design principles

### Character first, dashboard second

The face should remain the dominant visual element. Metrics should be concise and consistently positioned rather than turning the display into a dense table.

Albert's expression should summarize the situation before the text is read:

- calm when everything is healthy;
- excited while a backup is running;
- sleepy or alert according to the time of day;
- concerned when something requires attention.

### Prefer meaning over raw activity

Disk read/write throughput is not useful enough to occupy the default view. The important disk information is:

- presence and mount state;
- temperature;
- remaining capacity;
- SMART health and meaningful warning attributes.

Likewise, uptime and healthy background services do not need permanent space. A service such as Copyparty should become visible when it is unhealthy rather than always consuming a field.

### Never require memorising indicators

The three backup states must retain their direction labels. Three anonymous check marks are too ambiguous.

The intended compact form is similar to:

```text
│ BKP  XPS→AL 24m✓  XPS→BERT 2d✓  AL→BERT 18h✓      │
```

The exact typography may change for the target screen or font, but the source and destination should remain visible. When a job is active, its age is replaced by a clear running state:

```text
│ BKP  XPS→AL RUN!  XPS→BERT 2d✓  AL→BERT 18h✓      │
```

### Show good capacity as good

Storage should be expressed primarily as free space, for example `832G free`. If a bar is retained, a fuller bar should mean more space available. This is more intuitive than displaying a nearly empty bar when a disk is only 5% used.

### Reveal details only when they matter

Normal operation should be quiet. When a problem occurs, the affected row can replace its compact summary with the reason:

- `AL MISSING`
- `BERT 48°C HOT`
- `AL SMART: 2 pending sectors`
- `XPS→AL LATE`
- `COPY PARTY DOWN`

This allows exceptional information to be explicit without making the healthy view permanently busy.

## Normal visual structure

The normal screen should contain:

1. Albert's face.
2. One compact row for Al.
3. One compact row for Bert.
4. One row containing Pi temperature, CPU, and RAM.
5. One compact, labelled backup row.

An illustrative terminal composition is:

```text
┌─────────────────────────────────────────────────────┐
│              ╭───────╮     ╭───────╮               │
│              │   ●   │     │   ●   │               │
│              ╰───────╯     ╰───────╯               │
│                     ╰───╯                           │
│  AL    34°C ✓   832G free  █████████░              │
│  BERT  31°C ✓   427G free  █████████░              │
│  PI    53°C     CPU 12% · RAM 38%                   │
│  BKP XPS→AL 24m✓  XPS→BERT 2d✓  AL→BERT 18h✓       │
└─────────────────────────────────────────────────────┘
```

This is a direction rather than a fixed character-cell specification. Spacing, symbols, bars, and even the amount of text can be adapted when the physical display is known.

## Al and Bert

Al and Bert should be represented as distinct first-class entities rather than one generic storage percentage.

Each disk status should be able to describe:

- whether the expected physical device is present;
- whether the expected filesystem is mounted at the correct location;
- total and available capacity;
- current disk temperature;
- summarized health;
- the reason for any warning.

Device identity must not depend on names such as `/dev/sdb` or `/dev/sdc`, because these can change after reboot or reconnection. Stable filesystem UUIDs or `/dev/disk/by-id` identities should associate physical devices with the Al and Bert roles.

Mount validation is important. If an external disk is absent, Albert Eyes must not accidentally report the root filesystem's capacity for an ordinary directory at `/srv/storage` or `/srv/recovery`.

### Health interpretation

A single SMART `PASSED` value is not always sufficient. Where supported, Albert Eyes should consider the attributes most relevant to each device type.

For Al, the HDD, useful signals include:

- reallocated sectors;
- pending sectors;
- offline uncorrectable sectors;
- SMART self-assessment;
- temperature.

For Bert, the SSD, useful signals include:

- critical warnings;
- remaining life or percentage used;
- available spare;
- media or data-integrity errors;
- temperature.

The normal display should reduce these to a simple healthy or warning symbol. Attribute names and values should only take over the row when they require attention.

### SMART access

Disk SMART data requires privileges and the disks are behind USB bridges. The existing `smartmontools` service is currently not successfully monitoring them, so SMART support must first be validated for both enclosures.

Albert Eyes itself should remain unprivileged. A suitable direction is a small privileged collector that periodically writes sanitized disk-health data to a runtime file. Albert Eyes can then read that file without receiving raw block-device access. Other secure arrangements are acceptable if they preserve the same separation.

If SMART information is temporarily unavailable, the display should show an unknown health or temperature rather than implying the disk is healthy.

## Backup model

Albert has three meaningful backup paths:

- **XPS → Al**, normally daily;
- **XPS → Bert**, normally on selected weekdays;
- **Al → Bert**, normally daily.

Each should have its own semantic state, such as:

- checking;
- current;
- running;
- late;
- failed;
- unavailable.

Freshness must be aware of the intended schedule. A backup that does not run every day must not become stale merely because it is the weekend. The exact policy and grace periods should be configurable or easy to adjust rather than buried throughout rendering code.

### Observing backup activity

The Restic repositories themselves can provide much of the needed information:

- a live repository lock can indicate an active operation;
- the latest snapshot can indicate the last successful backup;
- a lock disappearing followed by a new snapshot can indicate successful completion.

This has the advantage of observing the actual backup result rather than merely whether a timer fired. Care is needed around stale locks left by interrupted processes.

The backup service running on Albert can also be observed through systemd. The two jobs initiated by XPS are not directly visible through Albert's systemd instance, so repository activity is a useful common signal. A later enhancement may allow the XPS services to publish small start, success, and failure events to Albert for faster and more precise feedback.

## Personality over the day

Time of day should affect Albert's baseline behaviour, not just swap a static face at fixed boundaries. The periods below describe moods; their exact clock boundaries should remain adjustable.

### Morning

Albert is waking up.

- eyes can begin partially open;
- movement is slow at first and becomes more attentive;
- blinks may be longer;
- an occasional yawn or sleepy mouth is appropriate.

The transition into the day should feel gradual rather than instantaneous.

### Day and noon

Albert is awake, alert, and curious.

- eyes are normally open;
- gaze changes use quick natural saccades followed by longer rests;
- blinks are brief and not perfectly periodic;
- small asymmetries between the eyes can prevent a robotic appearance.

Noon does not necessarily need a completely separate face, but it may be the most energetic part of the ordinary idle cycle.

### Afternoon

Albert remains awake but can become calmer than at noon.

- gaze movement can be less frequent;
- expressions remain attentive;
- the face should not become motionless.

This period can help make the daily progression noticeable without demanding another strongly distinct mode.

### Evening

Albert is relaxed and beginning to wind down.

- softer or slightly lowered eyelids;
- slower gaze changes;
- longer peaceful pauses;
- a relaxed mouth.

### Night

Albert is mostly asleep.

- closed-eye shapes replace ordinary pupils;
- subtle breathing-like motion or occasional small changes keep the screen alive;
- Albert may briefly peek before returning to sleep.

Night is especially useful to Albert's personality because scheduled backups can interrupt sleep and trigger the excited state.

### Transitions

The time-of-day system should support gradual transitions or a selection of neighbouring behaviours. It should not require an abrupt change from, for example, fully alert to fully asleep at one exact minute.

The initial implementation may use broad periods, while leaving room for more granular phases or environmental inputs later.

## Event-driven expressions

Time of day provides the baseline mood. Events temporarily override or modify it.

### Backup running

A running backup should make Albert visibly excited, including at night.

Possible elements include:

- wider eyes;
- quicker alternating gaze;
- a happy mouth;
- a small bounce;
- lightweight spark or accent characters;
- `RUN!` in the labelled backup entry.

The motion should be lively but still readable on a small display. The active backup's direction must remain visible.

### Backup completed

When a running operation ends and a new snapshot appears, Albert can briefly celebrate or look satisfied before returning to the current time-of-day mood.

A completion animation should be transient. The persistent evidence is the updated `just now` or short age in the backup row.

### Warning

A warning should make Albert concerned and replace compact information with a useful reason. Examples include a backup becoming late, storage becoming low, or a service becoming unavailable.

Warnings should not produce frantic animation that makes the text difficult to read.

### Critical condition

A missing disk, unsafe temperature, serious SMART signal, or nearly exhausted filesystem should take precedence over playful states. The expression and text should make it obvious which of Al or Bert needs attention.

If a backup and a critical event happen together, the critical event remains the primary message, although backup activity may still be indicated in its row.

## Animation model

The existing animation is limited to one shared horizontal pupil position and a blinking flag. A more expressive model should be able to represent independently:

- left and right eye direction;
- vertical as well as horizontal gaze;
- eyelid openness;
- blink progression;
- eyebrow or upper-eye shape;
- mouth expression;
- optional accent marks;
- small whole-face offsets for event animations.

Natural animation should emphasize pauses and transitions rather than constant random movement. Useful behaviours include:

- fast gaze movement followed by a longer dwell;
- occasional double blinks;
- slight delay between the two eyelids;
- rare glances rather than a new random choice every moment;
- smooth entry into and exit from expressive states.

The animator should receive semantic inputs such as `Night`, `BackupRunning`, or `DiskWarning`. It should not need to understand Restic, SMART attributes, or filesystem paths.

## Software direction

The code should separate four concerns:

1. **Collection** obtains raw facts from Linux, systemd, Restic repositories, and SMART data.
2. **Interpretation** converts facts into semantic states such as healthy, late, running, or critical.
3. **Personality** combines time of day, state changes, and alerts into a face or animation intent.
4. **Rendering** turns the resulting view into terminal output or, later, graphics for the dedicated display.

A conceptual application state might contain:

```rust
struct AlbertStatus {
    al: DiskStatus,
    bert: DiskStatus,
    pi: PiStatus,
    backups: BackupStatuses,
    alerts: Vec<Alert>,
}
```

The exact Rust types should evolve with implementation. The important point is that the renderer should consume meaningful state rather than directly formatting command output or interpreting device attributes.

### Configuration

Machine-specific information should be centralized rather than spread across constants. This includes:

- stable identities for Al and Bert;
- mount paths;
- Restic repository locations;
- expected backup schedules;
- health and capacity policy;
- optional time-of-day preferences.

Configuration should not expose backup passwords to the renderer. Reasonable defaults can remain in code where they make the installation simpler, provided they are not entangled with presentation logic.

### Terminal and future screen

The terminal renderer is the first frontend, not the permanent definition of the UI. Telemetry and personality state should not depend on terminal escape sequences or character-cell dimensions.

The future screen renderer may use different fonts, shapes, colors, and animation capabilities while preserving the same information hierarchy and moods. The eventual display hardware will determine whether the most reusable boundary is a renderer interface, a view model, or another representation; this need not be fixed prematurely.

## Implementation stages

### 1. Establish the semantic status model

Introduce Al, Bert, Pi, and the three backup paths as explicit concepts.

The visible result should still be simple, but the code should stop treating storage and backup as singular values. Disk read/write activity can leave the primary model, while CPU, RAM, and Pi temperature remain.

At the end of this stage, missing or unavailable information should be represented honestly rather than silently omitted.

### 2. Build the compact normal layout

Render the agreed information hierarchy:

- face;
- Al row;
- Bert row;
- Pi temperature, CPU, and RAM row;
- one labelled backup row.

The goal is an immediately readable terminal view without making final assumptions about physical screen dimensions.

### 3. Observe all backup paths

Collect the last successful state and active state for all three repositories. Interpret freshness according to each job's intended schedule.

The visual goal is for the compact backup row to be trustworthy: each age belongs to a named path, running work is obvious, and late work is distinguishable from unavailable information.

### 4. Replace the idle animation with personality states

Expand the face representation and implement time-of-day baseline moods. Add natural gaze, eyelid, blink, and mouth behaviour.

At this stage, Albert should feel noticeably different in the morning, during the day, in the evening, and at night without sacrificing readability.

### 5. Connect backup events to animation

Detect starts, completions, and relevant failures, then feed those transitions to the personality system.

The visible goal is for Albert to wake up and become excited during any of the three backups, briefly acknowledge completion, and then return naturally to the current daily mood.

### 6. Add trustworthy disk health

Validate SMART access through both USB bridges and introduce a safe privileged collection mechanism. Interpret device-specific health without exposing raw complexity in the healthy view.

At the end of this stage, Al and Bert temperatures and health symbols should represent the actual disks, not the Pi or a hard-coded Linux device name.

### 7. Prepare the dedicated display frontend

Once the display hardware is selected, implement a renderer appropriate to its resolution, color depth, and connection method. Preserve the face-first hierarchy and compact labelled backups rather than mechanically reproducing terminal characters.

## Success criteria

Albert Eyes succeeds when a glance is enough to tell that:

- both Al and Bert are present;
- both disks have safe temperatures and no known health warning;
- there is sufficient space remaining;
- the Pi itself is operating normally;
- each named backup path is current or actively running;
- Albert's expression matches the time and current activity.

It should remain pleasant to watch when everything is healthy and become direct, specific, and difficult to ignore when something is wrong.
