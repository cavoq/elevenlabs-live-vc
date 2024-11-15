from scipy.io.wavfile import write
import numpy as np
from io import BytesIO
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

    def get_audio_stream(self) -> BytesIO:
        if not self.audio_data:
            return BytesIO()

        audio_data = np.concatenate(self.audio_data, axis=0)
        audio_data_pcm = (audio_data * 32767).astype(np.int16)

        wav_memory = BytesIO()
        write(wav_memory, self.settings.sample_rate, audio_data_pcm)
        wav_memory.seek(0)
        return wav_memory

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
