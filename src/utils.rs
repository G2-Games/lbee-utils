pub mod util {
    /// Converts 8 bit bytes to a 16 bit little endian word or
    pub fn bytes_to_word(first:u8, second:u8) -> i16 {
        let final_value = ((second as i16) << 8) | (first as i16);

        return final_value;
    }

    /// Converts a 16 bit little endian word to 8 bit bytes
    pub fn word_to_bytes(word:i16) -> [u8; 2] {
        let first: u8 = (word & 0xFF) as u8; // Extract the first byte
        let second: u8 = ((word >> 8) & 0xFF) as u8; // Extract the second byte

        return [first, second];
    }
}
