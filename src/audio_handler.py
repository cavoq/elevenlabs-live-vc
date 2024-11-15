import io
import wave
import colorama
import keyboard
import numpy as np
import sounddevice as sd
from src.audio_recorder import AudioRecorder
from src.el_client import ElevenLabsClient
from pydub import AudioSegment


class AudioHandler:
    def __init__(self, recorder: AudioRecorder, el_client: ElevenLabsClient):
        self.recorder = recorder
        self.el_client = el_client
        keyboard.on_press_key("space", self.handle_recording)

    @classmethod
    def from_env(cls):
        return cls(
            AudioRecorder.from_env(),
            ElevenLabsClient.from_env()
        )

    def handle_recording(self, event):
        if self.recorder.is_recording:
            print(
                f"{colorama.Fore.GREEN}Recording stopped, processing audio...{
                    colorama.Style.RESET_ALL}"
            )
            self.recorder.stop()
            self.el_client.convert_audio(self.recorder.get_audio_stream())
        else:
            print(
                f"\n{colorama.Fore.GREEN}Start recording, press space to stop...{
                    colorama.Style.RESET_ALL}"
            )
            self.recorder.start()
