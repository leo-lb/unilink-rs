use std::fs::File;
use std::io::prelude::*;
use std::net;
use std::thread;

use miniserde::{json, Deserialize, Serialize};

use crate::noise_pattern::{Noise_XXpsk3_25519_ChaChaPoly_BLAKE2s, Pattern};

use crate::messaging::{MessageReader, MessageWriter};

mod messaging;
mod noise;
mod noise_pattern;

/* #[derive(Serialize, Deserialize, Clone)]
enum Role {
    Client = 0,
    Node = 1,
    Bridge = 2,
    Master = 3,
} */

struct Peer {
    role: u8,
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
    role: u8,
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
            role: 0,
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
        Ok(mut file) => {
            let mut buf = String::new();
            file.read_to_string(&mut buf).unwrap();
            json::from_str(&buf).unwrap()
        }
        Err(_) => Config::new(),
    };

    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 {
        let peer = &args[1];

        let mut stream = net::TcpStream::connect(peer).unwrap();

        let noise = Noise_XXpsk3_25519_ChaChaPoly_BLAKE2s::new_noise(&config.keypair.private, true)
            .unwrap();

        let mut pattern = Noise_XXpsk3_25519_ChaChaPoly_BLAKE2s::new(noise).unwrap();

        pattern.initiator(&mut stream).unwrap();

        let noise = pattern.into_inner();

        let mut noise =
            crate::noise::Noise::from(noise.into_transport_mode().unwrap(), &mut stream);

        loop {
            let mut buf = String::new();
            std::io::stdin().read_line(&mut buf).unwrap();

            let message = UnilinkHeader {
                r#type: 0,
                data: Vec::new(),
            };

            noise
                .write_message(&json::to_string(&message).as_bytes())
                .unwrap();

            let message: UnilinkHeader =
                json::from_str(&String::from_utf8_lossy(&noise.read_message().unwrap())).unwrap();

            println!("{:#?}", message);
        }
    }

    let listener = net::TcpListener::bind(net::SocketAddrV6::new(
        net::Ipv6Addr::UNSPECIFIED,
        3333, /* config.port */
        0,
        0,
    ))
    .unwrap();

    let listen_address = listener.local_addr().unwrap();

    if config.port == 0 {
        config.port = listen_address.port();
    }

    println!("listening on port {}", config.port);

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

                    let mut noise = crate::noise::Noise::from(
                        noise.into_transport_mode().unwrap(),
                        &mut stream,
                    );

                    loop {
                        let message: UnilinkHeader = json::from_str(&String::from_utf8_lossy(
                            &noise.read_message().unwrap(),
                        ))
                        .unwrap();

                        println!("{:#?}", message);

                        noise
                            .write_message(&json::to_string(&message).as_bytes())
                            .unwrap();
                    }
                });
            }
            Err(e) => {
                println!("couldn't get client: {:?}", e);
            }
        }
    }
}
