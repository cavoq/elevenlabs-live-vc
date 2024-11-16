# elevenlabs-live-vc

```bash
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

### Command Line

```bash
python live_vc.py
```

### Docker

```bash
docker build -t el-live-vc .
docker run --env-file .env -it --privileged -v /dev/input:/dev/input el-live-vc
```
