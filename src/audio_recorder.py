import numpy as np
import sounddevice as sd

from src.settings.audio import AudioSettings


class AudioRecorder:
    def __init__(self, settings: AudioSettings):
        self.settings = settings
        self.is_recording = False
        self.audio_data = []
        self.stream = None

    @classmethod
    def from_env(cls):
        return cls(AudioSettings.from_env())

    def get_audio_data(self) -> np.ndarray:
        return self.audio_data

    def callback(self, indata, frames, time, status):
        if self.is_recording:
            self.audio_data.append(indata.copy())

    def start(self):
        if not self.is_recording:
            self.is_recording = True
            self.audio_data = []

            self.stream = sd.InputStream(
                callback=self.callback,
                channels=self.settings.channels,
                samplerate=self.settings.sample_rate,
                dtype='float32'
            )
            self.stream.start()

    def stop(self):
        if self.is_recording and self.stream is not None:
            self.stream.stop()
            self.stream.close()
            self.is_recording = False
