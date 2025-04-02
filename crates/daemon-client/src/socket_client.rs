use std::io::{self, Read, Write};
use std::net::Shutdown;
use std::os::unix::fs::FileTypeExt;
use std::os::unix::net::UnixStream;
use std::str;
use std::{fs, vec};

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

        // TODO: this whole block could be avoided by using 'unix_stream.read_to_string'
        // but for some reason it sometimes stays hanging as if the socket connection
        // would not get closed from the server side, this is solved by checking for 'buffer[n - 1] == 0'
        let mut response = vec![];
        loop {
            let mut buffer = [0; 512];
            let n = unix_stream.read(&mut buffer[..])?;

            response.extend_from_slice(&buffer[..n]);

            if n == 0 || buffer[n - 1] == 0 {
                let _ = unix_stream.shutdown(Shutdown::Read);
                break;
            }
        }

        if !response.is_empty() && response[response.len() - 1] == 0 {
            response.pop();
        }
        let response_string =
            String::from_utf8(response).expect("Expected utf8 string as a response from daemon");
        Ok(response_string)
    }
}
