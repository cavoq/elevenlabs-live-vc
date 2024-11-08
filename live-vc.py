import cmd
import os
import colorama
from dotenv import load_dotenv
import time
import keyboard
from src.audio import *

load_dotenv()
load_dotenv("bot.env")
colorama.init()

banner = """
██╗     ██╗██╗   ██╗███████╗    ██╗   ██╗ ██████╗
██║     ██║██║   ██║██╔════╝    ██║   ██║██╔════╝
██║     ██║██║   ██║█████╗█████╗██║   ██║██║
██║     ██║╚██╗ ██╔╝██╔══╝╚════╝╚██╗ ██╔╝██║
███████╗██║ ╚████╔╝ ███████╗     ╚████╔╝ ╚██████╗
╚══════╝╚═╝  ╚═══╝  ╚══════╝      ╚═══╝   ╚═════╝
"""

description = """Description: A live voice-changer utilizing elevenlabs voice-cloning API.
Author: https://github.com/cavoq
Version: 1.0.0
"""


class ElevenlabsLiveVCCmd(cmd.Cmd):
    intro = f"""{colorama.Fore.GREEN}{banner}\n{
        description}{colorama.Style.RESET_ALL}\n"""
    prompt = f"{colorama.Fore.LIGHTBLUE_EX}(live-vc){colorama.Style.RESET_ALL}"

    def __init__(self):
        super().__init__()
        self.live_vc = ElevenLabsLiveVC.from_env()

    def do_clear(self, arg=None):
        """Clear the screen."""
        if os.name == 'nt':
            os.system('cls')
        else:
            os.system('clear')
        print(self.intro)

    def do_quit(self, arg=None):
        """Quit the program."""
        print(f"Exiting...")
        return True

    def do_set_mode(self, arg):
        """Set the mode to automatic (1) or manual (0)."""
        try:
            mode = int(arg)
            if mode not in self.live_vc.config.valid_modes():
                print(
                    f"{colorama.Fore.YELLOW}Invalid mode. Please enter 0 for manual or 1 for automatic.{
                        colorama.Style.RESET_ALL}"
                )
                return
            self.live_vc.config.mode = mode
            mode_str = "automatic" if self.mode == 1 else "manual"
            print(
                f"{colorama.Fore.GREEN}Mode set to {mode_str}.{
                    colorama.Style.RESET_ALL}"
            )
        except ValueError:
            print(
                f"{colorama.Fore.YELLOW}Invalid input. Please enter 0 for manual or 1 for automatic.{
                    colorama.Style.RESET_ALL}"
            )

    def do_get_mode(self, arg):
        """Get the current mode (1 = automatic, 0 = manual)."""
        mode_str = "automatic" if self.live_vc.config.mode == 1 else "manual"
        print(f"Current mode is {mode_str}.")


class Config:
    def __init__(self, mode: int = 0, sample_rate=48000, channels=1):
        self.mode = mode
        self.sample_rate = sample_rate
        self.channels = channels

    def valid_modes(self):
        return [0, 1]

    @classmethod
    def from_env(cls):
        return cls(
            mode=int(os.getenv("MODE", 0)),
            sample_rate=int(os.getenv("SAMPLE_RATE", 48000)),
            channels=int(os.getenv("CHANNELS", 1))
        )


class ElevenLabsLiveVC:
    def __init__(self, config: Config):
        self.config = config
        self.is_recording = False
        self.audio_data = []
        self.stream = None
        self.audio: BytesIO = None
        keyboard.on_press_key("space", self.handle_recording)

    @classmethod
    def from_env(cls):
        return cls(Config.from_env())

    def handle_recording(self, event):
        if self.is_recording:
            self.stop_recording()
        else:
            self.start_recording()

    def start_recording(self):
        print(
            f"\n{colorama.Fore.GREEN}Start recording, press space to stop...{
                colorama.Style.RESET_ALL}"
        )
        self.is_recording = True
        self.audio_data = []

        self.stream = sd.InputStream(
            callback=self.record_callback,
            channels=self.config.channels,
            samplerate=self.config.sample_rate,
            dtype='float32'
        )
        self.stream.start()
        self.is_recording = True

    def record_callback(self, indata, frames, time, status):
        if status:
            print(f"Recording status: {status}")
        if self.is_recording:
            self.audio_data.append(indata.copy())

    def process_audio_data(self):
        if not self.audio_data:
            print(
                f"{colorama.Fore.YELLOW}No audio data recorded...{
                    colorama.Style.RESET_ALL}"
            )
            return
        audio_data = np.concatenate(self.audio_data)
        audio_data = enhance_audio_quality(audio_data)
        self.audio = save_to_memory(audio_data)

    def stop_recording(self):
        print(
            f"{colorama.Fore.GREEN}Recording stopped...{
                colorama.Style.RESET_ALL}"
        )
        if self.stream:
            self.stream.stop()
            self.stream.close()
            self.stream = None

        self.process_audio_data()
        self.is_recording = False


if __name__ == "__main__":
    try:
        ElevenlabsLiveVCCmd().cmdloop()
    except KeyboardInterrupt:
        print("\nExiting gracefully...")
        exit(0)
