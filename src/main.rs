use std::env;

mod client;
mod game;
mod map;
mod network;
mod server;
mod utils;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 1 {
        eprintln!("Использование:");
        eprintln!("{} create [port]", args[0]);
        eprintln!("{} join [ip:port]", args[0]);
        return;
    }

    match args[1].as_str() {
        "create" => {
            let port = args.get(2).cloned().unwrap_or_else(|| String::from("8888"));
            match server::run_server(&port).await {
                Err(e) => eprintln!("Ошибка запуска сервера: {}", e),
                _ => (),
            }
        }
        "join" => {
            if args.len() < 3 {
                eprintln!("Необходимо указать адрес сервера: join [ip:port]");
                return;
            }
            let _ = client::run_client(&args[2]).await;
        }
        _ => eprintln!("Неизвестный режим: {}", args[1]),
    }
}
