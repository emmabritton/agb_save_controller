use crate::GbaSave;

pub const HEADER_LEN: usize = 4 + 1 + 4 + 4 + 4; // 17
const MAGIC: u32 = 0x53415645;

pub struct SlotHeader {
    magic: u32,
    version: u8,
    generation: u32,
    payload_len: u32, // == T::SIZE
    crc32: u32,
}

impl SlotHeader {
    pub fn new(version: u8, generation: u32, payload_len: u32, crc32: u32) -> Self {
        Self {
            magic: MAGIC,
            version,
            generation,
            payload_len,
            crc32,
        }
    }
    
    pub fn decode_from(bytes: &[u8]) -> SlotHeader {
        let magic = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        let version = bytes[4];
        let generation = u32::from_le_bytes([bytes[5], bytes[6], bytes[7], bytes[8]]);
        let payload_len = u32::from_le_bytes([bytes[9], bytes[10], bytes[11], bytes[12]]);
        let crc32 = u32::from_le_bytes([bytes[13], bytes[14], bytes[15], bytes[16]]);
        SlotHeader {
            magic,
            version,
            generation,
            payload_len,
            crc32,
        }
    }
}

impl SlotHeader {
    pub fn is_valid<T: GbaSave>(&self, expected_version: u8) -> bool {
        self.magic == MAGIC && self.version == expected_version && self.payload_len as usize == T::SIZE
    }
    
    pub fn encode_into(&self, out: &mut [u8]) {
        let magic = self.magic.to_le_bytes();
        out[0] = magic[0];
        out[1] = magic[1];
        out[2] = magic[2];
        out[3] = magic[3];

        out[4] = self.version;

        let generation = self.generation.to_le_bytes();
        out[5] = generation[0];
        out[6] = generation[1];
        out[7] = generation[2];
        out[8] = generation[3];

        let plen = self.payload_len.to_le_bytes();
        out[9] = plen[0];
        out[10] = plen[1];
        out[11] = plen[2];
        out[12] = plen[3];

        let crc = self.crc32.to_le_bytes();
        out[13] = crc[0];
        out[14] = crc[1];
        out[15] = crc[2];
        out[16] = crc[3];
    }
    
    #[inline(always)]
    pub fn generation(&self) -> u32 {
        self.generation
    }

    #[inline(always)]
    pub fn crc32(&self) -> u32 {
        self.crc32
    }
}
