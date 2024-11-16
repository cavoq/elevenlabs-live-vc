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

    def get_audio_stream(self, audio_data: np.ndarray) -> BytesIO:
        audio_data = np.concatenate(audio_data, axis=0)
        audio_data_pcm = (audio_data * 32767).astype(np.int16)
        wav_memory = BytesIO()
        write(wav_memory, self.settings.sample_rate, audio_data_pcm)
        wav_memory.seek(0)
        return wav_memory
