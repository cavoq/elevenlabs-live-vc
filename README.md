# ElevenLabs Live VC

A native Rust voice changer that records microphone audio, sends it to the
ElevenLabs Speech-to-Speech API, and plays the converted voice as streaming PCM.

The Python 2.0 implementation is preserved on the [`v2.0.0-py`](https://github.com/cavoq/elevenlabs-live-vc/tree/v2.0.0-py)
branch.

## Features

- One native executable; Python is not required
- Manual push-to-record and automatic voice activity detection modes
- Persistent native input and output streams through CPAL
- Lower-latency 16 kHz raw PCM uploads
- Streamed PCM playback as ElevenLabs generates it
- Stateful high-quality sample-rate conversion
- Input/output device selection by index or name
- Small TOML configuration file

ElevenLabs currently accepts a complete input segment for Voice Changer and
streams the converted output. This application therefore minimizes
speech-end-to-playback latency; it is not frame-by-frame, bidirectional voice
conversion.

## Quick start

Download or build `elevenlabs-live-vc`, then copy `config.example.toml` to
`config.toml` beside the executable. Set at least:

```toml
api_key = "your_elevenlabs_api_key"
voice_id = "your_voice_id"
mode = "automatic"
```

Run it:

```powershell
./elevenlabs-live-vc.exe
```

In manual mode, press Space to start or stop recording and Q to exit. In
automatic mode, speak naturally and VAD submits the utterance after the
configured silence interval. Ctrl+C exits either mode.

The API key is a secret. `config.toml` is ignored by Git and must not be
committed or bundled into public releases.

## Audio devices

List available devices:

```powershell
./elevenlabs-live-vc.exe --list-devices
```

Then use an index or a case-insensitive name fragment:

```toml
input_device = "Microphone"
output_device = "VB-Cable"
```

Omit either setting to use the operating-system default.

## Configuration

| Setting | Default | Description |
| --- | --- | --- |
| `api_key` | required | ElevenLabs API key |
| `voice_id` | required | Target voice ID |
| `mode` | `automatic` | `manual` or `automatic` |
| `model_id` | `eleven_multilingual_sts_v2` | Speech-to-Speech model |
| `remove_background_noise` | `true` | ElevenLabs input noise removal |
| `input_device` | system default | Device index or name fragment |
| `output_device` | system default | Device index or name fragment |
| `low_latency_input` | `true` | Upload raw 16 kHz PCM instead of WAV |
| `output_format` | `pcm_22050` | ElevenLabs PCM response format |
| `api_output_sample_rate` | `22050` | Sample rate represented by the response |
| `vad_threshold` | `0.01` | RMS level that activates VAD |
| `vad_silence_ms` | `350` | Silence required to finish an utterance |
| `vad_min_recording_ms` | `300` | Minimum VAD recording length |
| `vad_pre_buffer_ms` | `250` | Audio retained before VAD activation |
| `min_audio_ms` | `150` | Minimum segment sent to the API |
| `trim_threshold` | `0.005` | Silence-trimming amplitude |
| `trim_padding_ms` | `100` | Context preserved at segment boundaries |

If `output_format` is changed, `api_output_sample_rate` must match it. PCM at
44.1 kHz may require a higher ElevenLabs subscription tier.

You can also pass a configuration path explicitly:

```powershell
./elevenlabs-live-vc.exe C:/path/to/my-config.toml
```

## Build from source

Install the current stable Rust toolchain. Windows also requires the Visual
Studio Desktop development with C++ workload. Debian/Ubuntu requires ALSA
development headers.

```bash
# Debian/Ubuntu only
sudo apt-get install pkg-config libasound2-dev

cargo build --release
cargo test
```

The executable is written to `target/release/`.

## Latency and quality tuning

- Lower `vad_silence_ms` for faster turn completion; raise it if phrases are
  cut apart.
- Raw PCM input is faster to decode server-side. Set `low_latency_input = false`
  to A/B test WAV input quality.
- With a clean microphone, compare `remove_background_noise = false`; excessive
  denoising can remove subtle vocal detail.
- Use headphones or a virtual audio cable to prevent converted output from
  retriggering VAD.
- Clone quality and clean source audio generally matter more than local CPU
  performance.

## License

GPL-3.0-or-later. See [LICENSE](LICENSE).
