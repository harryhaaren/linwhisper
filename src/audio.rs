use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::io::Cursor;
use std::sync::{Arc, Mutex};

fn list_available_devices() -> String {
    let mut output = String::new();
    let host = cpal::default_host();

    output.push_str("\nAvailable input devices:\n");
    match host.input_devices() {
        Ok(devices) => {
            let mut count = 0;
            for (idx, device) in devices.enumerate() {
                if let Ok(name) = device.name() {
                    output.push_str(&format!("  {}. {}\n", idx + 1, name));
                    count += 1;
                }
            }
            if count == 0 {
                output.push_str("  (No input devices found)\n");
            }
        }
        Err(e) => {
            output.push_str(&format!("  Error listing devices: {}\n", e));
        }
    }

    output.push_str("\nTroubleshooting:\n");
    output.push_str("  - Check if your microphone is plugged in\n");
    output.push_str("  - Check if the microphone is enabled in your system settings\n");
    output.push_str("  - Try running: arecord -l (to list ALSA devices)\n");
    output.push_str("  - Check microphone permissions for this application\n");

    output
}

pub struct Recorder {
    samples: Arc<Mutex<Vec<f32>>>,
    stream: Option<cpal::Stream>,
    sample_rate: u32,
    channels: u16,
    _device_name: String,
}

impl Recorder {
    pub fn new() -> Result<Self, String> {
        let host = cpal::default_host();

        // Try to find a working device
        let (_device, config, device_name) = Self::find_working_device_static(&host)?;

        let sample_rate = config.sample_rate().0;
        let channels = config.channels();

        eprintln!(
            "Using audio device: {} ({}Hz, {} channels)",
            device_name, sample_rate, channels
        );

        Ok(Self {
            samples: Arc::new(Mutex::new(Vec::new())),
            stream: None,
            sample_rate,
            channels,
            _device_name: device_name,
        })
    }

    pub fn start(&mut self) -> Result<(), String> {
        let host = cpal::default_host();

        // Try to get a working device
        let (device, config) = self.find_working_device(&host)?;

        let samples = Arc::clone(&self.samples);
        samples.lock().unwrap().clear();

        let err_fn = |err| eprintln!("Audio stream error: {err}");

        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => {
                let samples = Arc::clone(&samples);
                device
                    .build_input_stream(
                        &config.into(),
                        move |data: &[f32], _: &_| {
                            samples.lock().unwrap().extend_from_slice(data);
                        },
                        err_fn,
                        None,
                    )
                    .map_err(|e| format!("Failed to build stream: {e}"))?
            }
            cpal::SampleFormat::I16 => {
                let samples = Arc::clone(&samples);
                device
                    .build_input_stream(
                        &config.into(),
                        move |data: &[i16], _: &_| {
                            let floats: Vec<f32> =
                                data.iter().map(|&s| s as f32 / i16::MAX as f32).collect();
                            samples.lock().unwrap().extend_from_slice(&floats);
                        },
                        err_fn,
                        None,
                    )
                    .map_err(|e| format!("Failed to build stream: {e}"))?
            }
            cpal::SampleFormat::U16 => {
                let samples = Arc::clone(&samples);
                device
                    .build_input_stream(
                        &config.into(),
                        move |data: &[u16], _: &_| {
                            let floats: Vec<f32> = data
                                .iter()
                                .map(|&s| (s as f32 / u16::MAX as f32) * 2.0 - 1.0)
                                .collect();
                            samples.lock().unwrap().extend_from_slice(&floats);
                        },
                        err_fn,
                        None,
                    )
                    .map_err(|e| format!("Failed to build stream: {e}"))?
            }
            fmt => return Err(format!("Unsupported sample format: {fmt:?}")),
        };

        stream.play().map_err(|e| format!("Failed to play: {e}"))?;
        self.stream = Some(stream);
        Ok(())
    }

    pub fn stop(&mut self) -> Result<Vec<u8>, String> {
        // Drop the stream to stop recording
        self.stream.take();

        let samples = self.samples.lock().unwrap();
        if samples.is_empty() {
            return Err("No audio recorded".into());
        }

        // Convert to mono if multi-channel
        let mono: Vec<f32> = if self.channels > 1 {
            samples
                .chunks(self.channels as usize)
                .map(|chunk| chunk.iter().sum::<f32>() / chunk.len() as f32)
                .collect()
        } else {
            samples.clone()
        };

        // Encode as WAV
        let mut buf = Cursor::new(Vec::new());
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: self.sample_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut writer =
            hound::WavWriter::new(&mut buf, spec).map_err(|e| format!("WAV write error: {e}"))?;

        for &sample in &mono {
            let s = (sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
            writer
                .write_sample(s)
                .map_err(|e| format!("WAV sample error: {e}"))?;
        }
        writer
            .finalize()
            .map_err(|e| format!("WAV finalize error: {e}"))?;

        Ok(buf.into_inner())
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn find_working_device(
        &self,
        host: &cpal::Host,
    ) -> Result<(cpal::Device, cpal::SupportedStreamConfig), String> {
        Self::find_working_device_static(host).map(|(dev, cfg, _name)| (dev, cfg))
    }

    fn find_working_device_static(
        host: &cpal::Host,
    ) -> Result<(cpal::Device, cpal::SupportedStreamConfig, String), String> {
        // First try default device
        if let Some(device) = host.default_input_device() {
            if let Ok(config) = device.default_input_config() {
                let name = device.name().unwrap_or_else(|_| "default".to_string());
                return Ok((device, config, name));
            } else {
                eprintln!("Warning: Default device configuration failed, trying other devices...");
            }
        }

        // Try all available devices
        let devices = host
            .input_devices()
            .map_err(|e| format!("Failed to enumerate devices: {}", e))?;

        for device in devices {
            if let Ok(config) = device.default_input_config() {
                let name = device.name().unwrap_or_else(|_| "unknown".to_string());
                eprintln!("Found working input device: {}", name);
                return Ok((device, config, name));
            }
        }

        Err(format!(
            "No working input device found{}",
            list_available_devices()
        ))
    }
}
