/// Converts 8 bit bytes to a 16 bit little endian word
pub fn bytes_to_word(first: u8, second: u8) -> u16 {
    ((second as u16) << 8) | (first as u16)
}

/// Converts a 16 bit little endian word to 8 bit bytes
pub fn word_to_bytes(word: u16) -> [u8; 2] {
    let first: u8 = (word & 0xFF) as u8; // Extract the first byte
    let second: u8 = ((word >> 8) & 0xFF) as u8; // Extract the second byte

    [first, second]
}

pub fn get_bytes<const S: usize>(iterator: &mut std::vec::IntoIter<u8>) -> [u8; S] {
    let mut bytes = [0; S];

    for byte in bytes.iter_mut().take(S) {
        *byte = iterator.next().unwrap();
    }

    bytes
}
