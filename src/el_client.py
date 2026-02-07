from io import BytesIO
import os
from elevenlabs.client import ElevenLabs
from elevenlabs.play import play


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

    def convert_audio(self, audio: BytesIO, remove_background_noise: bool):
        audio = self.client.speech_to_speech.convert_as_stream(
            voice_id=self.voice_id,
            audio=audio,
            remove_background_noise=remove_background_noise,
        )
        play(audio)
