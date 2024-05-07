pub struct BitIO {
    data: Vec<u8>,
    byte_offset: usize,
    bit_offset: usize,
}

impl BitIO {
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            data,
            byte_offset: 0,
            bit_offset: 0,
        }
    }

    pub fn byte_offset(&self) -> usize {
        self.byte_offset
    }

    pub fn bit_offset(&self) -> usize {
        self.bit_offset
    }

    pub fn into_vec(self) -> Vec<u8> {
        self.data
    }

    pub fn read_bit(&mut self, bit_len: usize) -> u64 {
        //print!("{}: ", bit_len);
        if bit_len > 8 * 8 {
            panic!()
        }

        if bit_len % 8 == 0 && self.bit_offset == 0 {
            return self.read(bit_len / 8);
        }

        let mut result = 0;
        for i in 0..bit_len {
            let bit_value =
                ((self.data[self.byte_offset] as usize >> self.bit_offset) & 1) as u64;
            self.bit_offset += 1;

            if self.bit_offset == 8 {
                self.byte_offset += 1;
                self.bit_offset = 0;
            }

            result |= bit_value << i;
        }

        result
    }

    pub fn read(&mut self, byte_len: usize) -> u64 {
        if byte_len > 8 {
            panic!()
        }

        let mut padded_slice = [0u8; 8];
        padded_slice.copy_from_slice(&self.data[self.byte_offset..self.byte_offset + byte_len]);
        self.byte_offset += byte_len;

        u64::from_le_bytes(padded_slice)
    }
}
