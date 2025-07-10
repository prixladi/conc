use std::fs;
use std::io::{self, Read, Write};
use std::net::Shutdown;
use std::os::unix::fs::FileTypeExt;
use std::os::unix::net::UnixStream;
use std::str;

#[derive(Debug, Clone)]
pub struct SocketClient {
    pub socket_path: String,
}

impl SocketClient {
    pub fn new(socket_path: &str) -> Self {
        Self {
            socket_path: socket_path.to_string(),
        }
    }

    pub fn is_alive(&self) -> bool {
        let socket_exists = fs::metadata(&self.socket_path)
            .map(|stat| stat.file_type().is_socket())
            .unwrap_or(false);

        if !socket_exists {
            return false;
        }

        let health_check = self.send(b"\0");
        match health_check {
            Ok(res) => res.is_empty(),
            Err(_) => false,
        }
    }

    pub fn send(&self, message: &[u8]) -> io::Result<String> {
        let mut unix_stream = UnixStream::connect(&self.socket_path)?;

        unix_stream.write_all(message)?;
        unix_stream.write_all(b"\0")?;
        unix_stream.shutdown(Shutdown::Write)?;

        let mut response = String::new();

        unix_stream.read_to_string(&mut response)?;

        let last_char = response.chars().last();

        if let Some(lc) = last_char {
            if lc == '\0' {
                response.truncate(response.len() - 1);
            }
        }

        Ok(response)
    }
}
