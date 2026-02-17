# LinWhisper

Minimal CLI voice-to-text tool for Linux. Records audio when launched, transcribes when you press Enter/Space. Fully local transcription via whisper.cpp (no network required by default).

## Privacy

LinWhisper has no account, no telemetry, and no background processes. Your microphone is **never accessed** until you explicitly run the program. Audio is captured in-memory, never written to disk. With **local mode** (default), everything stays on your machine - no network requests at all.

## Features

- Simple CLI interface - just run and speak
- Records on launch, stops when you press Enter or Space
- Local transcription via whisper.cpp (no internet required)
- Optional API transcription via any OpenAI-compatible endpoint (behind `api` feature flag)
- Audio stays in-memory, never saved to disk
- Minimal dependencies - no GUI framework required

## Dependencies

### System packages

**Debian/Ubuntu:**
```bash
sudo apt install libasound2-dev cmake libclang-dev
```

**Arch Linux:**
```bash
sudo pacman -S alsa-lib cmake clang
```

### Build tools

- [just](https://github.com/casey/just) (optional, for convenient commands)

## Setup

1. Clone the repository:
   ```bash
   git clone https://github.com/harryhaaren/linwhisper.git
   cd linwhisper
   ```

2. Build and run:

   **Local mode** (downloads model automatically on first run):
   ```bash
   just run-local
   ```

   **With a different model:**
   ```bash
   just run-local ggml-small.en.bin
   ```

   **Without just** (manual setup):
   ```bash
   # Download a whisper model for local mode
   mkdir -p ~/.local/share/linwhisper/models
   curl -L -o ~/.local/share/linwhisper/models/ggml-base.en.bin \
     https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin

   cargo build --release
   cargo run --release
   ```

### Available whisper models

Models are downloaded from [HuggingFace (ggerganov/whisper.cpp)](https://huggingface.co/ggerganov/whisper.cpp). Run `just list-models` to see options.

| Model | Size | Speed | Notes |
|-------|------|-------|-------|
| `ggml-tiny.en.bin` | ~75MB | Fastest | English only |
| `ggml-base.en.bin` | ~142MB | Fast | English only (default) |
| `ggml-small.en.bin` | ~466MB | Medium | English only, better accuracy |
| `ggml-medium.en.bin` | ~1.5GB | Slow | English only, high accuracy |
| `ggml-large-v3.bin` | ~3.1GB | Slowest | Multilingual, best accuracy |

## Usage

1. Run the program: `cargo run --release` (or `just run-local`)
2. Start speaking immediately (recording begins on launch)
3. Press **Enter** or **Space** to stop recording
4. Wait for transcription to complete
5. Your transcription will be displayed in the terminal

## Optional: API Mode

The API transcription feature is **disabled by default**. To enable it, build with the `api` feature flag:

```bash
cargo build --release --features api
```

Then set environment variables for API configuration:

**Groq:**
```bash
export API_KEY=gsk_...
cargo run --release --features api
```

**Ollama (local, no API key needed):**
```bash
export API_BASE_URL=http://localhost:11434/v1
export API_KEY=unused
export API_MODEL=whisper
cargo run --release --features api
```

**Other OpenAI-compatible endpoints** (OpenRouter, LM Studio, LocalAI, etc.) work similarly - just set `API_BASE_URL`, `API_KEY`, and `API_MODEL`.

## Stack

| Component | Crate/Tool |
|-----------|-----------|
| CLI | Pure Rust (no GUI framework) |
| Audio | cpal + hound |
| Local STT | whisper-rs (whisper.cpp) + rubato |
| API STT | reqwest + OpenAI-compatible API (optional, feature-gated) |

## License

MIT - see [LICENSE](LICENSE)
