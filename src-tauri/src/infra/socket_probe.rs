use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

pub fn probe_socket(host: &str, port: u16, timeout: Duration) -> Result<(), String> {
    let addresses = (host, port)
        .to_socket_addrs()
        .map_err(|error| format!("无法解析主机: {error}"))?
        .collect::<Vec<_>>();

    if addresses.is_empty() {
        return Err("未解析到可用地址".to_string());
    }

    let mut last_error = None;

    for address in addresses {
        match TcpStream::connect_timeout(&address, timeout) {
            Ok(stream) => {
                let _ = stream.set_read_timeout(Some(timeout));
                let _ = stream.set_write_timeout(Some(timeout));
                return Ok(());
            }
            Err(error) => {
                last_error = Some(error.to_string());
            }
        }
    }

    Err(last_error.unwrap_or_else(|| "未知连接错误".to_string()))
}
