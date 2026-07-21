#!/usr/bin/env python3
"""
Test script for PyRoboReplay Python API
Demonstrates sensor replay, filtering, and analysis
"""

import sys

try:
    from pyroboreplay import Mission
except ImportError:
    print("ERROR: PyRoboReplay not installed. Build with: pip install -e .")
    sys.exit(1)


def test_mission_loading():
    """Test loading a mission from a bag file"""
    print("\n📂 Test 1: Loading mission from bag file")
    print("━" * 50)

    mission = Mission.from_ros_bag("warehouse_exploration_v1.db3")

    print(f"✅ Mission loaded successfully")
    print(f"   ID: {mission.mission_id()}")
    print(f"   Name: {mission.name()}")
    print(f"   Events: {mission.event_count()}")
    print(f"   Duration: {mission.duration_seconds()}s")


def test_sensor_discovery():
    """Test discovering available sensors"""
    print("\n📡 Test 2: Sensor discovery")
    print("━" * 50)

    mission = Mission.from_ros_bag("warehouse_exploration_v1.db3")
    sensors = mission.get_available_sensors()

    print(f"✅ Found {len(sensors)} sensors:")
    for sensor in sensors:
        print(f"   - {sensor}")


def test_sensor_frames():
    """Test querying sensor-specific frames"""
    print("\n🎥 Test 3: Get sensor frames")
    print("━" * 50)

    mission = Mission.from_ros_bag("warehouse_exploration_v1.db3")

    # Get lidar frames
    lidar_frames = mission.get_sensor_frames("lidar")
    print(f"✅ Lidar frames: {len(lidar_frames)}")
    if lidar_frames:
        first_frame = lidar_frames[0]
        print(f"   First frame: {first_frame}")
        print(f"   Type: {first_frame.get_event_type()}")
        print(f"   Timestamp: {first_frame.get_timestamp()}")
        print(f"   Sensor: {first_frame.get_sensor_type()}")

    # Get camera frames
    camera_frames = mission.get_sensor_frames("camera")
    print(f"\n✅ Camera frames: {len(camera_frames)}")

    # Get IMU data
    imu_frames = mission.get_sensor_frames("imu")
    print(f"✅ IMU frames: {len(imu_frames)}")


def test_multi_sensor():
    """Test querying multiple sensor types"""
    print("\n🔀 Test 4: Multi-sensor filtering")
    print("━" * 50)

    mission = Mission.from_ros_bag("warehouse_exploration_v1.db3")

    # Get lidar and camera frames
    frames = mission.get_multi_sensor_frames(["lidar", "camera"])
    print(f"✅ Lidar + Camera frames: {len(frames)}")


def test_event_statistics():
    """Test event counting and statistics"""
    print("\n📊 Test 5: Event statistics")
    print("━" * 50)

    mission = Mission.from_ros_bag("warehouse_exploration_v1.db3")

    event_counts = mission.get_event_counts()
    print(f"✅ Event type breakdown:")
    for event_type, count in event_counts:
        percentage = (count / mission.event_count()) * 100
        print(f"   {event_type}: {count} ({percentage:.1f}%)")


def test_all_events():
    """Test getting all events"""
    print("\n📋 Test 6: Get all events")
    print("━" * 50)

    mission = Mission.from_ros_bag("warehouse_exploration_v1.db3")

    all_events = mission.get_all_events()
    print(f"✅ Total events: {len(all_events)}")

    if all_events:
        print(f"\nFirst 5 events:")
        for i, event in enumerate(all_events[:5]):
            print(f"  {i+1}. {event}")


def main():
    """Run all tests"""
    print("\n" + "=" * 50)
    print("PyRoboReplay Python API Tests")
    print("=" * 50)

    try:
        test_mission_loading()
        test_sensor_discovery()
        test_sensor_frames()
        test_multi_sensor()
        test_event_statistics()
        test_all_events()

        print("\n" + "=" * 50)
        print("✅ All tests passed!")
        print("=" * 50)

    except Exception as e:
        print(f"\n❌ Test failed with error: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)


if __name__ == "__main__":
    main()
