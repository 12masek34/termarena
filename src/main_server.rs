use std::env;
use termarena::config;
use termarena::server;
use termarena::utils;

fn main() {
    let args: Vec<String> = env::args().collect();
    let port = args
        .get(1)
        .cloned()
        .unwrap_or_else(|| config::UDP_PORT.to_string());

    let local_ip = utils::get_local_ip().unwrap_or("unknown".to_string());

    let local_with_port = format!("{}:{}", local_ip, port);

    println!("Local IP: {}", local_with_port);

    server::run_server(port);
}
