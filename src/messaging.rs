use std::io::{Read, Write};



pub trait MessageWriter {
    fn write_message(&mut self, message: &[u8]) -> Result<(), ()>;
}

pub trait MessageReader {
    fn read_message(&mut self) -> Result<Vec<u8>, ()>;
}

impl<T: Read> MessageReader for T {
    fn read_message(&mut self) -> Result<Vec<u8>, ()> {
        let mut len_buf = [0u8; 4];

        self.read_exact(&mut len_buf).map_err(|_| {})?;

        let mut buf = vec![0u8; u32::from_be_bytes(len_buf) as usize];

        self.read_exact(&mut buf).map_err(|_| {})?;

        Ok(buf)
    }
}

impl<T: Write> MessageWriter for T {
    // WARNING!! Cannot write more than 2^32-1 bytes at once
    fn write_message(&mut self, message: &[u8]) -> Result<(), ()> {
        if message.len() >= std::u32::MAX as usize {
            return Err(());
        }

        let len = message.len() as u32;

        let len_buf = len.to_be_bytes();

        self.write_all(&len_buf).map_err(|_| {})?;
        self.write_all(message).map_err(|_| {})?;

        Ok(())
    }
}
