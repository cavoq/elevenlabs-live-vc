# elevenlabs-live-vc

Live speech to speech bot for utilizing the elevenlabs API.

# Environment Setup

Create a file named `.env` in the root directory of the project and add the following environment variables:

```
API_KEY=<YOUR_ELEVENLABS_API_KEY>
VOICE_ID=<YOUR_ELEVENLABS_VOICE_ID>
SAMPLE_RATE=<YOUR_DESIRED_SAMPLE_RATE>
CHANNELS=<NUMBER_OF_AUDIO_CHANNELS>
```

## Installation

You need ffmpeg installed on your system to run this project. [https://ffmpeg.org/download.html](https://ffmpeg.org/download.html)

```bash
pip install -r requirements.txt
```

## Usage

```bash
python live_vc.py
```