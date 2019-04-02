use std::collections::HashMap;
use std::fs::File;
use std::net;
use std::thread;

use serde_derive::{Deserialize, Serialize};
use snow::Session;

use crate::noise_pattern::{Noise_XXpsk3_25519_ChaChaPoly_BLAKE2s, Pattern};

mod handshake;
mod messaging;
mod noise;
mod noise_pattern;

#[derive(Serialize, Deserialize)]
enum Role {
    Client = 0,
    Node = 1,
    Bridge = 2,
    Master = 3,
}

struct Peer {
    role: Role,
    address: net::IpAddr,
    connection: Option<(net::TcpStream, net::SocketAddr)>,
    port: u16,
    public_key: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
struct Keypair {
    public: Vec<u8>,
    private: Vec<u8>,
}

impl Into<snow::Keypair> for Keypair {
    fn into(self) -> snow::Keypair {
        snow::Keypair {
            public: self.public.clone(),
            private: self.private.clone(),
        }
    }
}

impl From<snow::Keypair> for Keypair {
    fn from(keypair: snow::Keypair) -> Self {
        Self {
            public: keypair.public,
            private: keypair.private,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Config {
    role: Role,
    port: u16,

    /// A key for each u8 Noise pattern types
    keys: HashMap<u8, Keypair>,
}

impl Config {
    fn new() -> Self {
        Config {
            role: Role::Client,
            port: 0,
            keys: HashMap::new(),
        }
    }

    fn generate_missing_keys(&mut self) -> Result<(), ()> {
        for t in 0..std::u8::MAX {
            match t {
                0 if !self.keys.contains_key(&(0 as u8)) => {
                    let keypair = snow::Builder::new(
                        Noise_XXpsk3_25519_ChaChaPoly_BLAKE2s::pattern()
                            .parse()
                            .map_err(|_| {})?,
                    )
                        .generate_keypair()
                        .map_err(|_| {})?;

                    self.keys.insert(0 as u8, keypair.into());
                }
                _ => {}
            };
        }
        Ok(())
    }
}

const CBOR_DATABASE: &str = "store.cbor";

fn main() {
    let mut config: Config = match File::open(CBOR_DATABASE) {
        Ok(file) => serde_cbor::from_reader(file).unwrap(),
        Err(_) => Config::new(),
    };

    config.generate_missing_keys().unwrap();

    let listener = net::TcpListener::bind(net::SocketAddrV6::new(
        net::Ipv6Addr::UNSPECIFIED,
        config.port,
        0,
        0,
    ))
    .unwrap();

    let listen_address = listener.local_addr().unwrap();

    if config.port == 0 {
        config.port = listen_address.port();
    }

    loop {
        match listener.accept() {
            Ok((stream, address)) => {
                println!("new client: {:?}", address);

                thread::spawn(move || {});
            }
            Err(e) => {
                println!("couldn't get client: {:?}", e);
            }
        }
    }
}
