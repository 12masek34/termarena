use std::net::UdpSocket;

pub fn get_external_id() -> Result<String, reqwest::Error> {
    let ip = reqwest::blocking::get("https://api.ipify.org")?.text()?;
    Ok(ip)
}

pub fn get_local_ip() -> Option<String> {
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    socket.local_addr().ok().map(|addr| addr.ip().to_string())
}
