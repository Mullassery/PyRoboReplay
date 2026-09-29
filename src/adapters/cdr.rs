//! Real CDR (Common Data Representation) decoding for ROS 2 message bytes.
//!
//! ROS 2 serializes messages on the wire (and in rosbag2 `.db3` files) using
//! OMG CDR, the same binary format used by DDS. This is the real decoder
//! `src/adapters/ros2.rs`'s parsers previously never called -- every parser
//! took the raw `msg.data` bytes and ignored them entirely, returning a
//! hardcoded constant regardless of what was actually recorded. This module
//! implements the real, standard CDR encapsulation header + alignment rules
//! (aligned relative to the start of the encapsulated body, immediately
//! after the 4-byte header -- the same convention used by `fastcdr`,
//! `rosbags`, and the `cdr` crate, which is what makes CDR interoperable
//! across implementations in the first place).

use anyhow::{anyhow, Result};

/// A cursor over real CDR-encoded bytes, tracking position for the
/// alignment rules CDR requires (each primitive of size N must start at an
/// offset that's a multiple of N, relative to the start of the encapsulated
/// body -- i.e. immediately after the 4-byte encapsulation header, not the
/// start of the raw message bytes).
pub struct CdrReader<'a> {
    data: &'a [u8],
    /// Position within `data`, already past the 4-byte encapsulation header.
    pos: usize,
}

impl<'a> CdrReader<'a> {
    /// Parse the 4-byte CDR encapsulation header and return a reader
    /// positioned at the start of the real message body. Real ROS 2
    /// messages are little-endian CDR (representation id `0x01`) unless a
    /// producer explicitly configured big-endian, which is vanishingly rare
    /// in practice -- if we ever see `0x00` (CDR_BE) we say so explicitly
    /// rather than silently misinterpreting the bytes.
    pub fn new(data: &'a [u8]) -> Result<Self> {
        if data.len() < 4 {
            return Err(anyhow!(
                "CDR message too short for encapsulation header: {} bytes",
                data.len()
            ));
        }
        let representation_id = data[1];
        if representation_id != 0x01 {
            return Err(anyhow!(
                "Unsupported CDR representation id {representation_id:#04x} (only little-endian \
                 CDR_LE / 0x01 is supported -- this is what every mainstream ROS 2 RMW \
                 implementation, e.g. rmw_fastrtps and rmw_cyclonedds, produces by default)"
            ));
        }
        Ok(CdrReader { data: &data[4..], pos: 0 })
    }

    fn align(&mut self, size: usize) {
        let rem = self.pos % size;
        if rem != 0 {
            self.pos += size - rem;
        }
    }

    fn take(&mut self, n: usize) -> Result<&'a [u8]> {
        if self.pos + n > self.data.len() {
            return Err(anyhow!(
                "CDR buffer underrun: need {n} bytes at offset {}, but only {} bytes remain",
                self.pos,
                self.data.len() - self.pos
            ));
        }
        let slice = &self.data[self.pos..self.pos + n];
        self.pos += n;
        Ok(slice)
    }

    pub fn read_u8(&mut self) -> Result<u8> {
        Ok(self.take(1)?[0])
    }

    pub fn read_u32(&mut self) -> Result<u32> {
        self.align(4);
        let b = self.take(4)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    pub fn read_i32(&mut self) -> Result<i32> {
        self.align(4);
        let b = self.take(4)?;
        Ok(i32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    pub fn read_f32(&mut self) -> Result<f32> {
        self.align(4);
        let b = self.take(4)?;
        Ok(f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    pub fn read_f64(&mut self) -> Result<f64> {
        self.align(8);
        let b = self.take(8)?;
        Ok(f64::from_le_bytes([
            b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7],
        ]))
    }

    /// Real CDR string: u32 length (including the trailing NUL), then that
    /// many bytes, the last of which is the NUL terminator.
    pub fn read_string(&mut self) -> Result<String> {
        let len = self.read_u32()? as usize;
        if len == 0 {
            return Ok(String::new());
        }
        let bytes = self.take(len)?;
        let without_nul = &bytes[..len.saturating_sub(1)];
        Ok(String::from_utf8_lossy(without_nul).into_owned())
    }

    /// Real CDR sequence of `f32`: u32 count, then that many 4-byte-aligned
    /// elements.
    pub fn read_f32_seq(&mut self) -> Result<Vec<f32>> {
        let count = self.read_u32()? as usize;
        let mut out = Vec::with_capacity(count);
        for _ in 0..count {
            out.push(self.read_f32()?);
        }
        Ok(out)
    }

    /// Real CDR sequence of `u8`: u32 count, then that many raw bytes (no
    /// per-element alignment needed since `u8` has no alignment
    /// requirement).
    pub fn read_u8_seq(&mut self) -> Result<Vec<u8>> {
        let count = self.read_u32()? as usize;
        Ok(self.take(count)?.to_vec())
    }

    /// Real CDR fixed-size array of `f64` (no length prefix -- the IDL
    /// declares a static size, e.g. `float64[9]`/`float64[36]` covariance
    /// arrays).
    pub fn read_f64_array<const N: usize>(&mut self) -> Result<[f64; N]> {
        let mut out = [0.0_f64; N];
        for slot in out.iter_mut() {
            *slot = self.read_f64()?;
        }
        Ok(out)
    }

    /// `builtin_interfaces/Time`: `{int32 sec; uint32 nanosec;}`.
    pub fn read_time(&mut self) -> Result<(i32, u32)> {
        let sec = self.read_i32()?;
        let nanosec = self.read_u32()?;
        Ok((sec, nanosec))
    }

    /// `std_msgs/Header`: `{Time stamp; string frame_id;}`. Returns
    /// `frame_id` (the real per-message frame id, replacing the previous
    /// hardcoded `"laser_frame"`/`"camera_frame"`/etc. constants) --
    /// `(sec, nanosec)` is available via `read_time` for callers that need
    /// the message's own embedded stamp too.
    pub fn read_header(&mut self) -> Result<((i32, u32), String)> {
        let stamp = self.read_time()?;
        let frame_id = self.read_string()?;
        Ok((stamp, frame_id))
    }

    /// `geometry_msgs/Vector3` or `geometry_msgs/Point`: 3 consecutive
    /// `float64`s.
    pub fn read_vec3(&mut self) -> Result<[f64; 3]> {
        Ok([self.read_f64()?, self.read_f64()?, self.read_f64()?])
    }

    /// `geometry_msgs/Quaternion`: `{x, y, z, w: float64}`.
    pub fn read_quaternion(&mut self) -> Result<[f64; 4]> {
        Ok([
            self.read_f64()?,
            self.read_f64()?,
            self.read_f64()?,
            self.read_f64()?,
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn encode_header() -> Vec<u8> {
        vec![0x00, 0x01, 0x00, 0x00]
    }

    #[test]
    fn rejects_short_buffers() {
        assert!(CdrReader::new(&[0x00, 0x01]).is_err());
    }

    #[test]
    fn rejects_big_endian_representation_id() {
        let data = vec![0x00, 0x00, 0x00, 0x00];
        assert!(CdrReader::new(&data).is_err());
    }

    #[test]
    fn reads_u32_and_f32_with_correct_alignment() {
        let mut data = encode_header();
        data.extend_from_slice(&42u32.to_le_bytes());
        data.extend_from_slice(&3.5f32.to_le_bytes());
        let mut r = CdrReader::new(&data).unwrap();
        assert_eq!(r.read_u32().unwrap(), 42);
        assert_eq!(r.read_f32().unwrap(), 3.5);
    }

    #[test]
    fn reads_f64_with_8_byte_alignment_after_u32() {
        let mut data = encode_header();
        data.extend_from_slice(&1u32.to_le_bytes()); // offset 0..4
        // 4 bytes of padding needed before the f64 (offset 4 -> 8)
        data.extend_from_slice(&[0u8; 4]);
        data.extend_from_slice(&2.5f64.to_le_bytes()); // offset 8..16
        let mut r = CdrReader::new(&data).unwrap();
        assert_eq!(r.read_u32().unwrap(), 1);
        assert_eq!(r.read_f64().unwrap(), 2.5);
    }

    #[test]
    fn reads_string_with_nul_terminator() {
        let mut data = encode_header();
        let s = "laser_frame";
        let len = (s.len() + 1) as u32; // CDR string length includes the NUL
        data.extend_from_slice(&len.to_le_bytes());
        data.extend_from_slice(s.as_bytes());
        data.push(0); // NUL terminator
        let mut r = CdrReader::new(&data).unwrap();
        assert_eq!(r.read_string().unwrap(), "laser_frame");
    }

    #[test]
    fn reads_f32_sequence() {
        let mut data = encode_header();
        data.extend_from_slice(&3u32.to_le_bytes());
        for v in [1.0f32, 2.0, 3.0] {
            data.extend_from_slice(&v.to_le_bytes());
        }
        let mut r = CdrReader::new(&data).unwrap();
        assert_eq!(r.read_f32_seq().unwrap(), vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn reads_u8_sequence() {
        let mut data = encode_header();
        data.extend_from_slice(&4u32.to_le_bytes());
        data.extend_from_slice(&[10, 20, 30, 40]);
        let mut r = CdrReader::new(&data).unwrap();
        assert_eq!(r.read_u8_seq().unwrap(), vec![10, 20, 30, 40]);
    }

    #[test]
    fn reads_fixed_f64_array_without_length_prefix() {
        let mut data = encode_header();
        for v in 0..9 {
            data.extend_from_slice(&(v as f64).to_le_bytes());
        }
        let mut r = CdrReader::new(&data).unwrap();
        let arr: [f64; 9] = r.read_f64_array().unwrap();
        assert_eq!(arr, [0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0]);
    }

    #[test]
    fn buffer_underrun_is_a_real_error_not_a_panic() {
        let data = encode_header();
        let mut r = CdrReader::new(&data).unwrap();
        assert!(r.read_f64().is_err());
    }
}
