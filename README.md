# elevenlabs-live-vc 🎙️

[![build](https://github.com/cavoq/elevenlabs-live-vc/actions/workflows/build.yml/badge.svg)](https://github.com/cavoq/elevenlabs-live-vc/actions/workflows/build.yml)

```
Live speech to speech bot using Eleven Labs API.

██╗     ██╗██╗   ██╗███████╗    ██╗   ██╗ ██████╗
██║     ██║██║   ██║██╔════╝    ██║   ██║██╔════╝
██║     ██║██║   ██║█████╗█████╗██║   ██║██║
██║     ██║╚██╗ ██╔╝██╔══╝╚════╝╚██╗ ██╔╝██║
███████╗██║ ╚████╔╝ ███████╗     ╚████╔╝ ╚██████╗
╚══════╝╚═╝  ╚═══╝  ╚══════╝      ╚═══╝   ╚═════╝

Description: A live voice-changer utilizing elevenlabs voice-cloning API.
Author: https://github.com/cavoq
```

## Features ✨

- **Real-time Voice Transformation** - Transform your voice using ElevenLabs' AI voice cloning
- **VB-Cable Integration** - Automatically routes transformed audio to VB-Cable for use in calls
- **Two Recording Modes**:
  - **Manual Mode (MODE=0)** - Press SPACE to start/stop recording
  - **Automatic Mode (MODE=1)** - Voice Activity Detection (VAD) auto-detects speech
- **Auto-Cleanup** - Temporary recordings are automatically deleted after 10 minutes

## Use Cases 💡

- Discord voice calls with voice changing
- Zoom/Teams meetings
- WhatsApp calls (via WhatsApp Desktop)
- Streaming with OBS
- Any application that supports microphone input

## Prerequisites 🧰

1. **ElevenLabs Account** - Get your API key from [ElevenLabs](https://elevenlabs.io/app/settings/api-keys)
2. **VB-Cable** - Download and install from [https://vb-audio.com/Cable/](https://vb-audio.com/Cable/)
3. **FFmpeg** - Required for audio processing. Download from [https://ffmpeg.org/download.html](https://ffmpeg.org/download.html)
4. **Python 3.12.x or 3.13.x** (3.13.12 recommended) - Download from [https://python.org](https://python.org)
5. **uv (optional, recommended)** - Install from [https://docs.astral.sh/uv/](https://docs.astral.sh/uv/)

## Installation 📦

### 1. Clone the repository

```bash
git clone https://github.com/cavoq/elevenlabs-live-vc.git
cd elevenlabs-live-vc
```

### 2. Install dependencies

#### Option A: uv (recommended)

```bash
## uv can install Python for you if you don't have 3.13.12 yet
uv python install 3.13.12
uv venv --python 3.13.12
uv sync
```

#### Option B: pip

```bash
pip install -r requirements.txt
```

### 3. Create your `.env` file

Create a file named `.env` in the root directory:

```env
# Required - Get from https://elevenlabs.io/app/settings/api-keys
API_KEY=your_elevenlabs_api_key_here

# Required - Get from ElevenLabs Voice Lab
# Go to https://elevenlabs.io/app/voice-lab → Select a voice → Copy Voice ID
VOICE_ID=your_voice_id_here

# Optional - Audio settings
SAMPLE_RATE=48000
CHANNELS=1

# Optional - Recording mode
# 0 = Manual (press SPACE to record)
# 1 = Automatic (Voice Activity Detection)
MODE=0
```

## Usage ▶️

### Starting the Application

#### Option A: uv

```bash
uv run python live-vc.py
```

#### Option B: pip

```bash
python live-vc.py
```

### Manual Mode (Default)

1. Press **SPACE** to start recording
2. Speak into your microphone
3. Press **SPACE** again to stop
4. Wait for processing - transformed voice plays to VB-Cable

### Automatic Mode (VAD)

Enable by setting `MODE=1` in `.env` or typing `set_mode 1` in the app.

1. Just start speaking - recording begins automatically
2. Stop speaking - after 1.5 seconds of silence, recording stops
3. Processing happens automatically
4. Cycle repeats - starts listening again after processing

### Commands

| Command | Description |
|---------|-------------|
| `set_mode 0` | Switch to manual mode (press SPACE) |
| `set_mode 1` | Switch to automatic mode (VAD) |
| `get_mode` | Show current mode |
| `clear` | Clear the screen |
| `quit` | Exit the application |

## Using with Call Applications

### Setup for Discord/Zoom/WhatsApp/etc.

1. **Install VB-Cable** if not already installed
2. **Run the voice changer**: `python live-vc.py`
3. **In your call app**, go to Settings → Audio/Voice
4. **Set Microphone/Input** to **"CABLE Output"**
5. Start talking - your transformed voice will be heard by others

### Audio Flow

```
┌─────────────┐    ┌──────────────────┐    ┌──────────────────┐    ┌──────────────┐
│ Your Mic    │───►│ elevenlabs-live-vc│───►│ VB-Cable Input   │───►│ Call App     │
│ (Input)     │    │ (Voice Transform)│    │ (Virtual Speaker)│    │ (Discord etc)│
└─────────────┘    └──────────────────┘    └──────────────────┘    └──────────────┘
                                                    │
                                                    ▼
                                           ┌──────────────────┐
                                           │ VB-Cable Output  │
                                           │ (Virtual Mic)    │
                                           └──────────────────┘
```

## Temporary Recordings

For debugging purposes, transformed audio is saved to the `recordings/` folder.

- Files are named: `transformed_YYYYMMDD_HHMMSS.mp3`
- **Auto-cleanup**: Files older than 10 minutes are automatically deleted
- You can play these files to verify the voice transformation is working

## Docker 🐳

```bash
docker build -t el-live-vc .
docker run --env-file .env -it --privileged -v /dev/input:/dev/input el-live-vc
```

## Troubleshooting 🛠️

### "No audio recorded" message
- Make sure you're speaking long enough (at least 0.5 seconds)
- Check that your microphone is set as the default Windows input device
- Verify microphone permissions in Windows Settings

### VB-Cable not detected
- Ensure VB-Cable is installed correctly
- The app looks for a device containing "CABLE Input" in the name
- Restart the app after installing VB-Cable

### Voice not heard in call apps
- Make sure you selected **"CABLE Output"** as the microphone in your call app
- Check that VB-Cable is not muted in Windows Sound settings
- Verify the app shows "Done! Audio sent to VB-Cable."

### API Errors
- Verify your API key is correct in `.env`
- Check your ElevenLabs account has available credits
- Ensure the Voice ID exists and you have access to it

## Configuration Options ⚙️

| Environment Variable | Default | Description |
|---------------------|---------|-------------|
| `API_KEY` | (required) | Your ElevenLabs API key |
| `VOICE_ID` | (required) | The voice to transform into |
| `SAMPLE_RATE` | 48000 | Audio sample rate in Hz |
| `CHANNELS` | 1 | Number of audio channels (1=mono) |
| `MODE` | 0 | 0=Manual, 1=Automatic (VAD) |

## License 📄

GNU General Public License v3.0 - See [LICENSE](LICENSE) for details.

## Credits 🙌

- **Author**: [cavoq](https://github.com/cavoq)
- **ElevenLabs**: [https://elevenlabs.io](https://elevenlabs.io)
- **VB-Audio**: [https://vb-audio.com](https://vb-audio.com)
