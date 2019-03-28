use snow::params::NoiseParams;
use std::io::{Read, Write};

pub struct Noise<S: Read + Write> {
    stream: S,
}

impl<S> Noise<S>
where
    S: Read + Write,
{
    pub fn from(stream: S) -> Self {
        Self {
            stream,
        }
    }
}
