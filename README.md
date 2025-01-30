# Rust_Capture
Rust Capture is a fast, lightweight, and efficient screen, audio, and webcam recording tool for Linux, written in Rust. It leverages ffmpeg, xdpyinfo, and slop to provide seamless recording capabilities with minimal resource usage. Perfect for tutorials, streaming, and content creation. 🚀🎥🎙️

## Dependencies

To use this tool, ensure the following dependencies are installed:

    Rust & Cargo: Required for compiling and running the project.
    FFmpeg: Handles video and audio encoding.
    xdpyinfo: Retrieves screen resolution.
    slop: Allows region selection for recording.
    dmenu: Provides an interactive recording mode selection.
    PulseAudio: Used for audio capture.

## Cargo Recording Commands

- **`cargo run -- screencast`** → Starts screen recording.
- **`cargo run -- video`** → Same as screencast (full-screen recording).
- **`cargo run -- "video selected"`** → Lets you select an area to record or a specific window.
- **`cargo run -- audio`** → Records audio only.
- **`cargo run -- webcam`** → Records webcam video.
- **`cargo run -- "webcam (hi-def)"`** → Records webcam in 1080p.
- **`cargo run -- kill`** → Stops the active recording.
