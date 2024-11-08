import os
from typing import IO
from src.audio import play_audio_from_file
from elevenlabs.client import ElevenLabs
from elevenlabs.play import play, save


class ElevenLabsClient:
    def __init__(self, api_key, voice_id):
        self.client = ElevenLabs(api_key=api_key)
        self.voice_id = voice_id

    @classmethod
    def from_env(cls):
        return cls(
            os.getenv("API_KEY", None),
            os.getenv("VOICE_ID", None)
        )

    def process_audio(self, audio):
        audio = self.client.speech_to_speech.convert(
            voice_id=self.voice_id,
            audio=audio,
            enable_logging=True,
            remove_background_noise=True,
        )
        save(audio, "recordings/output.wav")
        play_audio_from_file("recordings/output.wav")
