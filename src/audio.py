from io import BytesIO
import sounddevice as sd
import numpy as np
from scipy.signal import butter, filtfilt
from scipy.io.wavfile import write, read
import librosa


def record_audio(duration: int, sample_rate=48000, channels=1) -> np.ndarray:
    audio_data = sd.rec(int(sample_rate * duration),
                        samplerate=sample_rate, channels=channels, dtype='float32')
    sd.wait()
    return audio_data


def compress_audio(audio_data: np.ndarray, threshold=0.5, ratio=4) -> np.ndarray:
    audio_data_compressed = np.copy(audio_data)
    audio_data_compressed[audio_data_compressed > threshold] = threshold + (
        audio_data_compressed[audio_data_compressed > threshold] - threshold) / ratio
    return audio_data_compressed


def apply_equalization(audio_data: np.ndarray, sample_rate=48000, lowcut=100, highcut=3000) -> np.ndarray:
    b, a = butter(1, [lowcut / (0.5 * sample_rate),
                  highcut / (0.5 * sample_rate)], btype='band')
    return filtfilt(b, a, audio_data, axis=0)


def trim_silence(audio_data: np.ndarray) -> np.ndarray:
    return librosa.effects.trim(audio_data.flatten())[0]


def apply_dithering(audio_data: np.ndarray) -> np.ndarray:
    dither = np.random.uniform(-1/32768, 1/32768, audio_data.shape)
    return audio_data + dither


def enhance_audio_quality(audio_data: np.ndarray, sample_rate=48000) -> np.ndarray:
    audio_data = compress_audio(audio_data)
    audio_data = apply_equalization(
        audio_data, sample_rate, lowcut=100, highcut=3000)
    audio_data = trim_silence(audio_data)
    audio_data = apply_dithering(audio_data)
    return audio_data


def save_to_memory(audio_data: np.ndarray, sample_rate=48000) -> BytesIO:
    audio_data_pcm = (audio_data * 32767).astype(np.int16)
    wav_memory = BytesIO()
    write(wav_memory, sample_rate, audio_data_pcm)
    wav_memory.seek(0)
    return wav_memory


def save_to_file(audio_data: np.ndarray, file_path: str, sample_rate=48000):
    audio_data_pcm = (audio_data * 32767).astype(np.int16)
    write(file_path, sample_rate, audio_data_pcm)


def play_audio_from_memory(wav_memory: BytesIO):
    sample_rate, audio_data = read(wav_memory)
    sd.play(audio_data, samplerate=sample_rate)
    sd.wait()
