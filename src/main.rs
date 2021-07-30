mod server;

use std::thread;
use std::fs;
use std::fs::OpenOptions;
use std::io::{self, BufRead, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use structopt::StructOpt;
use uuid::Uuid;

use home::home_dir;


#[derive(Debug, StructOpt)]
struct Cli {
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


fn get_session_id() -> Result<Option<String>, io::Error> {
    let my_uuid = Uuid::new_v4().to_string();
    Ok(Some(my_uuid))
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


fn start_server_if_not_exists(tull_meta_dir: &String, host: String, port: u16) {
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


fn setup_session(data_dir_path: &String, sid: std::option::Option<String>) {
    let session_id = match sid {
        None => get_session_id().unwrap().unwrap(),
        Some(id) => id
    };

    println!("\nAccess the session using this id: {}\n", session_id);

    let session_file_path = format!("{}/{}", data_dir_path, session_id);
    let mut session_file = OpenOptions::new()
        .append(true)
        .create(true)
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


fn setup_and_start_server(
    session_id: Option<String>,
    data_dir_path: &String,
    meta_dir_path: &String,
    host: String,
    port: u16
) {
    if let Err(_e) = setup_data_directories(&data_dir_path, &meta_dir_path) {
        println!("Failed to create the data directories.");
    }

    println!("API: {}", format!("http://{}:{}/tull/api", host, port));
    println!("Web: {}", format!("http://{}:{}/tull/web", host, port));
    println!("Raw: {}", format!("http://{}:{}/tull/raw", host, port));

    start_server_if_not_exists(meta_dir_path, host, port);
    setup_session(&data_dir_path, session_id);
}


fn list_sessions(data_dir_path: &String) {
    let files = fs::read_dir(data_dir_path).unwrap();

    for file in files {
        println!("{}", file.unwrap().file_name().to_str().unwrap());
    }
}


fn check_api_status(host: &String, port: u16) -> bool {
    if let Ok(_res) = reqwest::blocking::get(format!("http://{}:{}/tull/api", host, port)) {
        return true;
    } else {
        return false;
    }
}


fn main() {
    let user_home = home_dir().unwrap().to_owned();
    let user_home_str = user_home.to_str().unwrap();
    let tull_data_dir = format!("{}/{}", user_home_str, ".tull/data");
    let tull_meta_dir = format!("{}/{}", user_home_str, ".tull/meta");

    let args = Cli::from_args();

    if args.status {
        let is_api_alive = check_api_status(&args.host, args.port);
        if is_api_alive {
            println!("API is alive!");
        } else {
            println!("API is down. Use --start option.");
        }
    }

    if args.start {
        setup_and_start_server(None, &tull_data_dir, &tull_meta_dir, args.host.clone(), args.port);
    }

    if args.stop {
        println!("stop server");
    }

    if args.ls {
        list_sessions(&tull_data_dir);
    }

    if args.follow != None {}

    if args.reopen != None {
        setup_and_start_server(args.reopen, &tull_data_dir, &tull_meta_dir, args.host, args.port);
    }
}
