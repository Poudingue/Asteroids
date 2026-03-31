use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::path::Path;

/// Header for a .inputs recording file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingHeader {
    pub seed: u64,
    pub target_fps: u32,
    pub frame_count: u64,
}

/// Per-frame input state (dense recording).
/// 4 stick axes (f32 for space efficiency) + button bitfield.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct InputFrame {
    pub left_stick_x: f32,
    pub left_stick_y: f32,
    pub right_stick_x: f32,
    pub right_stick_y: f32,
    pub buttons: u16,
}

// Button bitfield constants
pub const BTN_FIRE: u16 = 1 << 0;
pub const BTN_TELEPORT: u16 = 1 << 1;
pub const BTN_PAUSE: u16 = 1 << 2;
pub const BTN_MOVE_W: u16 = 1 << 3;
pub const BTN_MOVE_A: u16 = 1 << 4;
pub const BTN_MOVE_S: u16 = 1 << 5;
pub const BTN_MOVE_D: u16 = 1 << 6;

impl InputFrame {
    pub fn new() -> Self {
        Self {
            left_stick_x: 0.0,
            left_stick_y: 0.0,
            right_stick_x: 0.0,
            right_stick_y: 0.0,
            buttons: 0,
        }
    }

    pub fn has_button(&self, btn: u16) -> bool {
        self.buttons & btn != 0
    }

    pub fn set_button(&mut self, btn: u16) {
        self.buttons |= btn;
    }
}

impl Default for InputFrame {
    fn default() -> Self {
        Self::new()
    }
}

/// Writer for .inputs files (zstd-compressed bincode).
pub struct InputRecorder {
    frames: Vec<InputFrame>,
    pub header: RecordingHeader,
}

impl InputRecorder {
    pub fn new(seed: u64, target_fps: u32) -> Self {
        Self {
            frames: Vec::new(),
            header: RecordingHeader {
                seed,
                target_fps,
                frame_count: 0,
            },
        }
    }

    pub fn push_frame(&mut self, frame: InputFrame) {
        self.frames.push(frame);
        self.header.frame_count = self.frames.len() as u64;
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), String> {
        let file = std::fs::File::create(path.as_ref())
            .map_err(|e| format!("Failed to create recording file: {}", e))?;
        let mut encoder = zstd::Encoder::new(file, 3)
            .map_err(|e| format!("Failed to create zstd encoder: {}", e))?;

        // Write header
        let header_bytes = bincode::serialize(&self.header)
            .map_err(|e| format!("Failed to serialize header: {}", e))?;
        let header_len = header_bytes.len() as u32;
        encoder
            .write_all(&header_len.to_le_bytes())
            .map_err(|e| format!("Write error: {}", e))?;
        encoder
            .write_all(&header_bytes)
            .map_err(|e| format!("Write error: {}", e))?;

        // Write frames
        let frames_bytes = bincode::serialize(&self.frames)
            .map_err(|e| format!("Failed to serialize frames: {}", e))?;
        encoder
            .write_all(&frames_bytes)
            .map_err(|e| format!("Write error: {}", e))?;

        encoder
            .finish()
            .map_err(|e| format!("Failed to finish zstd stream: {}", e))?;

        Ok(())
    }
}

/// Reader for .inputs files.
pub struct InputPlayback {
    pub header: RecordingHeader,
    pub frames: Vec<InputFrame>,
}

impl InputPlayback {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, String> {
        let file = std::fs::File::open(path.as_ref())
            .map_err(|e| format!("Failed to open recording file: {}", e))?;
        let mut decoder = zstd::Decoder::new(file)
            .map_err(|e| format!("Failed to create zstd decoder: {}", e))?;

        // Read header
        let mut len_buf = [0u8; 4];
        decoder
            .read_exact(&mut len_buf)
            .map_err(|e| format!("Read error: {}", e))?;
        let header_len = u32::from_le_bytes(len_buf) as usize;
        let mut header_buf = vec![0u8; header_len];
        decoder
            .read_exact(&mut header_buf)
            .map_err(|e| format!("Read error: {}", e))?;
        let header: RecordingHeader = bincode::deserialize(&header_buf)
            .map_err(|e| format!("Failed to deserialize header: {}", e))?;

        // Read frames
        let mut frames_buf = Vec::new();
        decoder
            .read_to_end(&mut frames_buf)
            .map_err(|e| format!("Read error: {}", e))?;
        let frames: Vec<InputFrame> = bincode::deserialize(&frames_buf)
            .map_err(|e| format!("Failed to deserialize frames: {}", e))?;

        Ok(Self { header, frames })
    }

    pub fn frame(&self, index: u64) -> Option<&InputFrame> {
        self.frames.get(index as usize)
    }
}
