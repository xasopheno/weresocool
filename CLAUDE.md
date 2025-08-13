# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

WereSoCool is a language for composing microtonal music, implemented in Rust. It includes a custom DSL (Domain Specific Language) with the `.socool` file extension, a parser built with LALRPOP, and audio rendering capabilities using PortAudio.

## Build and Development Commands

### Build
- `just build` - Build the project in debug mode
- `just build-release` - Build the project in release mode
- `cargo build --release` - Direct cargo build command

### Testing
- `just test` - Run all workspace tests in release mode and snapshot tests
- `just test_snapshot` - Run snapshot tests only
- `just test_rehash` - Regenerate snapshot test hashes
- `cargo test --workspace --release` - Run all tests directly
- `cargo test --release _generated` - Run generated tests only

### Installation
- `just install` - Install the weresocool binary locally from source
- `cargo install --path . --bin weresocool` - Direct installation command

### Code Quality
- `just format-ci` - Check code formatting (CI mode)
- `just clippy` - Run clippy linter with warnings as errors
- `just check-licenses` - Check dependency licenses using cargo-deny

### Run a Single Test
- `cargo test --release test_name` - Run a specific test by name
- `cargo test --release --package package_name` - Run tests for a specific package

## Architecture

### Core Components

1. **Parser** (`parser/`)
   - LALRPOP-based parser for `.socool` files
   - Handles imports and file processing
   - Entry point: `parser/src/parser.rs`

2. **AST** (`ast/`)
   - Abstract Syntax Tree definitions
   - Operations, generators, lists, and term structures
   - Color and WGSL shader support
   - Normalization and substitution logic

3. **Core** (`core/`)
   - Audio generation and rendering pipeline
   - PortAudio integration for real-time playback
   - Manager modules for buffer and render management
   - CSV/JSON export capabilities

4. **Instrument** (`instrument/`)
   - Voice and oscillator implementations
   - Effects: reverb, distortion, filters
   - Karplus-Strong synthesis
   - ASR (Attack-Sustain-Release) envelopes

5. **CLI** (`src/`)
   - Main binary entry point
   - Commands: `new`, `play`, `watch`, `demo`, `print`
   - File watching for live-coding support

### File Types

- `.socool` - WereSoCool composition files
- `.csv` - Data import/export format
- `.json` - Structured data export
- `.wav`, `.mp3`, `.ogg` - Audio output formats
- `.stems.zip` - Multi-track stem exports

### Key Workspace Members

- `core` - Core audio processing and rendering
- `parser` - Language parser
- `ast` - Abstract syntax tree and operations
- `instrument` - Sound synthesis and effects
- `filter` - Audio filtering
- `analyze` - Fourier analysis
- `error` - Error handling
- `shared` - Shared utilities
- `lame` - MP3 encoding support
- `vorbis` - OGG Vorbis encoding

### Platform-Specific Features

- **macOS**: Full feature set with MP3/OGG support, MIDI bridge capability
- **Linux**: ALSA backend, full codec support
- **Windows**: Limited codec support (no MP3/OGG by default)

### MIDI Support (macOS)

When using `--midi` flag, the system attempts to spawn a MIDI bridge server. The server path can be configured via `WSC_MIDI_SERVER` environment variable, defaulting to `/Users/danny/code/weresocool_midi/target/debug/rust-midi2-ump-jit`.

## Development Workflow

1. Compositions are written in `.socool` files
2. Parser converts `.socool` to AST
3. AST is normalized and operations are applied
4. Normalized form is converted to renderable audio
5. Audio is rendered via PortAudio or exported to file

## Dependencies

### System Dependencies
- **macOS**: `lame`, `libvorbis` (via Homebrew)
- **Linux**: `lame`, `vorbis-tools` (via package manager)
- **Windows**: Limited codec support

### Build Tools
- Rust toolchain (via rustup)
- Just command runner
- cargo-deny (for license checking)