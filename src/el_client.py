from io import BytesIO
import os
import time
import numpy as np
import sounddevice as sd
import colorama
from scipy.signal import resample_poly
from elevenlabs.client import ElevenLabs


class ElevenLabsClient:
    def __init__(self, api_key, voice_id, output_device=None, output_sample_rate=48000, api_sample_rate=22050):
        self.client = ElevenLabs(api_key=api_key)
        self.voice_id = voice_id
        self.output_device = output_device
        self.output_sample_rate = output_sample_rate
        self.api_sample_rate = api_sample_rate

    @classmethod
    def from_env(cls):
        output_device = os.getenv("OUTPUT_DEVICE", None)
        if output_device is not None:
            try:
                output_device = int(output_device)
            except ValueError:
                output_device = None

        output_device_name = os.getenv("OUTPUT_DEVICE_NAME", None)
        if output_device is None and output_device_name:
            name_lower = output_device_name.lower()
            for i, device in enumerate(sd.query_devices()):
                if name_lower in device["name"].lower() and device["max_output_channels"] > 0:
                    output_device = i
                    break

        return cls(
            os.getenv("API_KEY", None),
            os.getenv("VOICE_ID", None),
            output_device=output_device,
            output_sample_rate=int(os.getenv("OUTPUT_SAMPLE_RATE", 48000)),
            api_sample_rate=int(os.getenv("API_SAMPLE_RATE", 22050)),
        )

    def _resample(self, samples: np.ndarray, orig_rate: int, target_rate: int) -> np.ndarray:
        if orig_rate == target_rate:
            return samples
        if samples.size == 0:
            return samples
        return resample_poly(samples, target_rate, orig_rate).astype(np.float32)

    def convert_audio(self, audio: BytesIO, remove_background_noise: bool):
        start_time = time.time()
        audio_stream = self.client.speech_to_speech.convert_as_stream(
            voice_id=self.voice_id,
            audio=audio,
            output_format="pcm_22050",
            remove_background_noise=remove_background_noise,
        )

        stream = None
        first_chunk_time = None
        try:
            stream = sd.OutputStream(
                samplerate=self.output_sample_rate,
                channels=1,
                dtype="float32",
                device=self.output_device
            )
            stream.start()

            for chunk in audio_stream:
                if not chunk:
                    continue
                if first_chunk_time is None:
                    first_chunk_time = time.time()
                    latency_ms = (first_chunk_time - start_time) * 1000
                    print(f"{colorama.Fore.CYAN}First chunk in {latency_ms:.0f}ms{colorama.Style.RESET_ALL}")

                samples = np.frombuffer(chunk, dtype=np.int16).astype(np.float32) / 32768.0
                if samples.size == 0:
                    continue
                samples = self._resample(samples, self.api_sample_rate, self.output_sample_rate)
                stream.write(samples.reshape(-1, 1))
        finally:
            if stream is not None:
                stream.stop()
                stream.close()
