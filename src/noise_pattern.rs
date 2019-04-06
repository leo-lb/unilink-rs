use snow::{Session, params::NoiseParams};

use crate::messaging::{MessageReader, MessageWriter};

pub trait Pattern
where Self: std::marker::Sized
{
    fn new(noise: Session) -> Result<Self, ()>;
    fn r#type() -> u8;
    fn pattern() -> &'static str;
    fn inst_type(&self) -> u8;
    fn inst_pattern(&self) -> &'static str;
    fn new_noise(private: &[u8], initiator: bool) -> Result<Session, ()>;
    fn initiator<S: MessageReader + MessageWriter>(&mut self, stream: &mut S) -> Result<(), ()>;
    fn responder<S: MessageReader + MessageWriter>(&mut self, stream: &mut S) -> Result<(), ()>;
    fn into_inner(self) -> Session;
}

#[allow(non_camel_case_types)]
pub struct Noise_XXpsk3_25519_ChaChaPoly_BLAKE2s {
    noise: Session,
}

impl Pattern for Noise_XXpsk3_25519_ChaChaPoly_BLAKE2s {
    fn new(noise: Session) -> Result<Self, ()> {
        match noise.is_handshake_finished() {
            true => Err(()),
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

    fn new_noise(private: &[u8], initiator: bool) -> Result<Session, ()>{
        let b = snow::Builder::new(Self::pattern().parse().unwrap())
        .local_private_key(private)
        .psk(3, b"Test PSK");

        match initiator {
            true => b.build_initiator().map_err(|_| {}),
            false => b.build_responder().map_err(|_| {})
        }
    }

    fn initiator<S: MessageReader + MessageWriter>(&mut self, stream: &mut S) -> Result<(), ()> {
        if !self.noise.is_initiator() {
            return Err(());
        }

        let mut buf: Vec<u8> = Vec::new();

        let len = self.noise.write_message(&[], &mut buf).map_err(|_| {})?;

        stream.write_message(&buf[..len]).map_err(|_| {})?;

        self.noise
            .read_message(&stream.read_message().map_err(|_| {})?, &mut buf)
            .map_err(|_| {})?;

        let len = self.noise.write_message(&[], &mut buf).map_err(|_| {})?;

        stream.write_message(&buf[..len]).map_err(|_| {})?;

        Ok(())
    }

    fn responder<S: MessageReader + MessageWriter>(&mut self, stream: &mut S) -> Result<(), ()> {
        if self.noise.is_initiator() {
            return Err(());
        }

        let mut buf: Vec<u8> = Vec::new();

        self.noise
            .read_message(&stream.read_message().map_err(|_| {})?, &mut buf)
            .map_err(|_| {})?;

        let len = self.noise.write_message(&[], &mut buf).map_err(|_| {})?;

        stream.write_message(&buf[..len]).map_err(|_| {})?;

        self.noise
            .read_message(&stream.read_message().map_err(|_| {})?, &mut buf)
            .map_err(|_| {})?;

        Ok(())
    }

    fn into_inner(self) -> Session {
        self.noise
    }
}