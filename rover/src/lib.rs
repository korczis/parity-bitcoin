extern crate chain;
extern crate db;
extern crate logs;
extern crate network;
extern crate p2p;
extern crate sync;
extern crate verification;

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use db::BlockProvider;
use sync::{create_local_sync_node, create_sync_connection_factory, create_sync_peers};

pub const THREADS: usize = 2;
pub const PROTOCOL_VERSION: u32 = 70_014;
pub const PROTOCOL_MINIMUM: u32 = 70_001;
pub const INBOUND_CONNECTIONS: u32 = 4;
pub const OUTBOUND_CONNECTIONS: u32 = 16;
pub const USER_AGENT: &str = "/Satoshi:0.12.1/";
pub const PORT: u16 = 8113;

pub const LOG_INFO: &'static str = "sync=info";

// pub const NETWORK
pub fn rove() {
    let mut el = p2p::event_loop();

    // logs::init(LOG_INFO, logs::DateAndColorLogFormatter);

    let network = network::Network::Mainnet;
    let consensus_fork = network::ConsensusFork::NoFork;
    let consensus = network::ConsensusParams::new(network, consensus_fork);

    // let internet_protocol = p2p::InternetProtocol::IpV4;
    let internet_protocol = p2p::InternetProtocol::Any;

    let seeds = vec![
        // Pieter Wuille
        String::from("seed.bitcoin.sipa.be:8333"),
        // Matt Corallo
        String::from("dnsseed.bluematt.me:8333"),
        // Luke Dashjr
        String::from("dnsseed.bitcoin.dashjr.org:8333"),
        // Christian Decker
        String::from("seed.bitcoinstats.com:8333"),
        // Jonas Schnelli
        String::from("seed.bitcoin.jonasschnelli.ch:8333"),
        // Peter Todd
        String::from("seed.btc.petertodd.org:8333"),
        //
        String::from("seed.voskuil.org:8333"),
    ];

//    let seeds = vec![
//        String::from( "testnet-seed.bitcoin.jonasschnelli.ch:18333"),
//        String::from("seed.tbtc.petertodd.org:18333"),
//        String::from("testnet-seed.bluematt.me:18333"),
//        String::from("testnet-seed.bitcoin.schildbach.de:18333"),
//        String::from("testnet-seed.voskuil.org:18333"),
//    ];

    let node_table_path = PathBuf::from(r"nodes.csv");

    let services = 0u64.into(); // 1u64.into();

    let connect = None;

    let p2p_cfg = p2p::Config {
        threads: THREADS,
        inbound_connections: INBOUND_CONNECTIONS,
        outbound_connections: OUTBOUND_CONNECTIONS,

        connection: p2p::NetConfig {
            protocol_version: PROTOCOL_VERSION,
            protocol_minimum: PROTOCOL_MINIMUM,
            magic: consensus.magic(),
            local_address: SocketAddr::new("127.0.0.1".parse().unwrap(), PORT),
            services,
            user_agent: USER_AGENT.into(),
            start_height: 0,
            relay: true,
        },

        peers: connect.map_or_else(|| vec![], |x| vec![x]),
        seeds,
        node_table_path,
        preferable_services: services,
        internet_protocol,
    };

    let sync_peers = create_sync_peers();
    println!("{:?}", &sync_peers);

    let verification_params = sync::VerificationParameters {
        verification_level: verification::VerificationLevel::NoVerification,
        verification_edge: 0u8.into(),
    };

    let cache_size = 16_108_864;

    let db_path = "_data";
    let db = Arc::new(db::BlockChainDatabase::open_at_path(db_path, cache_size).expect("Failed to open database"));

    let genesis_block: chain::IndexedBlock = network.genesis_block().into();
    let _: Result<(), String> = match db.block_hash(0) {
        Some(ref db_genesis_block_hash) if db_genesis_block_hash != genesis_block.hash() => Err("Trying to open database with incompatible genesis block".into()),
        Some(_) => Ok(()),
        None => {
            let hash = genesis_block.hash().clone();
            db.insert(genesis_block).expect("Failed to insert genesis block to the database");
            db.canonize(&hash).expect("Failed to canonize genesis block");
            Ok(())
        }
    };

    let local_sync_node = create_local_sync_node(consensus, db.clone(), sync_peers.clone(), verification_params);
    let sync_connection_factory = create_sync_connection_factory(sync_peers.clone(), local_sync_node.clone());

    let p2p = p2p::P2P::new(p2p_cfg, sync_connection_factory, el.handle()).unwrap();
    let _ = p2p.run();

    el.run(p2p::forever()).unwrap();
}
