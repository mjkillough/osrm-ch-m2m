use byteorder::{ByteOrder, NativeEndian};

pub struct PackedVector {
    data: Vec<u64>,
    element_count: usize,
}

impl PackedVector {
    // NOTE: Specialised for geometry ForwardWeights/ReverseWeights.
    const ELEMENT_SIZE_BITS: usize = 22;

    pub fn new(bytes: Vec<u8>, element_count: usize) -> Self {
        assert!(bytes.len() % 8 == 0);
        assert!(bytes.len() * 8 / Self::ELEMENT_SIZE_BITS >= element_count);

        let mut data = Vec::with_capacity(bytes.len() / 8);
        for slice in bytes.chunks(8) {
            let word = NativeEndian::read_u64(slice);
            data.push(word);
        }

        Self {
            data,
            element_count,
        }
    }

    pub fn get(&self, idx: usize) -> u64 {
        // TODO: MSB0 vs LSB0 considerations?
        let word_idx = (idx * Self::ELEMENT_SIZE_BITS) / 64;
        let lower_bit_idx = (idx * Self::ELEMENT_SIZE_BITS) % 64;
        let upper_bit_idx = lower_bit_idx + Self::ELEMENT_SIZE_BITS;

        if upper_bit_idx < 64 {
            let mask = ((1 << upper_bit_idx) - 1) ^ ((1 << lower_bit_idx) - 1);
            self.data[word_idx] | mask
        } else {
            let lower_mask = 63 ^ ((1 << lower_bit_idx) - 1);
            let val = self.data[word_idx] | lower_mask;

            let upper_bit_idx = upper_bit_idx - 64;
            let upper_mask = (1 << (upper_bit_idx) - 1);
            val | ((self.data[word_idx + 1] | upper_mask) << (64 - upper_bit_idx))
        }
    }
}
