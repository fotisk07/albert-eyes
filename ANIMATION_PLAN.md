# Albert Eyes — animation implementation plan

## Goal

Give Albert a recognisable daily routine while keeping the face readable, the telemetry stationary, and the implementation small enough to tune interactively.

The four baseline scenes are:

- **Morning:** sleepy Albert with coffee and an occasional sipping sequence.
- **Day:** the clean normal face, casually looking around and blinking.
- **Evening:** no prop; Albert becomes sleepy, yawns, and briefly nods off.
- **Night:** Albert sleeps under a soft nightcap while breathing and emitting drifting `Z`s.

Backup excitement is a complete scene override. Albert drops the time-of-day props, opens large eyes, smiles broadly, and moves around energetically until the backup finishes.

## Principles

### Preserve a fixed layout

Every scene occupies the same fixed face canvas. Props and movement stay inside it, so the Al, Bert, Pi, and backup rows never move.

### Separate intent from terminal drawing

The animator produces a semantic `FacePose`: phase, eye state, mouth, gaze, vertical offset, and a small phase-specific frame number. The renderer translates that pose into terminal characters.

This keeps timing and behaviour out of the renderer and leaves room for a graphical renderer later.

### Use one state machine

There is one animator and one active action at a time. Existing gaze, dwell, and blink actions remain. Short sequences are added for sipping, yawning, nodding, and sleeping.

### Use the right clock for each job

Local wall time selects the daily phase. A monotonic `Instant` advances animation actions. Animation should not be affected by wall-clock corrections.

### Make every phase testable

An `ALBERT_EYES_PHASE` environment override allows any scene to be viewed regardless of the current hour:

```bash
ALBERT_EYES_PHASE=morning cargo run
ALBERT_EYES_PHASE=day cargo run
ALBERT_EYES_PHASE=evening cargo run
ALBERT_EYES_PHASE=night cargo run
```

Without the variable, Albert selects the phase from local time. Backup excitement can be previewed safely with `ALBERT_EYES_BACKUP=xps-to-al`, `xps-to-bert`, or `al-to-bert` without creating a fake Restic lock.

## Pose vocabulary

The pose carries only properties currently needed by the approved scenes:

- daily phase;
- horizontal pupil position;
- open or closed eyes;
- smile, relaxed, yawning, sleeping, or hidden mouth;
- a small vertical offset for nodding and breathing;
- a scene frame used to position coffee or `Z`s.

More states should be added only when a concrete animation requires them.

## Scene behaviour

### Morning

Default state:

- open eyes with a relaxed mouth;
- slower gaze and longer blinks than daytime;
- coffee mug resting beside Albert with visible steam.

Sip sequence:

1. Albert looks toward the mug.
2. The mug approaches.
3. It moves beneath the mouth.
4. Albert closes his eyes while drinking.
5. The mug returns and Albert resumes the sleepy idle state.

The mug is part of the morning scene, but sipping is occasional rather than continuous.

### Day

Default state:

- open eyes;
- small smile;
- no prop;
- ordinary casual gaze movement.

Albert alternates quick horizontal glances with longer still periods and occasional brief blinks. This remains the least theatrical scene.

### Evening

Default state:

- open eyes and a relaxed mouth;
- gaze stays nearer the centre;
- slower movement and longer pauses.

Yawn sequence:

1. Mouth opens slightly.
2. Eyes close as the mouth opens fully.
3. Albert holds the yawn briefly.
4. Mouth closes and the relaxed open eyes return.

Nod sequence:

1. Eyes close for longer than a normal blink.
2. Face sinks slightly within the reserved canvas.
3. Albert rests there briefly.
4. Face rises and the relaxed eyes return.

These actions make evening distinct without adding an unnecessary prop.

### Night

Default state:

- eyes remain closed;
- sleeping mouth;
- nightcap remains visible;
- no ordinary gaze movement.

Sleep cycle:

1. Face rests with relaxed closed eyes.
2. Closed eyes narrow slightly and the face settles lower.
3. Mouth opens on the exhale.
4. Three `Z`s drift diagonally upward, becoming smaller.
5. Face rises and the cycle rests before repeating.

The movement should remain gentle and clearly different from blinking.

## Backup-running override

If any of the three backup paths is running, the animator temporarily replaces the normal phase behaviour:

- sleeping and yawning stop;
- eyes open and move around with shorter pauses;
- the mouth becomes a broad happy smile;
- coffee, nightcap, sleep, and other time-of-day details disappear for the duration of the backup.

When the backup stops, Albert returns to the default behaviour for the current phase. A separate completion celebration can be added later if it proves useful.

## Renderer work

The renderer reserves one face canvas large enough for the nightcap and coffee cup. It provides reusable drawing operations for:

- placing text at a canvas coordinate;
- drawing one of the three eye states;
- drawing the selected mouth;
- drawing the ordinary face at a chosen offset.

Phase-specific drawing then adds:

- the mug and steam in the morning;
- nothing in the day;
- no prop in the evening;
- the nightcap and drifting `Z`s at night.

All phase renderers return the same number of rows.

## Animator work

1. Determine the phase from the environment override or local hour.
2. Reset to the phase's default pose whenever the phase changes.
3. Continue using gaze, dwell, and blink actions for awake phases.
4. Choose phase-appropriate actions after each dwell.
5. Advance short sequence frames using monotonic deadlines.
6. Restore the phase's default expression when a sequence completes.

## Validation

For each forced phase:

- verify that the telemetry does not move;
- verify that every drawing fits inside the card;
- watch several action cycles for unnatural jumps;
- confirm that sleeping movement is visually distinct from an ordinary blink;
- ensure the morning mug and nightcap are immediately recognisable;
- tune motion only after seeing it in the real terminal.

After the baseline scenes and running override are accepted, a brief backup-completion reaction can be considered separately.
