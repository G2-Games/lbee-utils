pub struct BitIo {
    data: Vec<u8>,
    byte_offset: usize,
    bit_offset: usize,

    byte_size: usize,
}

impl BitIo {
    /// Create a new BitIO reader and writer over some data
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            data,
            byte_offset: 0,
            bit_offset: 0,
            byte_size: 0,
        }
    }

    /// Get the byte offset of the reader
    pub fn byte_offset(&self) -> usize {
        self.byte_offset
    }

    /// Get the byte size of the reader
    pub fn byte_size(&self) -> usize {
        self.byte_size
    }

    /// Get the current bytes up to `byte_size` in the reader
    pub fn bytes(&self) -> Vec<u8> {
        self.data[..self.byte_size].to_vec()
    }

    /// Read some bits from the buffer
    pub fn read_bit(&mut self, bit_len: usize) -> u64 {
        if bit_len > 8 * 8 {
            panic!("Cannot read more than 64 bits")
        }

        if bit_len % 8 == 0 && self.bit_offset == 0 {
            return self.read(bit_len / 8);
        }

        let mut result = 0;
        for i in 0..bit_len {
            let bit_value = ((self.data[self.byte_offset] as usize >> self.bit_offset) & 1) as u64;
            self.bit_offset += 1;

            if self.bit_offset == 8 {
                self.byte_offset += 1;
                self.bit_offset = 0;
            }

            result |= bit_value << i;
        }

        result
    }

    /// Read some bytes from the buffer
    pub fn read(&mut self, byte_len: usize) -> u64 {
        if byte_len > 8 {
            panic!("Cannot read more than 8 bytes")
        }

        let mut padded_slice = [0u8; 8];
        padded_slice.copy_from_slice(&self.data[self.byte_offset..self.byte_offset + byte_len]);
        self.byte_offset += byte_len;

        u64::from_le_bytes(padded_slice)
    }

    /// Write some bits to the buffer
    pub fn write_bit(&mut self, data: u64, bit_len: usize) {
        if bit_len > 8 * 8 {
            panic!("Cannot write more than 64 bits");
        }

        if bit_len % 8 == 0 && self.bit_offset == 0 {
            self.write(data, bit_len / 8);
            return;
        }

        for i in 0..bit_len {
            let bit_value = (data >> i) & 1;

            self.data[self.byte_offset] &= !(1 << self.bit_offset);

            self.data[self.byte_offset] |= (bit_value << self.bit_offset) as u8;

            self.bit_offset += 1;
            if self.bit_offset == 8 {
                self.byte_offset += 1;
                self.bit_offset = 0;
            }
        }

        self.byte_size = self.byte_offset + (self.bit_offset + 7) / 8;
    }

    pub fn write(&mut self, data: u64, byte_len: usize) {
        if byte_len > 8 {
            panic!("Cannot write more than 8 bytes")
        }

        let mut padded_slice = [0u8; 8];
        padded_slice.copy_from_slice(&data.to_le_bytes());

        self.data[self.byte_offset..self.byte_offset + byte_len]
            .copy_from_slice(&padded_slice[..byte_len]);
        self.byte_offset += byte_len;

        self.byte_size = self.byte_offset + (self.bit_offset + 7) / 8;
    }
}
