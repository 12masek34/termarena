use std::env;
use termarena::server;
use termarena::utils;

fn main() {
    let args: Vec<String> = env::args().collect();
    let port = args.get(1).cloned().unwrap_or_else(|| String::from("8888"));

    let external_ip = utils::get_external_id().unwrap_or_else(|e| {
        eprintln!("Failed to get external IP: {}", e);
        "unknown".to_string()
    });

    let local_ip = utils::get_local_ip().unwrap_or("unknown".to_string());

    let external_with_port = format!("{}:{}", external_ip, port);
    let local_with_port = format!("{}:{}", local_ip, port);

    println!(
        "External IP: {}, Local IP: {}",
        external_with_port, local_with_port
    );

    server::run_server(port);
}
