use std::fmt;

#[derive(Debug)]
pub enum Error {
    Message(MessageError),
    Pattern(PatternError),

    #[doc(hidden)]
    __Nonexhaustive
}

#[derive(Debug)]
pub enum MessageError {
    MessageTooLarge
}

impl From<MessageError> for Error {
    fn from(reason: MessageError) -> Self {
        Error::Message(reason)
    }
}

#[derive(Debug)]
pub enum PatternError {
    HandshakeAlreadyFinished,
    ShouldBeInitiator,
    ShouldBeResponder,
}

impl From<PatternError> for Error {
    fn from(reason: PatternError) -> Self {
        Error::Pattern(reason)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Error::*;

        match self {
            Message(reason) => write!(f, "{:?}", reason),
            Pattern(reason) => write!(f, "{:?}", reason),
            __Nonexhaustive => write!(f, "Nonexhaustive"),
        }
    }
}

impl std::error::Error for Error {}