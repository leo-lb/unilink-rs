use crate::messaging::{MessageReader, MessageWriter};
use snow::Session;

pub trait Pattern<S>
where
    S: MessageReader + MessageWriter,
    Self: std::marker::Sized,
{
    fn from(stream: S, noise: Session) -> Result<Self, ()>;
    fn r#type() -> u8;
    fn pattern() -> &'static str;
    fn initiator(&mut self) -> Result<(), ()>;
    fn responder(&mut self) -> Result<(), ()>;
}

#[allow(non_camel_case_types)]
struct Noise_XXpsk3_25519_ChaChaPoly_BLAKE2s<S: MessageReader + MessageWriter> {
    stream: S,
    noise: Session,
}

impl<S> Pattern<S> for Noise_XXpsk3_25519_ChaChaPoly_BLAKE2s<S>
where
    S: MessageReader + MessageWriter,
{
    fn from(stream: S, noise: Session) -> Result<Self, ()> {
        match noise.is_handshake_finished() {
            true => Err(()),
            false => Ok(Self { stream, noise }),
        }
    }

    fn r#type() -> u8 {
        0 // Must be unique for each Pattern impl
    }

    fn pattern() -> &'static str {
        "Noise_XXpsk3_25519_ChaChaPoly_BLAKE2s"
    }

    fn initiator(&mut self) -> Result<(), ()> {
        if !self.noise.is_initiator() {
            return Err(());
        }

        let mut buf: Vec<u8> = Vec::new();

        let len = self.noise.write_message(&[], &mut buf).map_err(|_| {})?;

        self.stream.write_message(&buf[..len]).map_err(|_| {})?;

        self.noise
            .read_message(&self.stream.read_message().map_err(|_| {})?, &mut buf)
            .map_err(|_| {})?;

        let len = self.noise.write_message(&[], &mut buf).map_err(|_| {})?;

        self.stream.write_message(&buf[..len]).map_err(|_| {})?;

        Ok(())
    }

    fn responder(&mut self) -> Result<(), ()> {
        if self.noise.is_initiator() {
            return Err(());
        }

        let mut buf: Vec<u8> = Vec::new();

        self.noise
            .read_message(&self.stream.read_message().map_err(|_| {})?, &mut buf)
            .map_err(|_| {})?;

        let len = self.noise.write_message(&[], &mut buf).map_err(|_| {})?;

        self.stream.write_message(&buf[..len]).map_err(|_| {})?;


        self.noise
            .read_message(&self.stream.read_message().map_err(|_| {})?, &mut buf)
            .map_err(|_| {})?;

        Ok(())
    }
}
