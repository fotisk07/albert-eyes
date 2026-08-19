# Albert Eyes — Milestones

Albert Eyes is a first Rust project: a continuously refreshing terminal dashboard for the Raspberry Pi NAS named Albert.

## Learning rule

The project owner writes every line of Rust. An assisting agent may inspect files, run Cargo commands, explain concepts and compiler errors, review existing code, and give small conceptual hints. It must not generate, insert, or edit Rust source code.

Work on one milestone at a time. Avoid adding crates or abstractions before they are genuinely useful.

## 1. Static dashboard

Print one static, well-formatted screen containing:

- Placeholder temperature
- Placeholder uptime
- Placeholder storage usage
- Placeholder service statuses
- A short personality message

Use only the standard library and keep everything in `main.rs`.

**Complete when:** `cargo run` succeeds without warnings, displays a readable screen, and exits normally.

## 2. Raspberry Pi temperature

Replace only the placeholder temperature with the real value from `/sys/class/thermal/thermal_zone0/temp`.

Learn file reading, trimming, number parsing, millidegree conversion, and graceful failure.

**Complete when:** Albert's real temperature is displayed and failures appear as `unavailable` rather than crashing the program.

## 3. Uptime

Read `/proc/uptime` and display the result as days, hours, and minutes.

Learn text splitting, numeric conversion, duration formatting, and separating collection from presentation.

**Complete when:** real uptime is displayed and malformed or missing input is handled safely.

## 4. Memory

Read `/proc/meminfo` and display used and total memory.

Learn structured-text parsing, named-field lookup, unit conversion, and Linux memory semantics.

## 5. Storage usage

Collect usage information specifically for `/srv/storage`. Initially invoke the system `df` command rather than adding a filesystem crate.

Display:

- Used space
- Total space
- Percentage used
- A small textual usage bar

Learn child processes, exit statuses, standard output, column parsing, and command-failure handling.

**Complete when:** the program measures `/srv/storage`, not the SD-card root filesystem, and unexpected output cannot crash it.

## 6. Service status

Check:

- Copyparty as a user service

Represent states as running, stopped, failed, or unknown.

Learn external commands, exit codes, enums, and mapping machine state to display text.

## 7. Restic backup status

Check the Restic repository at `/srv/storage/backups/dell-pc` and display the age of its latest snapshot.

Show a simple state: current, stale, or unavailable. Invoke the `restic snapshots` command and handle command or parsing failures without crashing.

**Complete when:** Albert Eyes shows when the latest backup was created and warns when it is more than 48 hours old.

## 8. Status snapshot

Introduce one structure representing the complete observed state of Albert:

- Temperature
- Uptime
- Memory
- Storage
- Services
- Collection time

Adopt the conceptual flow: collect status, produce a snapshot, then render the snapshot.

Learn structs, ownership, optional data, and separation between collection and presentation. Split into modules only when one file has genuinely become uncomfortable.


## 10. Continuous refresh

Turn the one-frame report into a dashboard:

- Refresh every two seconds
- Clear or redraw the terminal
- Flush output after rendering
- Stop normally with Ctrl-C
- Do not accumulate output indefinitely

Start with the standard library. Consider `crossterm` only if basic terminal handling becomes unreliable.

## 11. Health and personality

Derive an overall state such as healthy, attention, warning, or unknown.
Give him googly eyes that adapt to what is going on, a little bit like a celullar automata
Albert should have a personality !

Use it to select:

- Albert's mood
- Status symbol
- Personality message
- Optional colours

Possible warning conditions include elevated temperature, nearly full storage, stopped services, and unavailable metrics.

## 12. Configuration

Make these choices easy to change:

- Refresh interval
- Storage path
- Services to monitor
- Warning thresholds

Begin with constants. Add a configuration file only if runtime configuration becomes genuinely useful.

## 13. Deploy to Albert

Choose either native compilation on Albert or cross-compilation from the PC. Native compilation is simpler for a first project; cross-compilation can be a later exercise.

Verify that the final program:

- Is built for ARM64
- Works without its source tree
- Works from any current directory
- Handles unavailable services
- Uses little CPU while waiting

For an SSH dashboard, run it manually or inside `tmux`. Do not create a systemd service until there is a clear need for a background process or permanently attached display.

## Prompt for an assisting agent

> I want to write every line of Rust myself. Do not generate, insert, or edit Rust source code. Do not give me a complete implementation. You may inspect my work, run commands, explain concepts and compiler errors, and provide small conceptual hints.
>
> Inspect the current project without modifying source files. Tell me only the next small task for the current milestone, explain what it teaches, and wait for me to implement it.
