use std::io::{self, BufRead};
use std::net::TcpStream;

pub fn capture_adsb_data() -> Option<String> {
    let adsb_server = std::env::var("ADSB_SERVER").unwrap_or_else(|_| "127.0.0.1:30003".to_string());

    // Connect to ADS-B source
    match TcpStream::connect(&adsb_server) {
        Ok(mut stream) => {
            let mut buffer = String::new();
            let mut reader = io::BufReader::new(&mut stream);

            // Read a single line of ADS-B data
            match reader.read_line(&mut buffer) {
                Ok(_) => {
                    if buffer.trim().is_empty() {
                        None
                    } else {
                        Some(buffer.trim().to_string())
                    }
                }
                Err(err) => {
                    eprintln!("Error reading ADS-B data: {}", err);
                    None
                }
            }
        }
        Err(err) => {
            eprintln!("Failed to connect to ADS-B server {}: {}", adsb_server, err);
            None
        }
    }
}
