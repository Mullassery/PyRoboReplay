//! Video processing pipeline for Phase 14
//!
//! Handles video frame extraction, caching, optical flow computation, and
//! lighting analysis. Frame extraction shells out to the system `ffmpeg`/
//! `ffprobe` binaries (no Rust FFmpeg binding dependency) since those are a
//! reasonable, widely-available baseline for real video decode.
//!
//! YOLO object detection and monocular depth estimation genuinely need a
//! pretrained model + inference runtime (e.g. `ort` + an ONNX model) to be
//! real — there wasn't a vetted model source and inference pipeline to wire
//! in here, so rather than keep the previous hardcoded "always detects one
//! person at 95% confidence" placeholder (which looks like real output but
//! isn't), both now return a clear "not implemented" error. A caller can't
//! silently trust fabricated detections; it gets an explicit signal instead.
//!
//! NOTE ON REACHABILITY: as of this change, nothing else in this codebase
//! constructs a `VideoProcessor` — it isn't wired into the mission-replay
//! pipeline (that would go through the mcap/rosbag2 parsing in
//! `modality_adapters.rs`, which is itself still stubbed). This module is
//! now internally real and tested, but not yet reachable end-to-end.

use serde::{Serialize, Deserialize};
use thiserror::Error;
use std::collections::VecDeque;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Error)]
pub enum VideoError {
    #[error("Frame extraction failed: {0}")]
    ExtractionFailed(String),

    #[error("Detection failed: {0}")]
    DetectionFailed(String),

    #[error("Invalid frame index: {0}")]
    InvalidFrameIndex(u32),

    #[error("Cache error: {0}")]
    CacheError(String),

    #[error("Not implemented: {0}")]
    NotImplemented(String),
}

pub type VideoResult<T> = Result<T, VideoError>;

/// Video frame metadata, now carrying the actual decoded pixel buffer
/// (RGB24, row-major, no padding) rather than just metadata about a frame
/// that was never really loaded.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameData {
    pub index: u32,
    pub timestamp_ns: i64,
    pub resolution: (u32, u32),
    pub format: String,
    pub size_bytes: u32,
    #[serde(default)]
    pub pixels: Vec<u8>,
}

/// Object detection result from YOLO or similar
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectDetection {
    pub class_id: u32,
    pub class_name: String,
    pub confidence: f32,
    pub bbox: BoundingBox,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundingBox {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Optical flow frame showing motion vectors, computed via block matching
/// (sum-of-absolute-differences search over a small block grid) between two
/// consecutive frames' real pixel data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpticalFlowFrame {
    pub frame_index: u32,
    pub timestamp_ns: i64,
    pub flow_magnitude: f32,      // Average motion magnitude, in pixels
    pub dominant_direction: f32,  // Angle in radians
}

/// Lighting analysis of frame, computed from a real luminance histogram of
/// the frame's pixel data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightingAnalysis {
    pub frame_index: u32,
    pub luminance: f32,  // Mean luminance, 0-255 scale
    pub contrast: f32,   // Normalized std-dev of luminance, 0-1
    pub brightness: f32, // Same scale as luminance (kept as a separate field for API compatibility)
}

/// Depth estimation from single frame (monocular)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepthEstimate {
    pub frame_index: u32,
    pub depth_map: Vec<f32>,  // Depth in meters
    pub resolution: (u32, u32),
}

// ============================================================================
// Video Processor
// ============================================================================

pub struct VideoProcessor {
    fps: f32,
    frame_cache: FrameCache,
    yolo_enabled: bool,
    optical_flow_enabled: bool,
    lighting_enabled: bool,
    depth_enabled: bool,
    video_path: Option<PathBuf>,
    probed_resolution: Option<(u32, u32)>,
}

impl VideoProcessor {
    pub fn new(fps: f32) -> Self {
        VideoProcessor {
            fps,
            frame_cache: FrameCache::new(30),  // Cache 30 frames
            yolo_enabled: false,
            optical_flow_enabled: false,
            lighting_enabled: false,
            depth_enabled: false,
            video_path: None,
            probed_resolution: None,
        }
    }

    /// Associate a real video file with this processor. Without this,
    /// `get_frame` returns a clear error rather than fabricated metadata.
    pub fn with_video_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.video_path = Some(path.into());
        self
    }

    pub fn enable_yolo(mut self) -> Self {
        self.yolo_enabled = true;
        self
    }

    pub fn enable_optical_flow(mut self) -> Self {
        self.optical_flow_enabled = true;
        self
    }

    pub fn enable_lighting(mut self) -> Self {
        self.lighting_enabled = true;
        self
    }

    pub fn enable_depth(mut self) -> Self {
        self.depth_enabled = true;
        self
    }

    /// Convert frame number to timestamp (ROS nanoseconds, relative)
    pub fn frame_to_timestamp(&self, frame_number: u32) -> i64 {
        (frame_number as f64 / self.fps as f64 * 1e9) as i64
    }

    /// Convert timestamp to frame number
    pub fn timestamp_to_frame(&self, timestamp_ns: i64) -> u32 {
        (timestamp_ns as f64 / 1e9 * self.fps as f64) as u32
    }

    fn probe_resolution(&mut self) -> VideoResult<(u32, u32)> {
        if let Some(res) = self.probed_resolution {
            return Ok(res);
        }
        let path = self
            .video_path
            .as_ref()
            .ok_or_else(|| VideoError::ExtractionFailed("no video_path configured".to_string()))?;

        let output = Command::new("ffprobe")
            .args(["-v", "error", "-select_streams", "v:0", "-show_entries", "stream=width,height", "-of", "csv=s=x:p=0"])
            .arg(path)
            .output()
            .map_err(|e| VideoError::ExtractionFailed(format!("failed to run ffprobe: {e}")))?;

        if !output.status.success() {
            return Err(VideoError::ExtractionFailed(format!(
                "ffprobe exited with {}: {}",
                output.status,
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        let text = String::from_utf8_lossy(&output.stdout);
        let parts: Vec<&str> = text.trim().split('x').collect();
        if parts.len() != 2 {
            return Err(VideoError::ExtractionFailed(format!("unexpected ffprobe output: {text}")));
        }
        let width: u32 = parts[0].parse().map_err(|_| VideoError::ExtractionFailed(format!("bad width in: {text}")))?;
        let height: u32 = parts[1].parse().map_err(|_| VideoError::ExtractionFailed(format!("bad height in: {text}")))?;

        self.probed_resolution = Some((width, height));
        Ok((width, height))
    }

    /// Extract and cache a real frame from the configured video file via
    /// ffmpeg (seek to the frame's timestamp, decode exactly one frame as
    /// raw RGB24).
    pub fn get_frame(&mut self, frame_index: u32) -> VideoResult<FrameData> {
        if let Some(frame) = self.frame_cache.get(frame_index) {
            return Ok(frame);
        }

        let timestamp_ns = self.frame_to_timestamp(frame_index);
        let (width, height) = self.probe_resolution()?;
        let path = self.video_path.clone().expect("probe_resolution would have errored without a path");
        let timestamp_sec = timestamp_ns as f64 / 1e9;

        let output = Command::new("ffmpeg")
            .args(["-v", "error", "-ss", &format!("{timestamp_sec}")])
            .arg("-i")
            .arg(&path)
            .args(["-vframes", "1", "-f", "rawvideo", "-pix_fmt", "rgb24", "-"])
            .output()
            .map_err(|e| VideoError::ExtractionFailed(format!("failed to run ffmpeg: {e}")))?;

        if !output.status.success() {
            return Err(VideoError::ExtractionFailed(format!(
                "ffmpeg exited with {}: {}",
                output.status,
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        let pixels = output.stdout;
        let expected_size = (width as usize) * (height as usize) * 3;
        if pixels.len() != expected_size {
            return Err(VideoError::ExtractionFailed(format!(
                "expected {expected_size} bytes ({width}x{height}x3 rgb24) for frame {frame_index}, got {} \
                 (frame index likely past end of video)",
                pixels.len()
            )));
        }

        let frame = FrameData {
            index: frame_index,
            timestamp_ns,
            resolution: (width, height),
            format: "RGB24".to_string(),
            size_bytes: pixels.len() as u32,
            pixels,
        };

        self.frame_cache.insert(frame_index, frame.clone());
        Ok(frame)
    }

    /// Object detection needs a real pretrained model + inference runtime to
    /// produce real results. Rather than return the previous hardcoded
    /// "person, 95% confidence" placeholder (indistinguishable from a real
    /// detection to a caller), this is honest about not being wired up yet.
    pub fn detect_objects(&self, _frame_index: u32) -> VideoResult<Vec<ObjectDetection>> {
        if !self.yolo_enabled {
            return Ok(Vec::new());
        }
        Err(VideoError::NotImplemented(
            "YOLO object detection needs a pretrained model + inference runtime (e.g. the `ort` \
             crate + an ONNX model) that isn't wired in yet. Returning fabricated detections here \
             would be indistinguishable from real ones to a caller, which is worse than an explicit \
             error.".to_string(),
        ))
    }

    /// Compute optical flow between two consecutive frames using block
    /// matching: divide the first frame into a grid of blocks, and for each
    /// block find the best-matching block in the second frame within a
    /// small search window (minimizing sum of absolute luminance
    /// differences). The average displacement across blocks gives the
    /// frame's overall flow magnitude/direction.
    pub fn compute_optical_flow(&mut self, frame1_index: u32, frame2_index: u32) -> VideoResult<OpticalFlowFrame> {
        if !self.optical_flow_enabled {
            return Err(VideoError::DetectionFailed("Optical flow not enabled".to_string()));
        }

        let frame1 = self.get_frame(frame1_index)?;
        let frame2 = self.get_frame(frame2_index)?;
        if frame1.resolution != frame2.resolution {
            return Err(VideoError::DetectionFailed("frame resolutions don't match".to_string()));
        }

        let (width, height) = frame1.resolution;
        let gray1 = to_grayscale(&frame1.pixels, width, height);
        let gray2 = to_grayscale(&frame2.pixels, width, height);

        let (magnitude, direction) = block_matching_flow(&gray1, &gray2, width, height);

        Ok(OpticalFlowFrame {
            frame_index: frame2_index,
            timestamp_ns: self.frame_to_timestamp(frame2_index),
            flow_magnitude: magnitude,
            dominant_direction: direction,
        })
    }

    /// Estimate lighting conditions from a real luminance histogram of the
    /// frame's pixel data.
    pub fn analyze_lighting(&mut self, frame_index: u32) -> VideoResult<LightingAnalysis> {
        if !self.lighting_enabled {
            return Err(VideoError::DetectionFailed("Lighting analysis not enabled".to_string()));
        }

        let frame = self.get_frame(frame_index)?;
        let (width, height) = frame.resolution;
        let gray = to_grayscale(&frame.pixels, width, height);

        let n = gray.len().max(1) as f32;
        let mean = gray.iter().map(|&v| v as f32).sum::<f32>() / n;
        let variance = gray.iter().map(|&v| (v as f32 - mean).powi(2)).sum::<f32>() / n;
        let stddev = variance.sqrt();
        // Normalize contrast to 0-1 assuming max plausible stddev for 8-bit
        // luminance is ~127 (a perfectly bimodal black/white image).
        let contrast = (stddev / 127.0).min(1.0);

        Ok(LightingAnalysis { frame_index, luminance: mean, contrast, brightness: mean })
    }

    /// Monocular depth estimation genuinely needs a pretrained model (e.g.
    /// MiDaS) — same reasoning as detect_objects: an honest error instead of
    /// a fabricated constant depth map.
    pub fn estimate_depth(&self, _frame_index: u32) -> VideoResult<DepthEstimate> {
        if !self.depth_enabled {
            return Err(VideoError::DetectionFailed("Depth estimation not enabled".to_string()));
        }
        Err(VideoError::NotImplemented(
            "Monocular depth estimation needs a pretrained model (e.g. MiDaS) that isn't wired in \
             yet. Returning a fabricated constant depth map here would be indistinguishable from a \
             real estimate to a caller, which is worse than an explicit error.".to_string(),
        ))
    }
}

/// Convert an RGB24 buffer to 8-bit luminance using the standard Rec. 601
/// coefficients.
fn to_grayscale(rgb: &[u8], width: u32, height: u32) -> Vec<u8> {
    let n = (width as usize) * (height as usize);
    let mut gray = Vec::with_capacity(n);
    for i in 0..n {
        let base = i * 3;
        if base + 2 >= rgb.len() {
            break;
        }
        let r = rgb[base] as f32;
        let g = rgb[base + 1] as f32;
        let b = rgb[base + 2] as f32;
        gray.push((0.299 * r + 0.587 * g + 0.114 * b) as u8);
    }
    gray
}

/// Block-matching optical flow: returns (average magnitude, dominant
/// direction in radians) across all blocks with detectable motion.
fn block_matching_flow(gray1: &[u8], gray2: &[u8], width: u32, height: u32) -> (f32, f32) {
    const BLOCK_SIZE: usize = 16;
    const SEARCH_RADIUS: i32 = 8;

    let width = width as usize;
    let height = height as usize;
    if width < BLOCK_SIZE * 2 || height < BLOCK_SIZE * 2 {
        return (0.0, 0.0);
    }

    let mut dx_sum = 0.0f32;
    let mut dy_sum = 0.0f32;
    let mut count = 0u32;

    let mut by = BLOCK_SIZE;
    while by + BLOCK_SIZE < height {
        let mut bx = BLOCK_SIZE;
        while bx + BLOCK_SIZE < width {
            if let Some((best_dx, best_dy)) =
                best_match(gray1, gray2, width, height, bx, by, BLOCK_SIZE, SEARCH_RADIUS)
            {
                dx_sum += best_dx as f32;
                dy_sum += best_dy as f32;
                count += 1;
            }
            bx += BLOCK_SIZE;
        }
        by += BLOCK_SIZE;
    }

    if count == 0 {
        return (0.0, 0.0);
    }
    let avg_dx = dx_sum / count as f32;
    let avg_dy = dy_sum / count as f32;
    let magnitude = (avg_dx * avg_dx + avg_dy * avg_dy).sqrt();
    let direction = avg_dy.atan2(avg_dx);
    (magnitude, direction)
}

fn block_sad(gray1: &[u8], gray2: &[u8], width: usize, bx: usize, by: usize, size: usize, dx: i32, dy: i32) -> Option<u64> {
    let mut sad: u64 = 0;
    for y in 0..size {
        for x in 0..size {
            let x1 = bx + x;
            let y1 = by + y;
            let x2 = x1 as i32 + dx;
            let y2 = y1 as i32 + dy;
            if x2 < 0 || y2 < 0 {
                return None;
            }
            let (x2, y2) = (x2 as usize, y2 as usize);
            let idx1 = y1 * width + x1;
            let idx2 = y2 * width + x2;
            let p1 = *gray1.get(idx1)?;
            let p2 = *gray2.get(idx2)?;
            sad += (p1 as i32 - p2 as i32).unsigned_abs() as u64;
        }
    }
    Some(sad)
}

fn best_match(
    gray1: &[u8],
    gray2: &[u8],
    width: usize,
    _height: usize,
    bx: usize,
    by: usize,
    size: usize,
    radius: i32,
) -> Option<(i32, i32)> {
    let mut best_sad = u64::MAX;
    let mut best = (0, 0);
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            if let Some(sad) = block_sad(gray1, gray2, width, bx, by, size, dx, dy) {
                if sad < best_sad {
                    best_sad = sad;
                    best = (dx, dy);
                }
            }
        }
    }
    if best_sad == u64::MAX {
        None
    } else {
        Some(best)
    }
}

// ============================================================================
// Frame Cache (LRU)
// ============================================================================

struct FrameCache {
    cache: VecDeque<(u32, FrameData)>,
    max_size: usize,
}

impl FrameCache {
    fn new(max_size: usize) -> Self {
        FrameCache {
            cache: VecDeque::with_capacity(max_size),
            max_size,
        }
    }

    fn get(&self, frame_index: u32) -> Option<FrameData> {
        self.cache.iter()
            .find(|(idx, _)| *idx == frame_index)
            .map(|(_, data)| data.clone())
    }

    fn insert(&mut self, frame_index: u32, frame: FrameData) {
        // Remove if already exists
        self.cache.retain(|(idx, _)| *idx != frame_index);

        // Add to front
        self.cache.push_front((frame_index, frame));

        // Trim if over capacity
        if self.cache.len() > self.max_size {
            self.cache.pop_back();
        }
    }

    #[allow(dead_code)]
    fn clear(&mut self) {
        self.cache.clear();
    }

    fn size(&self) -> usize {
        self.cache.len()
    }
}

// ============================================================================
// Video Stream Processor
// ============================================================================

pub struct VideoStreamProcessor {
    processors: std::collections::HashMap<String, VideoProcessor>,
}

impl VideoStreamProcessor {
    pub fn new() -> Self {
        VideoStreamProcessor {
            processors: std::collections::HashMap::new(),
        }
    }

    pub fn register_camera(&mut self, camera_name: String, fps: f32) -> &mut VideoProcessor {
        self.processors.entry(camera_name)
            .or_insert_with(|| VideoProcessor::new(fps))
    }

    pub fn get_camera(&self, camera_name: &str) -> Option<&VideoProcessor> {
        self.processors.get(camera_name)
    }

    pub fn get_camera_mut(&mut self, camera_name: &str) -> Option<&mut VideoProcessor> {
        self.processors.get_mut(camera_name)
    }

    pub fn cameras(&self) -> Vec<String> {
        self.processors.keys().cloned().collect()
    }
}

impl Default for VideoStreamProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_video_processor_creation() {
        let processor = VideoProcessor::new(30.0);
        assert_eq!(processor.fps, 30.0);
    }

    #[test]
    fn test_frame_time_conversion() {
        let processor = VideoProcessor::new(30.0);
        let timestamp = processor.frame_to_timestamp(30);  // Frame 30 at 30fps = 1 second
        assert_eq!(timestamp, 1_000_000_000);  // 1 second in nanoseconds
    }

    #[test]
    fn test_timestamp_to_frame_conversion() {
        let processor = VideoProcessor::new(30.0);
        let frame = processor.timestamp_to_frame(1_000_000_000);  // 1 second
        assert_eq!(frame, 30);  // Frame 30 at 30fps
    }

    #[test]
    fn test_enable_pipelines() {
        let processor = VideoProcessor::new(30.0)
            .enable_yolo()
            .enable_optical_flow()
            .enable_lighting()
            .enable_depth();

        assert!(processor.yolo_enabled);
        assert!(processor.optical_flow_enabled);
        assert!(processor.lighting_enabled);
        assert!(processor.depth_enabled);
    }

    #[test]
    fn test_frame_cache() {
        let mut cache = FrameCache::new(3);

        let frame1 = FrameData {
            index: 0,
            timestamp_ns: 0,
            resolution: (1920, 1080),
            format: "RGB".to_string(),
            size_bytes: 1920 * 1080 * 3,
            pixels: Vec::new(),
        };

        cache.insert(0, frame1.clone());
        assert_eq!(cache.size(), 1);

        let retrieved = cache.get(0);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().index, 0);
    }

    #[test]
    fn test_video_stream_processor() {
        let mut processor = VideoStreamProcessor::new();
        processor.register_camera("front".to_string(), 30.0);
        processor.register_camera("rear".to_string(), 10.0);

        assert_eq!(processor.cameras().len(), 2);
        assert!(processor.get_camera("front").is_some());
    }

    #[test]
    fn get_frame_without_video_path_errors_clearly_not_fake_data() {
        let mut processor = VideoProcessor::new(30.0);
        let err = processor.get_frame(0).unwrap_err();
        assert!(matches!(err, VideoError::ExtractionFailed(_)));
    }

    #[test]
    fn detect_objects_returns_not_implemented_not_fake_detections() {
        let processor = VideoProcessor::new(30.0).enable_yolo();
        let err = processor.detect_objects(0).unwrap_err();
        assert!(matches!(err, VideoError::NotImplemented(_)));
    }

    #[test]
    fn estimate_depth_returns_not_implemented_not_fake_depth() {
        let processor = VideoProcessor::new(30.0).enable_depth();
        let err = processor.estimate_depth(0).unwrap_err();
        assert!(matches!(err, VideoError::NotImplemented(_)));
    }

    #[test]
    fn detect_objects_returns_empty_not_error_when_disabled() {
        let processor = VideoProcessor::new(30.0); // yolo NOT enabled
        assert!(processor.detect_objects(0).unwrap().is_empty());
    }

    #[test]
    fn to_grayscale_uses_rec601_luminance() {
        // Pure red, green, blue, white pixels.
        let rgb = vec![255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 255, 255];
        let gray = to_grayscale(&rgb, 4, 1);
        assert_eq!(gray.len(), 4);
        assert_eq!(gray[0], (0.299 * 255.0) as u8); // red
        assert_eq!(gray[1], (0.587 * 255.0) as u8); // green
        assert_eq!(gray[2], (0.114 * 255.0) as u8); // blue
        assert_eq!(gray[3], 255); // white
    }

    /// Deterministic pseudo-random (non-periodic) texture generator. Block
    /// matching is inherently ambiguous on periodic/repeating textures (the
    /// classic "aperture problem" — a checkerboard shifted by exactly one
    /// period is indistinguishable from not shifting at all), so tests need
    /// non-periodic texture for the true displacement to be unambiguous.
    fn noise_texture(width: usize, height: usize, seed: u32) -> Vec<u8> {
        let mut state = seed.wrapping_mul(2654435761).wrapping_add(1);
        let mut next = move || {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            state
        };
        (0..width * height).map(|_| (next() % 256) as u8).collect()
    }

    fn shift_texture(src: &[u8], width: usize, height: usize, dx: i32, dy: i32) -> Vec<u8> {
        let mut out = vec![0u8; width * height];
        for y in 0..height {
            for x in 0..width {
                let sx = x as i32 - dx;
                let sy = y as i32 - dy;
                if sx >= 0 && sy >= 0 && (sx as usize) < width && (sy as usize) < height {
                    out[y * width + x] = src[sy as usize * width + sx as usize];
                }
            }
        }
        out
    }

    #[test]
    fn block_matching_flow_detects_pure_horizontal_shift() {
        let width = 96usize;
        let height = 96usize;
        let base = noise_texture(width, height, 42);
        let gray1 = base.clone();
        let gray2 = shift_texture(&base, width, height, 4, 0); // shifted 4px right

        let (magnitude, direction) = block_matching_flow(&gray1, &gray2, width as u32, height as u32);
        assert!((magnitude - 4.0).abs() < 0.6, "expected magnitude ~4.0, got {magnitude}");
        // Rightward shift -> direction should be near 0 radians (positive x axis).
        assert!(direction.abs() < 0.3, "expected near-horizontal direction, got {direction}");
    }

    #[test]
    fn block_matching_flow_is_near_zero_for_static_frames() {
        let width = 96usize;
        let height = 96usize;
        let buf = noise_texture(width, height, 7);
        let (magnitude, _) = block_matching_flow(&buf, &buf, width as u32, height as u32);
        assert!(magnitude < 0.5, "expected ~0 motion for identical frames, got {magnitude}");
    }

    /// True end-to-end test against a real video file: generates a small
    /// synthetic test-pattern video with the system `ffmpeg` (a "testsrc"
    /// pattern — deterministic, no external asset needed), then verifies
    /// get_frame/analyze_lighting genuinely decode real pixel data from it.
    /// Skipped (not failed) if ffmpeg isn't installed, since that's an
    /// environment gap, not a code bug.
    #[test]
    fn real_ffmpeg_decode_end_to_end() {
        if Command::new("ffmpeg").arg("-version").output().map(|o| !o.status.success()).unwrap_or(true) {
            eprintln!("skipping: ffmpeg not available in this environment");
            return;
        }

        let dir = std::env::temp_dir().join(format!("pyroboreplay_video_test_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let video_path = dir.join("test.mp4");

        let gen = Command::new("ffmpeg")
            .args([
                "-y", "-v", "error",
                "-f", "lavfi", "-i", "testsrc=duration=1:size=64x64:rate=10",
                video_path.to_str().unwrap(),
            ])
            .output()
            .expect("failed to run ffmpeg to generate test video");
        assert!(gen.status.success(), "ffmpeg test video generation failed: {}", String::from_utf8_lossy(&gen.stderr));

        let mut processor = VideoProcessor::new(10.0).with_video_path(&video_path).enable_lighting();

        let frame = processor.get_frame(0).unwrap();
        assert_eq!(frame.resolution, (64, 64));
        assert_eq!(frame.pixels.len(), 64 * 64 * 3);
        // testsrc is a colorful pattern, not a blank frame — pixel data should
        // have real variation, not be all-zero or all-one-value.
        assert!(frame.pixels.iter().any(|&b| b != frame.pixels[0]), "decoded frame looks uniform/fake");

        let lighting = processor.analyze_lighting(0).unwrap();
        assert!(lighting.luminance > 0.0 && lighting.luminance < 255.0);

        std::fs::remove_dir_all(&dir).ok();
    }
}
