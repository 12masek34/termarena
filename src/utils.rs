use std::net::UdpSocket;


pub async fn get_extermal_id() -> Result<String, reqwest::Error> {
    let ip: String = reqwest::get("https://api.ipify.org")
        .await?
        .text()
        .await?;

    Ok(ip)
}


pub async fn get_local_ip() -> Option<String> {
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    socket.local_addr().ok().map(|addr| addr.ip().to_string())
}
