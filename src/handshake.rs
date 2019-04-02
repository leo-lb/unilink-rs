use std::io::{Read, Write};

use snow::params::NoiseParams;
use snow::Session;

use crate::noise_pattern::{Noise_XXpsk3_25519_ChaChaPoly_BLAKE2s, Pattern};

pub struct Handshake<S>
    where
        S: Read + Write,
{
    stream: S,
    id: Option<u8>,
}

impl<S> Handshake<S>
    where
        S: Read + Write,
{
    pub fn new(stream: S) -> Self {
        Self { stream, id: None }
    }

    pub fn id(&mut self) -> std::io::Result<u8> {
        match self.id {
            Some(id) => Ok(id),
            None => {
                let mut id = [0u8; 1];

                self.stream.read_exact(&mut id)?;

                self.id = Some(id[0]);
                Ok(id[0])
            }
        }
    }

    pub fn handshake<P: Pattern>(&mut self, initiator: bool, pattern: &mut P) -> Result<(), ()> {
        if self.id.is_none() {
            return Err(());
        }

        if pattern.inst_type() != self.id.unwrap() {
            return Err(());
        }

        match initiator {
            true => pattern.initiator(&mut self.stream)?,
            false => pattern.responder(&mut self.stream)?,
        };

        Ok(())
    }

    pub fn noise_params(&self) -> Result<NoiseParams, ()> {
        if self.id.is_none() {
            return Err(());
        }

        match self.id.unwrap() {
            0 => Noise_XXpsk3_25519_ChaChaPoly_BLAKE2s::pattern(),
            _ => {
                return Err(());
            }
        }
            .parse()
            .map_err(|_| {})
    }
}
