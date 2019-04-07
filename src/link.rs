use crate::message::{MessageReader, MessageWriter};
use crate::noise::Noise;
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::mpsc::{Receiver, Sender};
use std::thread::Thread;

pub struct Link
{
    pub thread: Thread,
    pub send: Sender<Vec<u8>>,
    pub recv: Receiver<Vec<u8>>,
    pub tagged_io: HashMap<u8, (Sender<Vec<u8>>, Receiver<Vec<u8>>)>,
}
