import os

class AudioSettings:
    def __init__(
        self,
        mode: int = 0,
        sample_rate=48000,
        channels=1,
        silence_threshold=0.01,
        remove_background_noise=True,
        vad_silence_duration=0.8,
        vad_min_recording_duration=0.3,
        vad_pre_buffer_duration=0.5
    ):
        self.mode = mode
        self.sample_rate = sample_rate
        self.channels = channels
        self.silence_threshold = silence_threshold
        self.remove_background_noise = remove_background_noise
        self.vad_silence_duration = vad_silence_duration
        self.vad_min_recording_duration = vad_min_recording_duration
        self.vad_pre_buffer_duration = vad_pre_buffer_duration

    def valid_modes(self):
        return [0, 1]

    @classmethod
    def from_env(cls):
        return cls(
            mode=int(os.getenv("MODE", 0)),
            sample_rate=int(os.getenv("SAMPLE_RATE", 48000)),
            channels=int(os.getenv("CHANNELS", 1)),
            silence_threshold=float(os.getenv("SILENCE_THRESHOLD", 0.01)),
            remove_background_noise=os.getenv("REMOVE_BACKGROUND_NOISE", "1") in ("1", "true", "True", "yes", "YES"),
            vad_silence_duration=float(os.getenv("VAD_SILENCE_DURATION", 0.8)),
            vad_min_recording_duration=float(os.getenv("VAD_MIN_RECORDING_DURATION", 0.3)),
            vad_pre_buffer_duration=float(os.getenv("VAD_PRE_BUFFER_DURATION", 0.5))
        )
