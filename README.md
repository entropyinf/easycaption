# EasyCaption

EasyCaption is a real-time speech-to-text desktop application designed for macOS. It captures system audio in real-time and converts it to text subtitles displayed on screen.

## Features

- Real-time speech recognition: Using SenseVoiceSmall model for high-accuracy speech-to-text conversion
- System-wide audio capture: Capable of capturing audio output from any application
- Real-time subtitle display: Shows recognized text as an overlay
- Lightweight design: Built with Tauri for minimal resource usage
- Multi-language support: Supports speech recognition in multiple languages

## Tech Stack

- Frontend: React + TypeScript + TailwindCSS
- Backend: Rust + Tauri
- Audio Processing: CPAL (Cross-Platform Audio Input Library)
- Machine Learning: Candle (Rust ML Framework)
- Speech Recognition Model: SenseVoiceSmall

## Usage

1. On first run, the program will automatically download the required speech recognition model files
2. Select the audio device to monitor in the settings page
3. Switch to the caption page to start displaying real-time speech captions

## Development

This project uses a Rust + Tauri architecture, divided into three main parts:

1. `frontend/` - Frontend user interface
2. `backend/` - Tauri backend logic
3. `enthalpy/` - Core audio processing and machine learning library

### Building

```bash
# Development mode
pnpm tauri dev

# Production build
pnpm tauri build
```

## License

[TBD]