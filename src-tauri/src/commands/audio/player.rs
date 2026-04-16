use std::sync::mpsc;

use rodio::Source;
use std::io::Read;
use tauri::Emitter;

/// A custom rodio Source that reads raw PCM s16le data from a pipe (FFmpeg stdout).
struct PcmSource<R: Read> {
    reader: R,
    sample_rate: u32,
    channels: u16,
    // Two-sample buffer for stereo interleaving
    pending: Option<i16>,
}

impl<R: Read> PcmSource<R> {
    fn new(reader: R, sample_rate: u32, channels: u16) -> Self {
        Self {
            reader,
            sample_rate,
            channels,
            pending: None,
        }
    }
}

impl<R: Read> Iterator for PcmSource<R> {
    type Item = i16;

    fn next(&mut self) -> Option<i16> {
        if self.channels == 1 {
            let mut buf = [0u8; 2];
            match self.reader.read_exact(&mut buf) {
                Ok(()) => Some(i16::from_le_bytes(buf)),
                Err(_) => None,
            }
        } else {
            // Stereo: interleave left/right samples
            if let Some(sample) = self.pending.take() {
                return Some(sample);
            }
            let mut buf = [0u8; 2];
            match self.reader.read_exact(&mut buf) {
                Ok(()) => {
                    let left = i16::from_le_bytes(buf);
                    // Read right channel
                    match self.reader.read_exact(&mut buf) {
                        Ok(()) => {
                            let right = i16::from_le_bytes(buf);
                            self.pending = Some(right);
                            Some(left)
                        }
                        Err(_) => Some(left),
                    }
                }
                Err(_) => None,
            }
        }
    }
}

impl<R: Read> Source for PcmSource<R> {
    fn current_frame_len(&self) -> Option<usize> {
        // Unknown length for a pipe
        None
    }

    fn channels(&self) -> u16 {
        self.channels
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        None
    }
}

enum AudioCommand {
    Play(String, mpsc::Sender<Result<(), String>>, Option<tauri::AppHandle>),
    Stop,
    SetVolume(f32),
    Shutdown,
}

pub struct AudioPlayer {
    tx: mpsc::Sender<AudioCommand>,
}

/// SAFETY: AudioPlayer only contains an `mpsc::Sender<AudioCommand>`, which is inherently
/// thread-safe (Sync + Send). The sender is a reference-counted channel handle that can
/// be safely shared across threads. All mutation happens on the dedicated background
/// thread that owns the receiver.
unsafe impl Sync for AudioPlayer {}

impl AudioPlayer {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel::<AudioCommand>();

        std::thread::spawn(move || {
            let mut current_sink: Option<std::sync::Arc<rodio::Sink>> = None;
            let mut current_stream: Option<rodio::OutputStream> = None;
            let mut current_child: Option<std::process::Child> = None;

            while let Ok(cmd) = rx.recv() {
                match cmd {
                    AudioCommand::Play(path, reply, app_handle) => {
                        // Stop any currently playing audio
                        if let Some(sink) = current_sink.take() {
                            sink.stop();
                        }
                        current_stream.take();
                        if let Some(mut child) = current_child.take() {
                            let _ = child.kill();
                        }

                        let result = (|| -> Result<(std::sync::Arc<rodio::Sink>, std::process::Child), String> {
                            let (stream, stream_handle) =
                                rodio::OutputStream::try_default()
                                    .map_err(|e| format!("Failed to open default audio output: {}", e))?;

                            let sink = rodio::Sink::try_new(&stream_handle)
                                .map_err(|e| format!("Failed to create audio sink: {}", e))?;

                            // Use FFmpeg to decode any audio format to raw PCM (16-bit signed LE, 44100Hz, stereo)
                            // then pipe it to rodio for playback. This handles WAV, FLAC, AAC, etc.
                            let ffmpeg_bin = super::ffmpeg::find_ffmpeg();
                            let mut child = std::process::Command::new(&ffmpeg_bin)
                                .args([
                                    "-i", &path,
                                    "-f", "s16le",
                                    "-acodec", "pcm_s16le",
                                    "-ar", "44100",
                                    "-ac", "2",
                                    "-",
                                ])
                                .stdin(std::process::Stdio::null())
                                .stdout(std::process::Stdio::piped())
                                .stderr(std::process::Stdio::null())
                                .spawn()
                                .map_err(|e| {
                                    if e.kind() == std::io::ErrorKind::NotFound {
                                        "FFmpeg not found".to_string()
                                    } else {
                                        format!("Failed to start FFmpeg: {}", e)
                                    }
                                })?;

                            let stdout = child.stdout.take()
                                .ok_or("FFmpeg has no stdout")?;

                            // Raw PCM source: 44100Hz, 16-bit signed LE, stereo
                            let source = PcmSource::new(std::io::BufReader::new(stdout), 44100, 2);

                            sink.append(source);
                            let sink = std::sync::Arc::new(sink);
                            current_stream = Some(stream);
                            Ok((sink, child))
                        })();

                        match result {
                            Ok((sink, child)) => {
                                current_sink = Some(sink.clone());
                                current_child = Some(child);
                                let _ = reply.send(Ok(()));

                                // Spawn a watcher thread that emits audio-finished when done
                                if let Some(app) = app_handle.clone() {
                                    let path = path.clone();
                                    std::thread::spawn(move || {
                                        sink.sleep_until_end();
                                        let _ = app.emit("audio-finished", &path);
                                    });
                                }
                            }
                            Err(e) => {
                                let _ = reply.send(Err(e));
                            }
                        }
                    }
                    AudioCommand::Stop => {
                        if let Some(sink) = current_sink.take() {
                            sink.stop();
                        }
                        current_stream.take();
                        if let Some(mut child) = current_child.take() {
                            let _ = child.kill();
                        }
                    }
                    AudioCommand::SetVolume(vol) => {
                        if let Some(ref sink) = current_sink {
                            sink.set_volume(vol);
                        }
                    }
                    AudioCommand::Shutdown => break,
                }
            }
        });

        AudioPlayer { tx }
    }

    pub fn play(&self, file_path: &str, app: Option<tauri::AppHandle>) -> Result<(), String> {
        let (reply_tx, reply_rx) = mpsc::channel();
        self.tx
            .send(AudioCommand::Play(file_path.to_string(), reply_tx, app))
            .map_err(|e| format!("Audio thread gone: {}", e))?;
        reply_rx
            .recv()
            .map_err(|e| format!("Audio thread reply failed: {}", e))?
    }

    pub fn stop(&self) {
        let _ = self.tx.send(AudioCommand::Stop);
    }

    pub fn set_volume(&self, volume: f32) {
        let _ = self.tx.send(AudioCommand::SetVolume(volume.clamp(0.0, 1.0)));
    }
}

impl Drop for AudioPlayer {
    fn drop(&mut self) {
        let _ = self.tx.send(AudioCommand::Shutdown);
    }
}

#[tauri::command]
pub fn play_audio(
    app: tauri::AppHandle,
    state: tauri::State<'_, AudioPlayer>,
    file_path: String,
) -> Result<(), crate::core::error::AppError> {
    state.play(&file_path, Some(app)).map_err(crate::core::error::AppError::Audio)
}

#[tauri::command]
pub fn stop_audio(
    state: tauri::State<'_, AudioPlayer>,
) -> Result<(), crate::core::error::AppError> {
    state.stop();
    Ok(())
}

#[tauri::command]
pub fn set_audio_volume(
    state: tauri::State<'_, AudioPlayer>,
    volume: f32,
) -> Result<(), crate::core::error::AppError> {
    state.set_volume(volume);
    Ok(())
}
