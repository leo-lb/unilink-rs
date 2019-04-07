use std::error::Error;
use std::io::{Read, Write};

use snow::{Session, Session::Transport};

use crate::message::{MessageReader, MessageWriter};

pub struct Noise<S: MessageReader + MessageWriter> {
    stream: S,
    noise: Session,
}

impl<S> Noise<S>
where
    S: Read + Write,
{
    pub fn from(noise: Session, stream: S) -> Self {
        let noise = match noise {
            Transport(transport_state) => Transport(transport_state),
            _ => panic!("Session should be in transport mode"),
        };

        Self { stream, noise }
    }
}

impl<S> MessageReader for Noise<S>
where
    S: Read + Write,
{
    fn read_message(&mut self) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut ret_buf = Vec::new();

        let message = self.stream.read_message()?;
        for message in message.chunks(65535) {
            let mut buf = vec![0u8; 65535];

            let len = self.noise.read_message(&message, &mut buf)?;

            ret_buf.append(&mut buf[..len].to_vec());
        }

        Ok(ret_buf)
    }
}

impl<S> MessageWriter for Noise<S>
where
    S: Read + Write,
{
    fn write_message(&mut self, message: &[u8]) -> Result<(), Box<dyn Error>> {
        let mut send_buf = Vec::new();

        for message in message.chunks(65535) {
            let mut buf = vec![0u8; 65535];

            let len = self.noise.write_message(message, &mut buf)?;

            send_buf.append(&mut buf[..len].to_vec());
        }

        self.stream.write_message(&send_buf)?;

        Ok(())
    }
}
