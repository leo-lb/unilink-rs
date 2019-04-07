use snow::Session;
use std::error::Error as StdError;

use crate::error::{Error, PatternError::*};
use crate::message::{MessageReader, MessageWriter};

pub trait Pattern
where
    Self: std::marker::Sized,
{
    fn new(noise: Session) -> Result<Self, Box<dyn StdError>>;
    fn r#type() -> u8;
    fn pattern() -> &'static str;
    fn inst_type(&self) -> u8;
    fn inst_pattern(&self) -> &'static str;
    fn new_noise(
        private: &[u8],
        psk: &[u8; 32],
        initiator: bool,
    ) -> Result<Session, Box<dyn StdError>>;
    fn initiator<S: MessageReader + MessageWriter>(
        &mut self,
        stream: &mut S,
    ) -> Result<(), Box<dyn StdError>>;
    fn responder<S: MessageReader + MessageWriter>(
        &mut self,
        stream: &mut S,
    ) -> Result<(), Box<dyn StdError>>;
    fn into_inner(self) -> Session;
}

#[allow(non_camel_case_types)]
pub struct Noise_XXpsk3_25519_ChaChaPoly_BLAKE2s {
    noise: Session,
}

impl Pattern for Noise_XXpsk3_25519_ChaChaPoly_BLAKE2s {
    fn new(noise: Session) -> Result<Self, Box<dyn StdError>> {
        match noise.is_handshake_finished() {
            true => Err(Error::from(HandshakeAlreadyFinished).into()),
            false => Ok(Self { noise }),
        }
    }

    fn r#type() -> u8 {
        0 // Must be unique for each Pattern impl
    }

    fn pattern() -> &'static str {
        "Noise_XXpsk3_25519_ChaChaPoly_BLAKE2s"
    }

    fn inst_type(&self) -> u8 {
        Self::r#type()
    }

    fn inst_pattern(&self) -> &'static str {
        Self::pattern()
    }

    fn new_noise(
        private: &[u8],
        psk: &[u8; 32],
        initiator: bool,
    ) -> Result<Session, Box<dyn StdError>> {
        let b = snow::Builder::new(Self::pattern().parse().unwrap())
            .local_private_key(private)
            .psk(3, psk);

        Ok(match initiator {
            true => b.build_initiator()?,
            false => b.build_responder()?,
        })
    }

    fn initiator<S: MessageReader + MessageWriter>(
        &mut self,
        stream: &mut S,
    ) -> Result<(), Box<dyn StdError>> {
        if !self.noise.is_initiator() {
            return Err(Error::from(ShouldBeInitiator).into());
        }

        let mut buf = vec![0u8; 65535];

        let len = self.noise.write_message(&[], &mut buf)?;

        stream.write_message(&buf[..len])?;

        self.noise.read_message(&stream.read_message()?, &mut buf)?;

        let len = self.noise.write_message(&[], &mut buf)?;

        stream.write_message(&buf[..len])?;

        Ok(())
    }

    fn responder<S: MessageReader + MessageWriter>(
        &mut self,
        stream: &mut S,
    ) -> Result<(), Box<dyn StdError>> {
        if self.noise.is_initiator() {
            return Err(Error::from(ShouldBeResponder).into());
        }

        let mut buf = vec![0u8; 65535];

        self.noise.read_message(&stream.read_message()?, &mut buf)?;

        let len = self.noise.write_message(&[], &mut buf)?;

        stream.write_message(&buf[..len])?;

        self.noise.read_message(&stream.read_message()?, &mut buf)?;

        Ok(())
    }

    fn into_inner(self) -> Session {
        self.noise
    }
}