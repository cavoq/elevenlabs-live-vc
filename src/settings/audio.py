import os

class AudioSettings:
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
