import numpy as np
import sounddevice as sd
import threading
import time
import colorama
from collections import deque

from src.settings.audio import AudioSettings


class AudioRecorder:
    def __init__(self, settings: AudioSettings):
        self.settings = settings
        self.is_recording = False
        self.audio_data = []
        self.stream = None

        # VAD settings
        self.vad_enabled = settings.mode == 1
        self.silence_threshold = settings.silence_threshold
        self.silence_duration = settings.vad_silence_duration
        self.min_recording_duration = settings.vad_min_recording_duration
        self.pre_buffer_duration = settings.vad_pre_buffer_duration

        # VAD state
        self.last_voice_time = 0
        self.recording_start_time = 0
        self.vad_callback = None
        self._vad_thread = None
        self._stop_vad = False
        self.voice_detected = False

        self.pre_buffer_chunks = int(self.pre_buffer_duration * settings.sample_rate / 1024) + 5
        self.pre_buffer = deque(maxlen=self.pre_buffer_chunks)

    @classmethod
    def from_env(cls):
        return cls(AudioSettings.from_env())

    def get_audio_data(self) -> np.ndarray:
        return self.audio_data

    def set_vad_callback(self, callback):
        self.vad_callback = callback

    def _calculate_rms(self, audio_chunk):
        return np.sqrt(np.mean(audio_chunk ** 2))

    def callback(self, indata, frames, time_info, status):
        audio_copy = indata.copy()

        if self.vad_enabled:
            rms = self._calculate_rms(audio_copy)

            if not self.voice_detected:
                self.pre_buffer.append(audio_copy)

                if rms > self.silence_threshold:
                    self.voice_detected = True
                    self.is_recording = True
                    self.recording_start_time = time.time()
                    self.last_voice_time = time.time()
                    self.audio_data = list(self.pre_buffer)
                    print(f"\n{colorama.Fore.GREEN}[VAD] Voice detected, recording...{colorama.Style.RESET_ALL}")
            else:
                if self.is_recording:
                    self.audio_data.append(audio_copy)

                    if rms > self.silence_threshold:
                        self.last_voice_time = time.time()
        else:
            if self.is_recording:
                self.audio_data.append(audio_copy)

    def _vad_monitor(self):
        while not self._stop_vad:
            time.sleep(0.1)

            if self._stop_vad or not self.voice_detected or not self.is_recording:
                continue

            current_time = time.time()
            recording_duration = current_time - self.recording_start_time
            silence_time = current_time - self.last_voice_time

            if recording_duration > self.min_recording_duration and silence_time > self.silence_duration:
                print(f"\n{colorama.Fore.YELLOW}[VAD] Silence detected, processing...{colorama.Style.RESET_ALL}")
                self.stop()
                if self.vad_callback:
                    self.vad_callback()
                break

    def start(self):
        if not self.is_recording:
            self.is_recording = True
            self.audio_data = []
            self.recording_start_time = time.time()
            self.last_voice_time = time.time()
            self._stop_vad = False
            self.voice_detected = True

            self.stream = sd.InputStream(
                callback=self.callback,
                channels=self.settings.channels,
                samplerate=self.settings.sample_rate,
                dtype='float32'
            )
            self.stream.start()

            if self.vad_enabled:
                self._vad_thread = threading.Thread(target=self._vad_monitor, daemon=True)
                self._vad_thread.start()

    def stop(self):
        if self.is_recording and self.stream is not None:
            self._stop_vad = True
            self.stream.stop()
            self.stream.close()
            self.is_recording = False
            self.voice_detected = False

    def start_continuous(self):
        if self.vad_enabled:
            self.audio_data = []
            self.pre_buffer.clear()
            self.voice_detected = False
            self.is_recording = False
            self._stop_vad = False

            print(f"{colorama.Fore.GREEN}[VAD] Listening for voice...{colorama.Style.RESET_ALL}")

            self.stream = sd.InputStream(
                callback=self.callback,
                channels=self.settings.channels,
                samplerate=self.settings.sample_rate,
                dtype='float32'
            )
            self.stream.start()

            self._vad_thread = threading.Thread(target=self._vad_monitor, daemon=True)
            self._vad_thread.start()
