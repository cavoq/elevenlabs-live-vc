import numpy as np
from io import BytesIO
from scipy.io.wavfile import write

from src.settings.audio import AudioSettings


class AudioProcessor:
    def __init__(self, settings: AudioSettings):
        self.settings = settings

    @classmethod
    def from_env(cls):
        return cls(AudioSettings.from_env())

    def _trim_silence(self, audio_data: np.ndarray) -> np.ndarray:
        if audio_data.size == 0:
            return audio_data

        if audio_data.ndim == 1:
            amplitude = np.abs(audio_data)
        else:
            amplitude = np.max(np.abs(audio_data), axis=1)

        threshold = self.settings.silence_threshold
        non_silent = np.where(amplitude > threshold)[0]
        if non_silent.size == 0:
            return audio_data

        start = int(non_silent[0])
        end = int(non_silent[-1]) + 1
        return audio_data[start:end]

    def get_audio_stream(self, audio_data: np.ndarray) -> BytesIO:
        audio_data = np.concatenate(audio_data, axis=0)
        audio_data = self._trim_silence(audio_data)
        audio_data_pcm = (audio_data * 32767).astype(np.int16)
        wav_memory = BytesIO()
        write(wav_memory, self.settings.sample_rate, audio_data_pcm)
        wav_memory.seek(0)
        return wav_memory
