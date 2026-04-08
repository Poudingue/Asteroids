use chrono::Local;
use std::path::PathBuf;

pub struct VideoCapture {
    session_dir: PathBuf,
    frame_count: u32,
    active: bool,
}

pub fn screenshot_path() -> PathBuf {
    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    PathBuf::from(format!("screenshots/asteroids_{}.png", timestamp))
}

pub fn capture_session_dir() -> PathBuf {
    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    PathBuf::from(format!("captures/session_{}", timestamp))
}

fn frame_path(session_dir: &std::path::Path, frame: u32) -> PathBuf {
    session_dir.join(format!("frame_{:05}.png", frame))
}

impl VideoCapture {
    pub fn new() -> Self {
        Self {
            session_dir: PathBuf::new(),
            frame_count: 0,
            active: false,
        }
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn start(&mut self) {
        self.session_dir = capture_session_dir();
        std::fs::create_dir_all(&self.session_dir).expect("Failed to create capture directory");
        self.frame_count = 0;
        self.active = true;
    }

    pub fn stop(&mut self) {
        self.active = false;
    }

    pub fn toggle(&mut self) {
        if self.active {
            self.stop();
        } else {
            self.start();
        }
    }

    pub fn next_frame_path(&mut self) -> PathBuf {
        let path = frame_path(&self.session_dir, self.frame_count);
        self.frame_count += 1;
        path
    }
}

impl Default for VideoCapture {
    fn default() -> Self {
        Self::new()
    }
}

pub fn save_png(path: &std::path::Path, data: &[u8], width: u32, height: u32) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("Failed to create screenshot directory");
    }
    let img = image::RgbaImage::from_raw(width, height, data.to_vec())
        .expect("Failed to create image from pixel data");
    img.save(path).expect("Failed to save PNG");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn screenshot_path_has_png_extension() {
        let path = screenshot_path();
        assert_eq!(path.extension().unwrap(), "png");
        assert!(path.to_str().unwrap().starts_with("screenshots/asteroids_"));
    }

    #[test]
    fn video_capture_starts_inactive() {
        let vc = VideoCapture::new();
        assert!(!vc.is_active());
    }

    #[test]
    fn video_capture_toggle() {
        let mut vc = VideoCapture::new();
        vc.start();
        assert!(vc.is_active());
        vc.stop();
        assert!(!vc.is_active());
    }

    #[test]
    fn frame_paths_increment() {
        let dir = std::path::PathBuf::from("/tmp/test_session");
        assert_eq!(frame_path(&dir, 0), dir.join("frame_00000.png"));
        assert_eq!(frame_path(&dir, 42), dir.join("frame_00042.png"));
    }

    #[test]
    fn save_png_creates_file() {
        let dir = std::env::temp_dir().join("claude_test_capture");
        let path = dir.join("test.png");
        let data = vec![255u8; 4 * 2 * 2]; // 2x2 white RGBA
        save_png(&path, &data, 2, 2);
        assert!(path.exists());
        std::fs::remove_dir_all(&dir).ok();
    }
}
