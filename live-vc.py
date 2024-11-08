from dotenv import load_dotenv
from src.audio import *
    
    
def run():
    duration = 5
    audio_data = record_audio(duration)
    audio_data = enhance_audio_quality(audio_data)
    wav_memory = save_to_memory(audio_data)
    play_audio_from_memory(wav_memory)


if __name__ == "__main__":
    load_dotenv("bot.env")
    run()
