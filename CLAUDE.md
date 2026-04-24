# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

- Build: `cargo build`
- Check (fastest feedback): `cargo check`
- Lint: `cargo clippy --all-targets -- -D warnings`
- Format: `cargo fmt`
- Test: `cargo test` (there is currently no test suite; run a single test with `cargo test <name>` once tests exist).
- Run: `cargo run -- [CHANNEL]` — positional arg is the initial channel to join; `--auth` triggers the OAuth token flow on startup.

## Architecture

This is a thin terminal frontend over the `ttvy_core` library, which owns all IRC/WebSocket state. This repo's job is CLI arg parsing, stdin-driven commands, and styled output. When something is missing (reconnect-on-disconnect, keepalive, message parsing tweaks), it likely belongs in `ttvy_core`, not here.

**Sibling crate**: `ttvy_core` lives at `../ttvy_core` and is owned by the same author. `Cargo.toml` pins it as a `git` dependency; for local iteration point it at `path = "../ttvy_core"` temporarily.

### Event loop (`src/main.rs`)

`main` is a single `tokio::select!` over three sources:

1. `chat.receive()` — a `ChatMessage` from `ttvy_core`, rendered via `StyleConfig::display`.
2. `user_input_rx` — a line of raw text typed into stdin that didn't start with `!`; forwarded to `chat.send`.
3. `command_rx` — a parsed `CommandMessage` (lines starting with `!`); dispatched through `handle_command`.

Both input channels are produced by `input::start()`, which spawns a blocking stdin reader and a parser task. Exiting the loop falls through to `std::process::exit(0)` because the stdin reader task cannot be cleanly cancelled — the comment `//stdin receiver freezes, todo!` marks this; don't "fix" it by removing the hard exit without replacing the stdin strategy first.

### Command model (`src/input/`)

`CommandMessage` is a **data-carrying enum** (recent refactor: was previously a `CommandType` + `CommandMessage` pair — see commit `405d7e0`). Each variant either carries its argument (`Join(String)`, `SetNick(String)`, `Echo(String)`) or is a unit variant. Parsing happens in `input::command` and the result is sent over an mpsc channel to `main`.

`input::start()` returns `(JoinHandle, UserInputRx, CommandRx)` — two separate receivers rather than a single multiplexed one, because user messages and commands have different downstream handling.

### Styling (`src/output/style.rs`)

`StyleConfig` owns presentation toggles (`color`, `pad`) **and** the rendering function (`display`). Rendering was folded into `StyleConfig::display` (commit `63c4d7a`) — if adding a new toggle, add it as a field on `StyleConfig` and branch inside `display`, rather than introducing a parallel renderer.

Username coloring uses `hex_color` to parse the Twitch-provided color tag and `colored` to apply it; when `color` is off or no color is provided, usernames render unstyled.

### CLI (`src/cli_args.rs`)

`clap` derive-based. The struct is a single unified `Parser` (commit `ce9db9f` collapsed a prior `CliArgs`/`ProvidedArgs` split — don't re-introduce the split). `initial_channel` is an `Option<String>`; `authenticate` gates the OAuth flow.

## Recent refactor direction

Git history shows a consistent trend toward **collapsing abstractions** (unified arg struct, enum-with-data commands, rendering folded into config, removed `Option` bottlenecks). When in doubt, prefer the simpler/flatter shape; avoid re-introducing builder structs, wrapper types, or parallel-hierarchy enums.
