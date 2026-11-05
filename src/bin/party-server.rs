use mgame::{
    level::Level,
    party::{Room, DEFAULT_PORT},
    party_server::PartyServer,
};
use std::{net::SocketAddr, time::Duration};

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    if args.iter().any(|a| a == "--help") {
        println!("party-server [--bind IP:PORT] [--arena 0..17 | --map PATH] [--endless]\nDefaults: 0.0.0.0:7777, arena 0, 20-wave campaign. One room, up to 8 players.");
        return Ok(());
    }
    let mut bind: SocketAddr = format!("0.0.0.0:{DEFAULT_PORT}").parse()?;
    let mut level = Level::builtin(0);
    let mut endless = false;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--bind" | "--arena" | "--map" => {
                let value = args.get(i + 1).ok_or("Missing option value")?;
                match args[i].as_str() {
                    "--bind" => bind = value.parse()?,
                    "--arena" => {
                        let index: usize = value.parse()?;
                        if index >= mgame::arenas::NAMES.len() {
                            return Err("Arena index is out of range".into());
                        }
                        level = Level::builtin(index);
                    }
                    _ => level = Level::load(std::path::Path::new(value))?,
                }
                i += 2;
            }
            "--endless" => {
                endless = true;
                i += 1;
            }
            other => return Err(format!("Unknown option: {other}").into()),
        }
    }
    let room = Room::new(level, endless)?;
    let mut server = PartyServer::bind(bind, room)?;
    println!(
        "Online Party listening on {} / {} / maximum 8 players",
        server.local_addr()?,
        server.room.level.name
    );
    loop {
        server.poll()?;
        std::thread::sleep(Duration::from_millis(1));
    }
}

fn main() {
    if let Err(error) = run() {
        eprintln!("Party server: {error}");
        std::process::exit(1);
    }
}
