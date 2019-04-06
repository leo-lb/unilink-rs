use std::collections::HashMap;
use std::fs::File;
use std::net;
use std::thread;

use serde_derive::{Deserialize, Serialize};

use crate::noise_pattern::{Noise_XXpsk3_25519_ChaChaPoly_BLAKE2s, Pattern};

use crate::messaging::{MessageReader, MessageWriter};

mod messaging;
mod noise;
mod noise_pattern;

#[derive(Serialize, Deserialize, Clone)]
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

#[derive(Serialize, Deserialize, Clone)]
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

#[derive(Serialize, Deserialize, Clone)]
struct Config {
    role: Role,
    port: u16,
    keypair: Keypair,
}

impl Config {
    fn new() -> Self {
        let noise = snow::Builder::new(
            Noise_XXpsk3_25519_ChaChaPoly_BLAKE2s::pattern()
                .parse()
                .unwrap(),
        );

        Config {
            role: Role::Client,
            port: 0,
            keypair: noise.generate_keypair().unwrap().into(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct UnilinkHeader {
    r#type: u16,
    data: Vec<u8>,
}

const CBOR_DATABASE: &str = "store.cbor";

fn main() {
    let mut config: Config = match File::open(CBOR_DATABASE) {
        Ok(file) => serde_cbor::from_reader(file).unwrap(),
        Err(_) => Config::new(),
    };

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
            Ok((mut stream, address)) => {
                println!("new client: {:?}", address);

                let config = config.clone();

                thread::spawn(move || {
                    let noise = Noise_XXpsk3_25519_ChaChaPoly_BLAKE2s::new_noise(
                        &config.keypair.private,
                        false,
                    )
                    .unwrap();
                    let mut pattern = Noise_XXpsk3_25519_ChaChaPoly_BLAKE2s::new(noise).unwrap();

                    pattern.responder(&mut stream).unwrap();

                    let noise = pattern.into_inner();

                    let mut noise = crate::noise::Noise::from(noise, &mut stream);

                    loop {
                        let message: UnilinkHeader =
                            serde_cbor::from_slice(&noise.read_message().unwrap()).unwrap();

                        println!("{:#?}", message);
                    }
                });
            }
            Err(e) => {
                println!("couldn't get client: {:?}", e);
            }
        }
    }
}
