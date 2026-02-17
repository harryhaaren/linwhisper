use std::path::PathBuf;

pub struct Config {
    #[cfg(feature = "api")]
    pub api_base_url: String,
    #[cfg(feature = "api")]
    pub api_key: Option<String>,
    #[cfg(feature = "api")]
    pub api_model: String,
    pub whisper_model_path: PathBuf,
}

impl Config {
    pub fn load() -> Self {
        let data_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("linwhisper");
        
        let models_dir = data_dir.join("models");
        std::fs::create_dir_all(&models_dir).ok();
        
        let model_name =
            std::env::var("WHISPER_MODEL").unwrap_or_else(|_| "ggml-base.en.bin".into());
        let whisper_model_path = models_dir.join(&model_name);

        if !whisper_model_path.exists() {
            eprintln!("ERROR: Whisper model not found at {}", whisper_model_path.display());
            eprintln!("Download it with:");
            eprintln!(
                "  mkdir -p {} && curl -L -o {} https://huggingface.co/ggerganov/whisper.cpp/resolve/main/{}",
                models_dir.display(),
                whisper_model_path.display(),
                model_name,
            );
            std::process::exit(1);
        }

        #[cfg(feature = "api")]
        {
            let api_base_url = std::env::var("API_BASE_URL")
                .unwrap_or_else(|_| "https://api.groq.com/openai/v1".into());
            let api_key = std::env::var("API_KEY").ok();
            let api_model = std::env::var("API_MODEL")
                .unwrap_or_else(|_| "whisper-large-v3-turbo".into());
            
            Self {
                api_base_url,
                api_key,
                api_model,
                whisper_model_path,
            }
        }

        #[cfg(not(feature = "api"))]
        Self {
            whisper_model_path,
        }
    }
}
