import cmd
import os
import colorama
from dotenv import load_dotenv
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
    prompt = f"{colorama.Fore.CYAN}(live-vc) {colorama.Style.RESET_ALL}"

    def do_clear(self, arg=None):
        """Clear the screen and print the help."""
        if os.name == 'nt':
            os.system('cls')
        else:
            os.system('clear')
        print(self.intro)

    def help_clear(self):
        print(f"Clear the screen and print the help.")


class ElevenLabsLiveVC:
    def __init__(self):
        self.audio_data = None
        self.sample_rate = 48000
        self.channels = 1


if __name__ == "__main__":
    ElevenlabsLiveVCCmd().cmdloop()
