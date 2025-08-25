# EasyCaption

EasyCaption is an offline real-time transcription application that leverages the power of OpenAI's Whisper model for audio transcription with a Tauri-based desktop interface.

## Project Overview

EasyCaption provides a seamless offline solution for real-time audio transcription using the Whisper model. Built with a modern technology stack, it offers a native desktop experience while maintaining complete offline functionality for privacy-focused transcription needs.

## Architecture

The application follows a client-server architecture pattern with clear separation of concerns:

### Frontend Layer
- **Framework**: React with TypeScript
- **Build Tool**: Vite for fast development and building
- **UI Components**: Modern React-based interface
- **State Management**: Integrated React state management

### Backend Layer
- **Desktop Framework**: Tauri (Rust-based)
- **Audio Processing**: Integration with Whisper model
- **Transcription Engine**: OpenAI Whisper for offline speech recognition
- **System Integration**: Native OS capabilities through Tauri

### Data Flow
1. Audio input captured through system APIs
2. Audio processing and preparation via Tauri backend
3. Real-time transcription using Whisper model
4. Results displayed in React frontend interface
5. Optional export functionality for transcripts

## Key Features

- **Offline Operation**: Complete offline functionality with no external dependencies after installation
- **Real-time Transcription**: Live audio transcription with minimal latency
- **Privacy Focused**: All processing happens locally on the user's machine
- **Cross-platform**: Native desktop applications for Windows, macOS, and Linux
- **Modern UI**: Responsive and intuitive user interface built with React

## Development Stack

- **Frontend**: React + TypeScript + Vite
- **Backend**: Tauri (Rust)
- **AI Model**: OpenAI Whisper
- **Build System**: Vite with Tauri integration
- **Package Management**: npm/yarn for frontend, cargo for Rust components

## Recommended Development Environment

- [VS Code](https://code.visualstudio.com/) as primary IDE
- [Tauri VS Code Extension](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) for Tauri development support
- [Rust Analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer) for Rust code intelligence

## Getting Started

This template provides a solid foundation for building a Whisper-powered transcription application with Tauri, React, and TypeScript. The modular architecture allows for easy extension and customization while maintaining performance and offline capabilities.