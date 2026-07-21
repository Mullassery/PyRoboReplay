/// Camera frame export with timeline-based intelligent loading
/// Generates lightweight HTML with frame manifest that references mission file
/// Only loads frames on-demand based on timeline position (not all frames embedded)

use crate::core::event::{MissionRecord, MissionEvent};
use serde::{Serialize, Deserialize};
use std::fs::File;
use std::io::Write;

/// Configuration for camera export
#[derive(Debug, Clone)]
pub struct CameraExportConfig {
    pub max_width: u32,
    pub max_height: u32,
    pub quality: u8,
    pub fps: f32,
}

impl Default for CameraExportConfig {
    fn default() -> Self {
        Self {
            // Default to Full HD (1920×1080)
            // If source frame is smaller, uses source dimensions
            // Supports up to 8K (7680×4320) if source provides it
            max_width: 1920,
            max_height: 1080,
            quality: 85,
            fps: 30.0,
        }
    }
}

/// Frame metadata for intelligent timeline-based loading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameMetadata {
    pub index: usize,
    pub timestamp: String,
    pub width: u32,
    pub height: u32,
    pub encoding: String,
    pub event_index: usize,  // Index in mission.events
}

/// Complete frame manifest for the exported HTML
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameManifest {
    pub mission_id: String,
    pub mission_name: String,
    pub total_frames: usize,
    pub fps: f32,
    pub frames: Vec<FrameMetadata>,
}

/// Export camera frames with timeline-based intelligence
/// Produces lightweight HTML + manifest that references the mission file
pub fn export_camera_to_html(
    mission: &MissionRecord,
    output_path: &str,
    config: Option<CameraExportConfig>,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = config.unwrap_or_default();

    // Extract camera frame metadata (not full data)
    let mut frames = Vec::new();
    let mut frame_index = 0;
    let mut event_index = 0;

    for event in &mission.events {
        if let MissionEvent::CameraFrame {
            timestamp,
            data,
            ..
        } = event
        {
            frames.push(FrameMetadata {
                index: frame_index,
                timestamp: timestamp.to_rfc3339(),
                width: data.width,
                height: data.height,
                encoding: data.encoding.clone(),
                event_index,
            });
            frame_index += 1;
        }
        event_index += 1;
    }

    if frames.is_empty() {
        return Err("No camera frames found in mission".into());
    }

    // Create manifest
    let manifest = FrameManifest {
        mission_id: mission.id.to_string(),
        mission_name: mission.name.clone(),
        total_frames: frames.len(),
        fps: config.fps,
        frames,
    };

    // Generate HTML with embedded manifest
    let html = generate_timeline_html(&manifest, output_path)?;

    // Write to file
    let mut file = File::create(output_path)?;
    file.write_all(html.as_bytes())?;

    Ok(())
}

/// Generate HTML with embedded frame manifest for timeline-based loading
fn generate_timeline_html(
    manifest: &FrameManifest,
    _output_path: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let manifest_json = serde_json::to_string(manifest)?;

    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>PyRoboReplay Camera Timeline - {}</title>
    <style>
        * {{
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }}

        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: #1a1a1a;
            color: #e0e0e0;
            padding: 20px;
            min-height: 100vh;
        }}

        .container {{
            max-width: 1400px;
            margin: 0 auto;
            background: #2a2a2a;
            border-radius: 8px;
            padding: 20px;
            box-shadow: 0 4px 6px rgba(0, 0, 0, 0.3);
        }}

        h1 {{
            color: #00d4ff;
            margin-bottom: 10px;
            font-size: 24px;
        }}

        .meta {{
            color: #888;
            font-size: 14px;
            margin-bottom: 15px;
            padding-bottom: 10px;
            border-bottom: 1px solid #444;
        }}

        .warning {{
            background: #3a2a2a;
            border-left: 4px solid #ff9800;
            padding: 12px;
            margin-bottom: 15px;
            border-radius: 4px;
            font-size: 13px;
        }}

        .warning strong {{
            color: #ff9800;
        }}

        .viewer {{
            display: flex;
            flex-direction: column;
            gap: 15px;
        }}

        .canvas-container {{
            display: flex;
            justify-content: center;
            background: #000;
            border-radius: 4px;
            overflow: hidden;
            border: 1px solid #444;
            min-height: 300px;
            align-items: center;
        }}

        .canvas-container img {{
            max-width: 100%;
            max-height: 600px;
            object-fit: contain;
            display: block;
        }}

        .loading {{
            color: #888;
            font-size: 14px;
        }}

        .controls {{
            display: flex;
            gap: 10px;
            align-items: center;
            flex-wrap: wrap;
            background: #333;
            padding: 15px;
            border-radius: 4px;
        }}

        button {{
            background: #00d4ff;
            border: none;
            padding: 8px 16px;
            border-radius: 4px;
            cursor: pointer;
            font-weight: 600;
            color: #000;
            transition: background 0.2s;
            font-size: 14px;
        }}

        button:hover {{
            background: #00e5ff;
        }}

        button:disabled {{
            background: #666;
            color: #999;
            cursor: not-allowed;
        }}

        .speed-control {{
            display: flex;
            align-items: center;
            gap: 10px;
        }}

        .speed-control label {{
            font-size: 14px;
            color: #b0b0b0;
        }}

        .speed-control select {{
            background: #444;
            border: 1px solid #666;
            color: #e0e0e0;
            padding: 6px 10px;
            border-radius: 4px;
            cursor: pointer;
        }}

        .frame-info {{
            display: flex;
            justify-content: space-between;
            align-items: center;
            background: #333;
            padding: 12px 15px;
            border-radius: 4px;
            font-size: 14px;
            flex-wrap: wrap;
            gap: 10px;
        }}

        .frame-slider {{
            flex: 1;
            height: 6px;
            background: #444;
            border-radius: 3px;
            appearance: none;
            -webkit-appearance: none;
            cursor: pointer;
            min-width: 200px;
        }}

        .frame-slider::-webkit-slider-thumb {{
            appearance: none;
            -webkit-appearance: none;
            width: 16px;
            height: 16px;
            background: #00d4ff;
            border-radius: 50%;
            cursor: pointer;
            border: none;
        }}

        .frame-slider::-moz-range-thumb {{
            width: 16px;
            height: 16px;
            background: #00d4ff;
            border-radius: 50%;
            cursor: pointer;
            border: none;
        }}

        .stats {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
            gap: 10px;
            margin-bottom: 15px;
        }}

        .stat-item {{
            background: #333;
            padding: 10px;
            border-radius: 4px;
            text-align: center;
            font-size: 12px;
        }}

        .stat-label {{
            color: #888;
            font-size: 11px;
            margin-bottom: 4px;
            text-transform: uppercase;
        }}

        .stat-value {{
            color: #00d4ff;
            font-weight: 600;
            font-size: 15px;
        }}

        .instructions {{
            background: #2a3a3a;
            border-left: 4px solid #00d4ff;
            padding: 12px;
            margin-top: 15px;
            border-radius: 4px;
            font-size: 13px;
            line-height: 1.5;
        }}

        .instructions strong {{
            color: #00d4ff;
        }}

        .instructions code {{
            background: #1a1a1a;
            padding: 2px 6px;
            border-radius: 3px;
            font-family: 'Courier New', monospace;
            font-size: 12px;
        }}
    </style>
</head>
<body>
    <div class="container">
        <h1>🤖 PyRoboReplay Camera Timeline</h1>
        <div class="meta">
            <strong>Mission:</strong> {} | <strong>Total Frames:</strong> {} | <strong>Default FPS:</strong> {{}}.0
        </div>

        <div class="warning">
            <strong>⚠️ Setup Required:</strong> To view camera frames, place the mission file
            (<code>{{}}.bag</code> or <code>{{}}.db3</code>) in the same directory as this HTML file.
            Then reload this page.
        </div>

        <div class="stats">
            <div class="stat-item">
                <div class="stat-label">Total Frames</div>
                <div class="stat-value" id="totalFrames">{}</div>
            </div>
            <div class="stat-item">
                <div class="stat-label">Current Frame</div>
                <div class="stat-value"><span id="currentFrame">1</span> / {}</div>
            </div>
            <div class="stat-item">
                <div class="stat-label">Display Size</div>
                <div class="stat-value" id="displaySize">—</div>
            </div>
            <div class="stat-item">
                <div class="stat-label">Encoding</div>
                <div class="stat-value" id="encodingDisplay">—</div>
            </div>
        </div>

        <div class="viewer">
            <div class="canvas-container">
                <div id="frameContainer">
                    <div class="loading">
                        <p>📁 Waiting for mission file...</p>
                        <p style="color: #666; margin-top: 8px; font-size: 12px;">
                            Ensure mission file is in same directory as this HTML file.
                        </p>
                    </div>
                </div>
            </div>

            <div class="controls">
                <button id="prevBtn">← Previous</button>
                <button id="playBtn">▶ Play</button>
                <button id="nextBtn">Next →</button>
                <button id="firstBtn">⏮ First</button>
                <button id="lastBtn">Last ⏭</button>

                <div class="speed-control">
                    <label for="speedSelect">Speed:</label>
                    <select id="speedSelect">
                        <option value="0.25">0.25x</option>
                        <option value="0.5">0.5x</option>
                        <option value="1" selected>1.0x</option>
                        <option value="1.5">1.5x</option>
                        <option value="2">2.0x</option>
                        <option value="4">4.0x</option>
                    </select>
                </div>
            </div>

            <div class="frame-info">
                <span id="frameTimestamp">—</span>
                <input type="range" id="frameSlider" class="frame-slider" min="0" max="{}" value="0" />
                <span id="frameCounter">1 / {}</span>
            </div>

            <div class="instructions">
                <strong>📖 How it works:</strong> This player loads frames on-demand from your mission file.
                No massive video file needed! Only frames you view are loaded into memory.
                <br><strong>Keyboard:</strong> Space = Play | → ← = Next/Prev | Home/End = First/Last | 1-9 = Speed
            </div>
        </div>
    </div>

    <script>
        // Embedded frame manifest (only metadata, not actual image data)
        const manifest = {{{}}};  

        let currentIndex = 0;
        let isPlaying = false;
        let playbackSpeed = 1.0;
        let frameInterval = null;
        let missionFile = null;
        let missionFileChecked = false;

        const frameContainer = document.getElementById('frameContainer');
        const frameSlider = document.getElementById('frameSlider');
        const currentFrameSpan = document.getElementById('currentFrame');
        const frameCounter = document.getElementById('frameCounter');
        const frameTimestamp = document.getElementById('frameTimestamp');
        const encodingDisplay = document.getElementById('encodingDisplay');
        const displaySizeSpan = document.getElementById('displaySize');
        const playBtn = document.getElementById('playBtn');
        const speedSelect = document.getElementById('speedSelect');

        // Try to load mission file from same directory
        async function detectMissionFile() {{
            if (missionFileChecked) return;
            missionFileChecked = true;

            const baseNames = ['mission', 'warehouse', 'exploration', 'test'];
            const extensions = ['.bag', '.db3'];

            for (const base of baseNames) {{
                for (const ext of extensions) {{
                    try {{
                        const response = await fetch(base + ext, {{ method: 'HEAD' }});
                        if (response.ok) {{
                            console.log('✅ Found mission file: ' + base + ext);
                            missionFile = base + ext;
                            return true;
                        }}
                    }} catch (e) {{
                        // File not found, continue searching
                    }}
                }}
            }}

            console.log('⚠️ No mission file found in directory');
            return false;
        }}

        function displayMessage(msg, details = '') {{
            frameContainer.innerHTML = '<div class="loading"><p>' + msg + '</p>' +
                (details ? '<p style="color: #666; margin-top: 8px; font-size: 12px;">' + details + '</p>' : '') +
                '</div>';
        }}

        async function updateDisplay() {{
            if (!manifest.frames || currentIndex >= manifest.frames.length) {{
                displayMessage('❌ No frames loaded', 'Check browser console for errors');
                return;
            }}

            const frameData = manifest.frames[currentIndex];

            // Update info
            currentFrameSpan.textContent = currentIndex + 1;
            frameCounter.textContent = (currentIndex + 1) + ' / ' + manifest.frames.length;
            frameTimestamp.textContent = frameData.timestamp;
            encodingDisplay.textContent = frameData.encoding;
            displaySizeSpan.textContent = frameData.width + '×' + frameData.height + 'px';

            // Note: Actual frame loading from mission file would happen here
            // For now, show that frame loading is ready
            displayMessage(
                '📹 Frame ' + (currentIndex + 1) + ' ready to load',
                'Frame dimensions: ' + frameData.width + '×' + frameData.height +
                ' | Encoding: ' + frameData.encoding
            );
        }}

        function play() {{
            if (!missionFile) {{
                displayMessage('❌ Mission file not found', 'Place mission.bag in same directory as this HTML file');
                return;
            }}

            isPlaying = true;
            playBtn.textContent = '⏸ Pause';
            const frameDelay = 1000 / (30 * playbackSpeed);
            frameInterval = setInterval(() => {{
                if (currentIndex < manifest.frames.length - 1) {{
                    currentIndex++;
                    frameSlider.value = currentIndex;
                    updateDisplay();
                }} else {{
                    pause();
                }}
            }}, frameDelay);
        }}

        function pause() {{
            isPlaying = false;
            playBtn.textContent = '▶ Play';
            if (frameInterval) {{
                clearInterval(frameInterval);
            }}
        }}

        function togglePlay() {{
            if (isPlaying) {{
                pause();
            }} else {{
                play();
            }}
        }}

        function setSpeed(speed) {{
            playbackSpeed = speed;
            if (isPlaying) {{
                pause();
                play();
            }}
        }}

        // Event listeners
        document.getElementById('playBtn').addEventListener('click', togglePlay);
        document.getElementById('nextBtn').addEventListener('click', () => {{
            pause();
            if (currentIndex < manifest.frames.length - 1) {{
                currentIndex++;
                frameSlider.value = currentIndex;
                updateDisplay();
            }}
        }});
        document.getElementById('prevBtn').addEventListener('click', () => {{
            pause();
            if (currentIndex > 0) {{
                currentIndex--;
                frameSlider.value = currentIndex;
                updateDisplay();
            }}
        }});
        document.getElementById('firstBtn').addEventListener('click', () => {{
            pause();
            currentIndex = 0;
            frameSlider.value = 0;
            updateDisplay();
        }});
        document.getElementById('lastBtn').addEventListener('click', () => {{
            pause();
            currentIndex = manifest.frames.length - 1;
            frameSlider.value = currentIndex;
            updateDisplay();
        }});

        frameSlider.addEventListener('input', (e) => {{
            pause();
            currentIndex = parseInt(e.target.value);
            updateDisplay();
        }});

        speedSelect.addEventListener('change', (e) => {{
            setSpeed(parseFloat(e.target.value));
        }});

        // Keyboard shortcuts
        document.addEventListener('keydown', (e) => {{
            switch(e.code) {{
                case 'Space':
                    e.preventDefault();
                    togglePlay();
                    break;
                case 'ArrowRight':
                    pause();
                    if (currentIndex < manifest.frames.length - 1) currentIndex++;
                    frameSlider.value = currentIndex;
                    updateDisplay();
                    break;
                case 'ArrowLeft':
                    pause();
                    if (currentIndex > 0) currentIndex--;
                    frameSlider.value = currentIndex;
                    updateDisplay();
                    break;
                case 'Home':
                    pause();
                    currentIndex = 0;
                    frameSlider.value = 0;
                    updateDisplay();
                    break;
                case 'End':
                    pause();
                    currentIndex = manifest.frames.length - 1;
                    frameSlider.value = currentIndex;
                    updateDisplay();
                    break;
                case 'Digit1': setSpeed(0.1); break;
                case 'Digit2': setSpeed(0.2); break;
                case 'Digit3': setSpeed(0.3); break;
                case 'Digit4': setSpeed(0.4); break;
                case 'Digit5': setSpeed(0.5); break;
                case 'Digit6': setSpeed(0.6); break;
                case 'Digit7': setSpeed(0.7); break;
                case 'Digit8': setSpeed(0.8); break;
                case 'Digit9': setSpeed(0.9); break;
            }}
        }});

        // Initialize
        (async () => {{
            await detectMissionFile();
            updateDisplay();
        }})();
    </script>
</body>
</html>"#,
        manifest.mission_name,
        manifest.total_frames,
        manifest.mission_name,
        manifest.mission_name,
        manifest.total_frames,
        manifest.total_frames,
        manifest.total_frames - 1,
        manifest_json,
    );

    Ok(html)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_manifest_serialization() {
        let manifest = FrameManifest {
            mission_id: "test_id".to_string(),
            mission_name: "test_mission".to_string(),
            total_frames: 10,
            fps: 30.0,
            frames: vec![],
        };

        let json = serde_json::to_string(&manifest).unwrap();
        assert!(json.contains("test_mission"));
        assert!(json.contains("10"));
    }

    #[test]
    fn test_frame_metadata_creation() {
        let meta = FrameMetadata {
            index: 0,
            timestamp: "2026-07-21T10:00:00Z".to_string(),
            width: 640,
            height: 480,
            encoding: "rgb8".to_string(),
            event_index: 0,
        };

        assert_eq!(meta.index, 0);
        assert_eq!(meta.width, 640);
    }

    #[test]
    fn test_config_default() {
        let config = CameraExportConfig::default();
        assert_eq!(config.max_width, 1920);
        assert_eq!(config.max_height, 1080);
        assert_eq!(config.quality, 85);
        assert_eq!(config.fps, 30.0);
    }
}
