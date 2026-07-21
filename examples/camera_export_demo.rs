/// Demonstration of camera frame export to standalone HTML
/// Shows how to extract camera frames and generate playable HTML

use pyroboreplay::core::event::{MissionRecord, MissionEvent, CameraFrame};
use chrono::Utc;
use pyroboreplay::cli::camera_export::export_camera_to_html;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║       PyRoboReplay Camera Export Demo                          ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    // Create a synthetic mission with sample camera frames
    println!("📹 Creating synthetic mission with 10 camera frames...");
    let mission = create_sample_mission();

    println!("✅ Generated {} camera frames", count_camera_frames(&mission));
    println!("   Frame size: 640×480");
    println!("   Encoding: rgb8 (RGB JPEG)");

    // Export to HTML
    let output_path = "/tmp/camera_replay_demo.html";
    println!("\n🎬 Exporting to HTML...");
    export_camera_to_html(&mission, output_path, None)?;

    println!("✅ Export complete!");
    println!("📖 File: {}", output_path);
    println!("📊 File size: {} bytes", fs::metadata(output_path)?.len());
    println!("\n💡 Open in browser:");
    println!("   macOS: open {}", output_path);
    println!("   Linux: xdg-open {}", output_path);
    println!("   Windows: start {}", output_path);

    println!("\n🎮 Features in exported HTML:");
    println!("   ▶️  Play/Pause button");
    println!("   ⏮  First/Last frame buttons");
    println!("   ← → Previous/Next frame buttons");
    println!("   ⚡ Speed control: 0.25x → 4.0x");
    println!("   🎚️  Frame slider for quick navigation");
    println!("   ⌨️  Keyboard shortcuts:");
    println!("       Space     = Play/Pause");
    println!("       ← / →     = Previous/Next");
    println!("       Home/End  = First/Last");
    println!("       1-9       = Speed (10%-90%)");

    Ok(())
}

fn create_sample_mission() -> MissionRecord {
    let mut mission = MissionRecord::new("camera_demo");

    // Create 10 sample camera frames with simple RGB patterns
    for i in 0..10 {
        let timestamp = Utc::now();

        // Create a simple RGB image (640×480, 3 bytes per pixel)
        // For demo, use a simple gradient pattern
        let mut image_data = Vec::new();
        for y in 0..480 {
            for x in 0..640 {
                // Create a gradient pattern
                let r = ((x as f32 / 640.0) * 255.0) as u8;
                let g = ((y as f32 / 480.0) * 255.0) as u8;
                let b = (((i as f32 / 10.0) * 255.0)) as u8;

                image_data.push(r);
                image_data.push(g);
                image_data.push(b);
            }
        }

        let camera_frame = CameraFrame {
            sensor_id: format!("camera_{}", i),
            frame_id: format!("frame_{}", i),
            width: 640,
            height: 480,
            encoding: "rgb8".to_string(),
            image_data,
            camera_info: None,
        };

        let event = MissionEvent::CameraFrame {
            robot_id: "robot_0".to_string(),
            timestamp,
            data: camera_frame,
        };

        mission.events.push(event);
    }

    mission
}

fn count_camera_frames(mission: &MissionRecord) -> usize {
    mission
        .events
        .iter()
        .filter(|e| matches!(e, MissionEvent::CameraFrame { .. }))
        .count()
}
