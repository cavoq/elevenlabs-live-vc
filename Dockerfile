FROM python:3.12.6-slim

ENV PYTHONDONTWRITEBYTECODE 1
ENV PYTHONUNBUFFERED 1

WORKDIR /elevenlabs-live-vc

RUN apt-get update && apt-get install -y --no-install-recommends \
    ffmpeg \
    kmod kbd \
    libportaudio2 \
    portaudio19-dev \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

COPY requirements.txt .

RUN pip install --upgrade pip \
    && pip install -r requirements.txt

COPY . .

CMD ["python", "live-vc.py"]