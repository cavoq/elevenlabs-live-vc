FROM python:3.13-slim

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

RUN pip install --upgrade pip \
    && pip install uv

COPY pyproject.toml .

RUN uv sync --no-dev

COPY . .

CMD ["uv", "run", "python", "live-vc.py"]
