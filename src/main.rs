mod audio;
mod config;
mod dsp;
mod elevenlabs;

use std::{
    env,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};

use anyhow::{Context, Result, bail};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};

use crate::{
    audio::{AudioPlayer, AudioRecorder},
    config::{Config, Mode},
    elevenlabs::{ElevenLabsClient, prepare_payload},
};

fn main() {
    if let Err(error) = run() {
        eprintln!("\nError: {error:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let arguments: Vec<String> = env::args().skip(1).collect();
    if arguments
        .iter()
        .any(|argument| argument == "--list-devices")
    {
        return audio::list_devices();
    }
    if arguments
        .iter()
        .any(|argument| argument == "--help" || argument == "-h")
    {
        print_help();
        return Ok(());
    }
    let config_path = config_path(&arguments)?;
    let config = Config::load(&config_path).with_context(|| {
        format!(
            "create config.toml from config.example.toml, then retry (looked at {})",
            config_path.display()
        )
    })?;

    println!("ElevenLabs Live VC v{}", env!("CARGO_PKG_VERSION"));
    println!("Config: {}", config_path.display());
    let recorder = AudioRecorder::new(&config)?;
    let player = AudioPlayer::new(&config)?;
    let client = ElevenLabsClient::new(&config)?;
    let running = Arc::new(AtomicBool::new(true));
    let signal = Arc::clone(&running);
    ctrlc::set_handler(move || signal.store(false, Ordering::SeqCst))?;

    match config.mode {
        Mode::Manual => run_manual(&config, &recorder, &player, &client, &running),
        Mode::Automatic => run_automatic(&config, &recorder, &player, &client, &running),
    }
}

fn run_manual(
    config: &Config,
    recorder: &AudioRecorder,
    player: &AudioPlayer,
    client: &ElevenLabsClient,
    running: &AtomicBool,
) -> Result<()> {
    let _terminal = RawTerminal::enable()?;
    println!("Manual mode: SPACE starts/stops recording; Q or Ctrl+C exits.");
    let mut recording = false;

    while running.load(Ordering::SeqCst) {
        if !event::poll(Duration::from_millis(50))? {
            continue;
        }
        let Event::Key(key) = event::read()? else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }
        match key.code {
            KeyCode::Char('q') | KeyCode::Char('Q') => break,
            KeyCode::Char(' ') if !recording => {
                recorder.start_manual();
                recording = true;
                println!("Recording...");
            }
            KeyCode::Char(' ') => {
                recording = false;
                println!("Processing...");
                process_recording(config, recorder, player, client, recorder.stop_manual())?;
                println!("Ready.");
            }
            _ => {}
        }
    }
    Ok(())
}

fn run_automatic(
    config: &Config,
    recorder: &AudioRecorder,
    player: &AudioPlayer,
    client: &ElevenLabsClient,
    running: &AtomicBool,
) -> Result<()> {
    println!("Automatic VAD mode: speak naturally; Ctrl+C exits.");
    println!("Listening...");
    while running.load(Ordering::SeqCst) {
        if let Some(samples) = recorder.poll_automatic() {
            println!("Silence detected, processing...");
            process_recording(config, recorder, player, client, samples)?;
            println!("Listening...");
        }
        thread::sleep(Duration::from_millis(20));
    }
    Ok(())
}

fn process_recording(
    config: &Config,
    recorder: &AudioRecorder,
    player: &AudioPlayer,
    client: &ElevenLabsClient,
    samples: Vec<f32>,
) -> Result<()> {
    recorder.set_enabled(false);
    let result = (|| {
        let trimmed = dsp::trim_with_padding(
            &samples,
            recorder.sample_rate(),
            config.trim_threshold,
            config.trim_padding_ms,
        );
        let minimum = (u64::from(recorder.sample_rate()) * config.min_audio_ms / 1000) as usize;
        if trimmed.len() < minimum {
            println!("No usable audio recorded; try speaking longer.");
            return Ok(());
        }
        let payload = prepare_payload(&trimmed, recorder.sample_rate(), config.low_latency_input)?;
        client.convert_and_play(payload, player)
    })();
    recorder.set_enabled(true);
    result
}

fn config_path(arguments: &[String]) -> Result<PathBuf> {
    if arguments.len() > 1 {
        bail!("expected at most one config path; use --help for usage");
    }
    if let Some(path) = arguments.first() {
        return Ok(PathBuf::from(path));
    }
    let local = PathBuf::from("config.toml");
    if local.exists() {
        return Ok(local);
    }
    let executable = env::current_exe().context("cannot locate executable")?;
    Ok(executable
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("config.toml"))
}

fn print_help() {
    println!(
        "elevenlabs-live-vc {}\n\nUSAGE:\n    elevenlabs-live-vc [config-path]\n    elevenlabs-live-vc --list-devices\n\nThe default config is config.toml beside the executable or in the current directory.",
        env!("CARGO_PKG_VERSION")
    );
}

struct RawTerminal;

impl RawTerminal {
    fn enable() -> Result<Self> {
        crossterm::terminal::enable_raw_mode()?;
        Ok(Self)
    }
}

impl Drop for RawTerminal {
    fn drop(&mut self) {
        let _ = crossterm::terminal::disable_raw_mode();
    }
}
