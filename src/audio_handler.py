import colorama
import keyboard
from src.audio_processor import AudioProcessor
from src.audio_recorder import AudioRecorder
from src.el_client import ElevenLabsClient


class AudioHandler:
    def __init__(self, recorder: AudioRecorder, processor: AudioProcessor, el_client: ElevenLabsClient):
        self.recorder = recorder
        self.processor = processor
        self.el_client = el_client
        keyboard.on_press_key("space", self.handle_recording)

    @classmethod
    def from_env(cls):
        return cls(
            AudioRecorder.from_env(),
            AudioProcessor.from_env(),
            ElevenLabsClient.from_env()
        )

    def handle_recording(self, event):
        if self.recorder.is_recording:
            print(
                f"\n{colorama.Fore.GREEN}Recording stopped, processing audio...{
                    colorama.Style.RESET_ALL}"
            )
            self.recorder.stop()
            audio = self.recorder.get_audio_data()
            self.el_client.convert_audio(self.processor.get_audio_stream(audio))
        else:
            print(
                f"\n{colorama.Fore.GREEN}Start recording, press space to stop...{
                    colorama.Style.RESET_ALL}"
            )
            self.recorder.start()
