// Typing lecture: `repr(C)` and byte-oriented systems data.

use std::mem::{align_of, size_of};

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PacketHeader {
    version: u8,
    flags: u8,
    length: u16,
}

impl PacketHeader {
    fn from_le_bytes(bytes: [u8; 4]) -> Self {
        Self {
            version: bytes[0],
            flags: bytes[1],
            length: u16::from_le_bytes([bytes[2], bytes[3]]),
        }
    }

    fn to_le_bytes(self) -> [u8; 4] {
        let length = self.length.to_le_bytes();
        [self.version, self.flags, length[0], length[1]]
    }
}

fn payload_len(packet: &[u8]) -> Option<u16> {
    let header = packet.get(..4)?;
    Some(PacketHeader::from_le_bytes([header[0], header[1], header[2], header[3]]).length)
}

fn main() {
    println!(
        "{} {}",
        size_of::<PacketHeader>(),
        align_of::<PacketHeader>()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_expected_c_layout_size() {
        assert_eq!(size_of::<PacketHeader>(), 4);
        assert_eq!(align_of::<PacketHeader>(), 2);
    }

    #[test]
    fn parses_and_serializes_header() {
        let header = PacketHeader::from_le_bytes([1, 0b1010, 44, 1]);

        assert_eq!(
            header,
            PacketHeader {
                version: 1,
                flags: 0b1010,
                length: 300,
            }
        );
        assert_eq!(header.to_le_bytes(), [1, 0b1010, 44, 1]);
    }

    #[test]
    fn reads_payload_length_from_packet() {
        assert_eq!(payload_len(&[2, 0, 5, 0, 9, 9, 9]), Some(5));
        assert_eq!(payload_len(&[2, 0, 5]), None);
    }
}
