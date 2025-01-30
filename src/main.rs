use std::fs;
use std::process::{Command, Stdio};
use std::env;
use std::thread::sleep;
use std::time::Duration;



const PID_FILE: &str = "/tmp/recordingpid";

// Generate timestamped filenames
fn get_timestamped_filename(prefix: &str, extension: &str) -> String {
    let home_dir = env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let timestamp = chrono::Local::now().format("%y%m%d-%H%M-%S").to_string();
    format!("{}/{}-{}.{}", home_dir, prefix, timestamp, extension)
}

// Get screen size from `xdpyinfo`
fn get_screen_size() -> Option<String> {
    let output = Command::new("sh")
        .arg("-c")
        .arg("xdpyinfo | awk '/dimensions/ {print $2;}'")
        .output()
        .ok()?;
    let screen_size = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if screen_size.is_empty() { None } else { Some(screen_size) }
}

// Record full screen
fn record_screen() {
    let screen_size = match get_screen_size() {
        Some(size) => size,
        None => {
            eprintln!("Failed to get screen dimensions.");
            return;
        }
    };

    let output_file = get_timestamped_filename("video", "mkv");
    let child = Command::new("ffmpeg")
        .args(&[
            "-f", "x11grab", "-s", &screen_size, "-i", ":0",
            "-f", "pulse", "-i", "default", // Audio input from PulseAudio
            "-c:v", "libx264", "-qp", "0", "-r", "30",
            "-c:a", "aac", "-b:a", "192k", // Encode audio with AAC
            &output_file,
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start ffmpeg");

    let _ = fs::write(PID_FILE, child.id().to_string());
}



// Record audio only
fn record_audio() {
    let output_file = get_timestamped_filename("audio", "flac");
    let child = Command::new("ffmpeg")
        .args(&["-f", "pulse", "-i", "default", "-c:a", "flac", &output_file])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start ffmpeg");

    let _ = fs::write(PID_FILE, child.id().to_string());
}


fn record_webcam(resolution: Option<&str>) {
    let video_size = resolution.unwrap_or("640x480");
    let output_file = get_timestamped_filename("webcam", "mkv");

    // Add PulseAudio input for audio along with the webcam video input
    let child = Command::new("ffmpeg")
        .args(&[
            "-f", "v4l2",              // Video capture using v4l2 (webcam)
            "-video_size", video_size, // Video resolution (default to 640x480)
            "-i", "/dev/video0",       // Input video device (webcam)
            "-f", "pulse",             // Audio capture using PulseAudio
            "-i", "default",           // Default PulseAudio input device
            "-c:v", "libx264",         // Video codec (libx264 for better compression)
            "-c:a", "aac",             // Audio codec (AAC for compatibility)
            "-preset", "ultrafast",    // Encoding preset (ultrafast for low latency)
            "-crf", "23",              // Constant rate factor (lower values = better quality)
            &output_file,              // Output file path
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null()) // Silence output
        .spawn()
        .expect("Failed to start ffmpeg");

    let _ = fs::write(PID_FILE, child.id().to_string());
}



fn record_selected_area() {
    let output = Command::new("sh")
        .arg("-c")
        .arg("slop -f \"%x %y %w %h\"")
        .output()
        .expect("Failed to get selected area")
        .stdout;

    let selection = String::from_utf8_lossy(&output).trim().to_string();
    let parts: Vec<&str> = selection.split_whitespace().collect();
    if parts.len() != 4 {
        eprintln!("Invalid selection: {}", selection);
        return;
    }

    let x = parts[0];
    let y = parts[1];
    let w = parts[2];
    let h = parts[3];
    let output_file = get_timestamped_filename("box", "mkv");

    let child = Command::new("ffmpeg")
        .args(&[
            "-f", "x11grab", "-framerate", "60",
            "-video_size", &format!("{}x{}", w, h),
            "-i", &format!(":0.0+{},{}", x, y),
            "-f", "pulse", "-i", "default", // Audio input from PulseAudio
            "-c:v", "libx264", "-qp", "0", "-r", "30",
            "-c:a", "aac", "-b:a", "192k", // Encode audio with AAC
            &output_file,
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start ffmpeg");

    let _ = fs::write(PID_FILE, child.id().to_string());
}




// Prompt user for recording mode using dmenu
fn prompt_recording() {
    let output = Command::new("sh")
        .arg("-c")
        .arg("echo -e \"screencast\\nvideo\\nvideo selected\\naudio\\nwebcam\\nwebcam (hi-def)\" | dmenu -i -p \"Select recording mode:\"")
        .output()
        .expect("Failed to run dmenu");

    let choice = String::from_utf8_lossy(&output.stdout).trim().to_string();

    match choice.as_str() {
        "screencast" => record_screen(),
        "video" => record_screen(),
        "video selected" => record_selected_area(),
        "audio" => record_audio(),
        "webcam" => record_webcam(None),
        "webcam (hi-def)" => record_webcam(Some("1920x1080")),
        _ => eprintln!("Invalid choice"),
    }
}

// Check if a recording is active and prompt to stop it
fn check_existing_recording() -> bool {
    // if let Ok(pid) = fs::read_to_string(PID_FILE) {
    if fs::read_to_string(PID_FILE).is_ok() {
        let output = Command::new("sh")
            .arg("-c")
            .arg("echo -e \"Yes\\nNo\" | dmenu -i -p \"Recording active. Stop?\"")
            .output()
            .expect("Failed to run dmenu");

        if String::from_utf8_lossy(&output.stdout).trim() == "Yes" {
            terminate_recording();
            return true;
        }
    }
    false
}

// Terminate active recording
fn terminate_recording() {
    if let Ok(pid) = fs::read_to_string(PID_FILE) {
        let _ = Command::new("kill").arg("-15").arg(pid.trim()).status();
        let _ = fs::remove_file(PID_FILE);
        sleep(Duration::from_secs(3));
        let _ = Command::new("kill").arg("-9").arg(pid.trim()).status();
    }
}

// Main function
fn main() {
    let args: Vec<String> = env::args().collect();

    if check_existing_recording() {
        return;
    }

    match args.get(1).map(|s| s.as_str()) {
        // Some("screencast") => record_screen(),
        Some("video") => record_screen(),
        Some("video selected") => record_selected_area(),
        Some("audio") => record_audio(),
        Some("webcam") => record_webcam(None),
        Some("webcam (hi-def)") => record_webcam(Some("1920x1080")),
        Some("kill") => terminate_recording(),
        _ => prompt_recording(),
    }
}
