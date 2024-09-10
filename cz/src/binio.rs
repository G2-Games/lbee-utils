use std::io::{self, Read, Write};

use byteorder::{ReadBytesExt, WriteBytesExt};

/// A simple way to write individual bits to an input implementing [Write].
pub struct BitWriter<'a, O: Write + WriteBytesExt> {
    output: &'a mut O,

    current_byte: u8,

    byte_offset: usize,
    bit_offset: usize,

    byte_size: usize,
}

impl<'a, O: Write + WriteBytesExt> BitWriter<'a, O> {
    /// Create a new BitWriter wrapper around something which
    /// implements [Write].
    pub fn new(output: &'a mut O) -> Self {
        Self {
            output,

            current_byte: 0,

            byte_offset: 0,
            bit_offset: 0,

            byte_size: 0,
        }
    }

    /// Get the number of whole bytes written to the stream.
    pub fn byte_size(&self) -> usize {
        self.byte_size
    }

    /// Get the bit offset within the current byte.
    pub fn bit_offset(&self) -> u8 {
        self.bit_offset as u8
    }

    /// Check if the stream is aligned to a byte.
    pub fn aligned(&self) -> bool {
        self.bit_offset() == 0
    }

    /// Align the writer to the nearest byte by padding with zero bits.
    ///
    /// Returns the number of zero bits
    pub fn flush(&mut self) -> Result<usize, io::Error> {
        self.byte_offset += 1;

        // Write out the current byte unfinished
        self.output.write_u8(self.current_byte).unwrap();
        self.current_byte = 0;
        self.bit_offset = 0;

        Ok(8 - self.bit_offset)
    }

    /// Write some bits to the output.
    pub fn write_bit(&mut self, data: u64, bit_len: usize) {
        if bit_len > 64 {
            panic!("Cannot write more than 64 bits at once.");
        } else if bit_len == 0 {
            panic!("Must write 1 or more bits.")
        }

        if bit_len % 8 == 0 && self.bit_offset == 0 {
            self.write(data, bit_len / 8);
            return;
        }

        for i in 0..bit_len {
            let bit_value = (data >> i) & 1;

            self.current_byte &= !(1 << self.bit_offset);

            self.current_byte |= (bit_value << self.bit_offset) as u8;

            self.bit_offset += 1;
            if self.bit_offset >= 8 {
                self.byte_offset += 1;
                self.bit_offset = 0;

                self.output.write_u8(self.current_byte).unwrap();
                self.current_byte = 0;
            }
        }

        self.byte_size = self.byte_offset + (self.bit_offset + 7) / 8;
    }

    /// Write some bytes to the output.
    pub fn write(&mut self, data: u64, byte_len: usize) {
        if byte_len > 8 {
            panic!("Cannot write more than 8 bytes at once.")
        } else if byte_len == 0 {
            panic!("Must write 1 or more bytes.")
        }

        self.output
        .write_all(&data.to_le_bytes()[..byte_len])
        .unwrap();
        self.byte_offset += byte_len;

        self.byte_size = self.byte_offset + (self.bit_offset + 7) / 8;
    }
}

/// A simple way to read individual bits from an input implementing [Read].
pub struct BitReader<'a, I: Read + ReadBytesExt> {
    input: &'a mut I,

    current_byte: Option<u8>,

    byte_offset: usize,
    bit_offset: usize,
}

impl<'a, I: Read + ReadBytesExt> BitReader<'a, I> {
    /// Create a new BitReader wrapper around something which
    /// implements [Write].
    pub fn new(input: &'a mut I) -> Self {
        let first = input.read_u8().unwrap();
        Self {
            input,

            current_byte: Some(first),

            byte_offset: 0,
            bit_offset: 0,
        }
    }

    /// Get the number of whole bytes read from the stream.
    pub fn byte_offset(&self) -> usize {
        self.byte_offset
    }

    /// Read some bits from the input.
    pub fn read_bit(&mut self, bit_len: usize) -> u64 {
        if bit_len > 64 {
            panic!("Cannot read more than 64 bits at once.")
        } else if bit_len == 0 {
            panic!("Must read 1 or more bits.")
        }

        if bit_len % 8 == 0 && self.bit_offset == 0 {
            return self.read(bit_len / 8);
        }

        let mut result = 0;
        for i in 0..bit_len {
            let bit_value = ((self.current_byte.unwrap() as usize >> self.bit_offset) & 1) as u64;
            self.bit_offset += 1;

            if self.bit_offset == 8 {
                self.byte_offset += 1;
                self.bit_offset = 0;

                self.current_byte = Some(self.input.read_u8().unwrap());
            }

            result |= bit_value << i;
        }

        result
    }

    /// Read some bytes from the input.
    pub fn read(&mut self, byte_len: usize) -> u64 {
        if byte_len > 8 {
            panic!("Cannot read more than 8 bytes at once.")
        } else if byte_len == 0 {
            panic!("Must read 1 or more bytes")
        }

        let mut padded_slice = vec![0u8; byte_len];
        self.input.read_exact(&mut padded_slice).unwrap();
        self.byte_offset += byte_len;

        let extra_length = padded_slice.len() - byte_len;
        padded_slice.extend_from_slice(&vec![0u8; extra_length]);

        u64::from_le_bytes(padded_slice.try_into().unwrap())
    }
}
