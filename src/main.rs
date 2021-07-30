mod server;

use std::thread;
use std::fs;
use std::fs::OpenOptions;
use std::io::{self, BufRead, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use structopt::StructOpt;
use uuid::Uuid;

use home::home_dir;


#[derive(StructOpt)]
struct Cli {
    #[structopt(short, long)]
    web: bool,

    #[structopt(long)]
    start: bool,

    #[structopt(long)]
    stop: bool,

    #[structopt(short, long)]
    status: bool,

    #[structopt(short, long)]
    ls: bool,

    #[structopt(short, long)]
    follow: Option<String>,

    #[structopt(short, long)]
    reopen: Option<String>,

    #[structopt(short, long, default_value = "127.0.0.1")]
    host: String,

    #[structopt(short, long, default_value = "17171")]
    port: u16,
}


fn setup_data_directories(data_dir_path: &String, meta_dir_path: &String) -> Result<(), io::Error> {
    fs::create_dir_all(data_dir_path)?;
    fs::create_dir_all(meta_dir_path)?;

    Ok(())
}


fn get_session_id() -> Result<String, io::Error> {
    let my_uuid = Uuid::new_v4().to_string();
    Ok(my_uuid)
}


fn start_server(host: String, port: u16) {
    let host_parts : Vec<u8> = host.split(".").map(|octet| octet.parse().unwrap()).collect();
    let socket = SocketAddr::new(
        IpAddr::V4(
            Ipv4Addr::new(
                host_parts[0], host_parts[1], host_parts[2], host_parts[3]
            )
        ),
        port
    );
    server::run_server(socket);
}


fn start_server_if_not_exists(tull_meta_dir: String, host: String, port: u16) {
    if let Err(_result) = reqwest::blocking::get(format!("http://{}:{}/tull/api", host, port)) {
        let _log_file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(format!("{}/server.log", tull_meta_dir))
            .unwrap();

        thread::spawn(move || {
            start_server(host, port);
        });
    }
}


fn setup_session(data_dir_path: &String) {
    let session_id = get_session_id().unwrap();

    println!("\nAccess the session using this id: {}\n", session_id);

    let session_file_path = format!("{}/{}", data_dir_path, session_id);
    let mut session_file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(session_file_path.clone())
        .unwrap();

    let stdin = io::stdin();
    let mut line_count = 0;

    for line in stdin.lock().lines() {
        let value = line.as_ref().unwrap().as_str();
        if let Err(_e) = session_file.write_all(format!("{}\n", value).as_bytes()) {
            println!("Couldn't write to session file.")
        }
        println!("{}", line.unwrap());

        line_count += 1;
    }

    if line_count == 0 {
        if let Err(_e) = fs::remove_file(session_file_path) {
            println!("Couldn't remove the session file.");
        }
    }
}


fn main() {
    let args = Cli::from_args();

    if args.web {
        println!("TULL_API_URL: {}", format!("http://{}:{}/tull/api", args.host, args.port));
        println!("TULL_WEB_URL: {}", format!("http://{}:{}/tull/web", args.host, args.port));
        println!("TULL_RAW_URL: {}", format!("http://{}:{}/tull/raw", args.host, args.port));
    }

    if args.status {
        println!("check API status");
    }

    if args.start {
        if let Err(_e) = setup_data_directories(&tull_data_dir, &tull_meta_dir) {
            println!("Failed to create the data directories.");
        }

        println!("API: {}", format!("http://{}:{}/tull/api", args.host, args.port));
        println!("Web: {}", format!("http://{}:{}/tull/web", args.host, args.port));
        println!("Raw: {}", format!("http://{}:{}/tull/raw", args.host, args.port));

        start_server_if_not_exists(tull_meta_dir, args.host, args.port);
        setup_session(&tull_data_dir);
    }

    if args.stop {
        println!("stop server");
    }

    if args.ls {
        println!("list session ids");
    }

    let user_home = home_dir().unwrap().to_owned();
    let user_home_str = user_home.to_str().unwrap();
    let tull_data_dir = format!("{}/{}", user_home_str, ".tull/data");
    let tull_meta_dir = format!("{}/{}", user_home_str, ".tull/meta");

    if let Err(_e) = setup_data_directories(&tull_data_dir, &tull_meta_dir) {
        println!("Failed to create the data directories.");
    }

    setup_session(tull_data_dir, tull_meta_dir, args.host, args.port);
}
