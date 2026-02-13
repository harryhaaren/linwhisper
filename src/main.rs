#[cfg(feature = "api")]
mod api;
mod audio;
mod config;
mod local_stt;

use std::io::{self, Read};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::thread;

use audio::Recorder;
use config::Config;
use local_stt::LocalWhisper;

fn main() {
    println!("LinWhisper - Minimal CLI Transcriber");
    println!("====================================");
    
    // Load config
    let config = Config::load();
    
    // Initialize local whisper
    println!("Loading Whisper model...");
    let whisper = Arc::new(
        LocalWhisper::new(&config.whisper_model_path)
            .expect("Failed to load whisper model")
    );
    println!("Model loaded successfully!");
    
    // Create recorder
    let mut recorder = Recorder::new().expect("Failed to initialize audio recorder");
    
    println!("\nPress Enter or Space to stop recording...");
    println!("Starting recording NOW...\n");
    
    // Start recording
    if let Err(e) = recorder.start() {
        eprintln!("Failed to start recording: {e}");
        std::process::exit(1);
    }
    
    // Set up signal to stop recording
    let stop_signal = Arc::new(AtomicBool::new(false));
    let stop_signal_clone = Arc::clone(&stop_signal);
    
    // Spawn thread to listen for Enter or Space key
    thread::spawn(move || {
        let stdin = io::stdin();
        let mut buffer = [0u8; 1];
        
        loop {
            if let Ok(_) = stdin.lock().read(&mut buffer) {
                // Any key press (Enter, Space, or any other key) will stop
                if buffer[0] == b'\n' || buffer[0] == b' ' || buffer[0] != 0 {
                    stop_signal_clone.store(true, Ordering::Relaxed);
                    break;
                }
            }
        }
    });
    
    // Wait for stop signal
    while !stop_signal.load(Ordering::Relaxed) {
        thread::sleep(std::time::Duration::from_millis(100));
    }
    
    println!("Stopping recording...");
    
    // Stop recording and get audio data
    let wav_data = match recorder.stop() {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Failed to stop recording: {e}");
            std::process::exit(1);
        }
    };
    
    println!("Transcribing...");
    
    // Transcribe the audio
    let sample_rate = recorder.sample_rate();
    
    #[cfg(feature = "api")]
    let text = {
        if let Some(api_key) = &config.api_key {
            println!("Using API transcription...");
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(api::transcribe(
                &config.api_base_url,
                api_key,
                &config.api_model,
                wav_data,
            ))
        } else {
            whisper.transcribe(&wav_data, sample_rate)
        }
    };
    
    #[cfg(not(feature = "api"))]
    let text = whisper.transcribe(&wav_data, sample_rate);
    
    match text {
        Ok(transcription) => {
            println!("\n========================================");
            println!("Transcription:");
            println!("========================================");
            println!("{}", transcription);
            println!("========================================\n");
        }
        Err(e) => {
            eprintln!("Transcription failed: {e}");
            std::process::exit(1);
        }
    }
}
