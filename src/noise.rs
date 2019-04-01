use std::io::{self, Read, Write};

use snow::params::NoiseParams;
use snow::Session;

use crate::messaging::{MessageReader, MessageWriter};
use crate::noise_pattern::{Noise_XXpsk3_25519_ChaChaPoly_BLAKE2s, Pattern};

pub struct Noise<S: MessageReader + MessageWriter> {
    stream: S,
    noise: Option<Session>,
}

impl<S> Noise<S>
where
    S: MessageReader + MessageWriter + Read + Write,
{
    pub fn from(stream: S) -> Self {
        Self {
            stream,
            noise: None,
        }
    }

    pub fn do_handshake(&mut self, initiator: bool) -> Result<(), ()> {
        let mut pattern_id = [0u8; 1];
        self.stream.read_exact(&mut pattern_id).map_err(|_| {})?;

        let pattern_id = pattern_id[0];

        match pattern_id {
            0 => {
                // Noise_XXpsk3_25519_ChaChaPoly_BLAKE2s
                let noise = snow::Builder::new(
                    Noise_XXpsk3_25519_ChaChaPoly_BLAKE2s::pattern()
                        .parse()
                        .unwrap(),
                )
                .psk(3, b"This is a private static key");

                let noise = match initiator {
                    true => noise.build_initiator().unwrap(),
                    false => noise.build_responder().unwrap(),
                };

                let mut handshaker = Noise_XXpsk3_25519_ChaChaPoly_BLAKE2s::new(noise).unwrap();

                match initiator {
                    true => handshaker.initiator(&mut self.stream),
                    false => handshaker.responder(&mut self.stream),
                }
                .map_err(|_| {})?;

                self.noise = Some(
                    handshaker
                        .into_inner()
                        .into_transport_mode()
                        .map_err(|_| {})?,
                );
            }
            _ => {
                return Err(());
            }
        };

        Ok(())
    }

    pub fn read_decrypt(&mut self) -> Result<Vec<u8>, ()> {
        let mut ret_buf = Vec::new();

        let message = self.stream.read_message()?;
        for message in message.chunks(65535) {
            let mut buf = Vec::new();

            let len: usize;
            match self.noise.as_mut() {
                Some(noise) => {
                    len = noise.read_message(&message, &mut buf).map_err(|_| {})?;
                }
                None => {
                    return Err(());
                }
            };

            ret_buf.append(&mut buf[..len].to_vec());
        }

        Ok(ret_buf)
    }

    pub fn write_encrypt(&mut self, message: &mut [u8]) -> Result<(), ()> {
        let mut send_buf = Vec::new();

        for message in message.chunks(65535) {
            let mut buf = Vec::new();

            let len: usize;
            match self.noise.as_mut() {
                Some(noise) => {
                    len = noise.write_message(message, &mut buf).map_err(|_| {})?;
                }
                None => {
                    return Err(());
                }
            }

            send_buf.append(&mut buf[..len].to_vec());
        }

        self.stream.write_message(&send_buf).map_err(|_| {})?;

        Ok(())
    }
}
