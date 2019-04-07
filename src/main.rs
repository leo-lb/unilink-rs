use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::net;
use std::sync::{Arc, Mutex};
use std::thread;

use miniserde::{json, Deserialize, Serialize};

use crate::noise_pattern::{Noise_XXpsk3_25519_ChaChaPoly_BLAKE2s, Pattern};

use crate::message::{MessageReader, MessageWriter};

mod commands;
mod error;
mod link;
mod message;
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
    #[serde(rename = "w")]
    way: bool,

    #[serde(rename = "t")]
    tag: u8,

    #[serde(rename = "k")]
    kind: u16,

    #[serde(rename = "d")]
    data: String,
}

const JSON_DATABASE: &str = "store.json";
const PSK: &[u8; 32] = b"01234567890123456798012345678901";

fn main() {
    let mut config: Config = match File::open(JSON_DATABASE) {
        Ok(mut file) => {
            let mut buf = String::new();
            file.read_to_string(&mut buf).unwrap();
            json::from_str(&buf).unwrap()
        }
        Err(_) => Config::new(),
    };

    let mut links: Arc<Mutex<HashMap<(net::IpAddr, u16), link::Link>>> =
        Arc::new(Mutex::new(HashMap::new()));

    let args: Vec<String> = std::env::args().collect();

    /*
    if args.len() > 1 {
        let peer = &args[1];

        let mut stream = net::TcpStream::connect(peer).unwrap();

        let noise =
            Noise_XXpsk3_25519_ChaChaPoly_BLAKE2s::new_noise(&config.keypair.private, PSK, true)
                .unwrap();

        let mut pattern = Noise_XXpsk3_25519_ChaChaPoly_BLAKE2s::new(noise).unwrap();

        pattern.initiator(&mut stream).unwrap();

        let noise = pattern.into_inner();

        let mut noise =
            crate::noise::Noise::from(noise.into_transport_mode().unwrap(), &mut stream);
    } */

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
                let links = links.clone();

                thread::spawn(move || {
                    let noise = Noise_XXpsk3_25519_ChaChaPoly_BLAKE2s::new_noise(
                        &config.keypair.private,
                        PSK,
                        false,
                    )
                    .unwrap();

                    let mut pattern = Noise_XXpsk3_25519_ChaChaPoly_BLAKE2s::new(noise).unwrap();

                    pattern.responder(&mut stream).unwrap();

                    let noise = pattern.into_inner();

                    let mut noisy_stream =
                        crate::noise::Noise::from(noise.into_transport_mode().unwrap(), stream);

                    let (send, local_recv) = std::sync::mpsc::channel::<Vec<u8>>();
                    let (local_send, recv) = std::sync::mpsc::channel::<Vec<u8>>();

                    links.lock().unwrap().insert(
                        (address.ip(), address.port()),
                        link::Link {
                            send,
                            recv,
                            tagged_io: HashMap::new(),
                            thread: std::thread::current(),
                        },
                    );

                    loop {
                        match noisy_stream.read_message() {
                            Ok(message) => {
                                let links = links.clone();

                                let s = String::from_utf8(message.clone()).unwrap();
                                let header: UnilinkHeader = json::from_str(&s).unwrap();

                                if header.way == true {
                                    if let Some((sender, receiver)) = links
                                        .lock()
                                        .unwrap()
                                        .get(&(address.ip(), address.port()))
                                        .unwrap()
                                        .tagged_io
                                        .get(&header.tag)
                                    {
                                        sender.send(message).unwrap();
                                    } else {
                                        noisy_stream
                                            .write_message(
                                                json::to_string(&UnilinkHeader {
                                                    tag: header.tag,
                                                    way: !header.way,
                                                    data: String::new(),
                                                    kind: 0,
                                                })
                                                .as_bytes(),
                                            )
                                            .unwrap();
                                    }
                                }
                            }
                            Err(error) => {
                                eprintln!("{:#?}", error);
                                links
                                    .lock()
                                    .unwrap()
                                    .remove(&(address.ip(), address.port()));
                            }
                        }
                    }
                });
            }
            Err(e) => {
                println!("couldn't get client: {:?}", e);
            }
        }
    }
}
