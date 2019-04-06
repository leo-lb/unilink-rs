use snow::Session;

use crate::messaging::{MessageReader, MessageWriter};

pub trait Pattern
where
    Self: std::marker::Sized,
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

    fn new_noise(private: &[u8], initiator: bool) -> Result<Session, ()> {
        let b = snow::Builder::new(Self::pattern().parse().unwrap())
            .local_private_key(private)
            .psk(3, b"01234567890123456789012345678901");

        match initiator {
            true => b.build_initiator().map_err(|e| {
                eprintln!("{:#?}", e);
            }),
            false => b.build_responder().map_err(|e| {
                eprintln!("{:#?}", e);
            }),
        }
    }

    fn initiator<S: MessageReader + MessageWriter>(&mut self, stream: &mut S) -> Result<(), ()> {
        if !self.noise.is_initiator() {
            return Err(());
        }

        let mut buf = vec![0u8; 65535];

        let len = self.noise.write_message(&[], &mut buf).map_err(|e| {
            eprintln!("{:#?}", e);
        })?;

        stream.write_message(&buf[..len]).map_err(|e| {
            eprintln!("{:#?}", e);
        })?;

        self.noise
            .read_message(
                &stream.read_message().map_err(|e| {
                    eprintln!("{:#?}", e);
                })?,
                &mut buf,
            )
            .map_err(|e| {
                eprintln!("{:#?}", e);
            })?;

        let len = self.noise.write_message(&[], &mut buf).map_err(|e| {
            eprintln!("{:#?}", e);
        })?;

        stream.write_message(&buf[..len]).map_err(|e| {
            eprintln!("{:#?}", e);
        })?;

        Ok(())
    }

    fn responder<S: MessageReader + MessageWriter>(&mut self, stream: &mut S) -> Result<(), ()> {
        if self.noise.is_initiator() {
            return Err(());
        }

        let mut buf = vec![0u8; 65535];

        self.noise
            .read_message(
                &stream.read_message().map_err(|e| {
                    eprintln!("{:#?}", e);
                })?,
                &mut buf,
            )
            .map_err(|e| {
                eprintln!("{:#?}", e);
            })?;

        let len = self.noise.write_message(&[], &mut buf).map_err(|e| {
            eprintln!("{:#?}", e);
        })?;

        stream.write_message(&buf[..len]).map_err(|e| {
            eprintln!("{:#?}", e);
        })?;

        self.noise
            .read_message(
                &stream.read_message().map_err(|e| {
                    eprintln!("{:#?}", e);
                })?,
                &mut buf,
            )
            .map_err(|e| {
                eprintln!("{:#?}", e);
            })?;

        Ok(())
    }

    fn into_inner(self) -> Session {
        self.noise
    }
}
