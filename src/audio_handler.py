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

        if self.recorder.vad_enabled:
            self.recorder.set_vad_callback(self.process_vad_recording)

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
            self.el_client.convert_audio(
                self.processor.get_audio_stream(audio),
                remove_background_noise=self.recorder.settings.remove_background_noise
            )
            if self.recorder.vad_enabled:
                self.recorder.start_continuous()
        else:
            print(
                f"\n{colorama.Fore.GREEN}Start recording, press space to stop...{
                    colorama.Style.RESET_ALL}"
            )
            self.recorder.start()

    def process_vad_recording(self):
        audio = self.recorder.get_audio_data()
        audio_stream = self.processor.get_audio_stream(audio)
        if audio_stream is None:
            print(f"{colorama.Fore.YELLOW}No usable audio recorded. Try speaking longer.{colorama.Style.RESET_ALL}")
            self.recorder.start_continuous()
            return
        self.el_client.convert_audio(
            audio_stream,
            remove_background_noise=self.recorder.settings.remove_background_noise
        )
        self.recorder.start_continuous()

    def start_vad_mode(self):
        print(f"{colorama.Fore.CYAN}=== VAD Mode Active ==={colorama.Style.RESET_ALL}")
        print(f"{colorama.Fore.CYAN}Speak naturally - recording starts/stops automatically{colorama.Style.RESET_ALL}")
        print(f"{colorama.Fore.CYAN}Press SPACE to manually trigger, Ctrl+C to exit{colorama.Style.RESET_ALL}")
        self.recorder.start_continuous()
