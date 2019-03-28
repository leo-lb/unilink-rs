use serde_derive::{Deserialize, Serialize};
use std::fs::File;
use std::net;
use std::thread;

mod noise;
mod noise_pattern;
mod messaging;

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
    //ed25519_public: PublicKey,
}

#[derive(Serialize, Deserialize)]
struct Config {
    role: Role,
    port: u16,
    //ed25519_public: PublicKey,
    //ed25519_secret: SecretKey,
}

impl Config {
    fn new() -> Self {
        //let (ed25519_public, ed25519_secret) = gen_keypair();

        Config {
            role: Role::Client,
            port: 0,
            //ed25519_public,
            //ed25519_secret,
        }
    }
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
