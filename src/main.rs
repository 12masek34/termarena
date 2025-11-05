use std::env;

mod game;
mod network;
mod server;
mod client;
mod utils;
mod map;


#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Использование:");
        eprintln!("{} create [port]", args[0]);
        eprintln!("{} join <ip:port>", args[0]);
        return;
    }

    match args[1].as_str() {
        "create" => {
            let port = args.get(2).cloned().unwrap_or_else(|| String::from("4000"));
            match server::run_server(&port).await {
                Ok(external_ip) => {
                    println!("Игра создана! Ваш внешний адрес: {}:{}", external_ip, port);
                    let _ = client::run_client(&format!("127.0.0.1:{}", port)).await;
                },
                Err(e) => eprintln!("Ошибка запуска сервера: {}", e),
            }
        },
        "join" => {
            if args.len() < 3 {
                eprintln!("Необходимо указать адрес сервера: join <ip:port>");
            }
            let addr = &args[2];
            let _ = client::run_client(addr).await;
        },
        _ => eprintln!("Неизвестный режим: {}", args[1]),
    }
}
