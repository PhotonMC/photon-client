//! `cargo xtask rcon`: commands to the test server over RCON.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

use super::layout::{RCON_PASSWORD, RCON_PORT};

const LOGIN: i32 = 3;
const COMMAND: i32 = 2;

fn packet(
    connection: &mut TcpStream,
    id: i32,
    kind: i32,
    body: &str,
) -> Result<(i32, String), String> {
    let mut data = Vec::new();
    data.extend(id.to_le_bytes());
    data.extend(kind.to_le_bytes());
    data.extend(body.as_bytes());
    data.extend([0, 0]);
    let length = i32::try_from(data.len()).map_err(|_| "RCON command too long".to_owned())?;
    let io_error = |error: std::io::Error| format!("RCON: {error}");
    let framed = [length.to_le_bytes().as_slice(), &data].concat();
    connection.write_all(&framed).map_err(io_error)?;
    let mut size = [0u8; 4];
    connection.read_exact(&mut size).map_err(io_error)?;
    let mut payload = vec![
        0u8;
        usize::try_from(i32::from_le_bytes(size))
            .unwrap_or(0)
            .max(10)
    ];
    connection.read_exact(&mut payload).map_err(io_error)?;
    let answered = i32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]);
    let text = String::from_utf8_lossy(&payload[8..payload.len() - 2]).into_owned();
    Ok((answered, text))
}

/// Sends `commands` to the test server and returns the replies.
pub fn send(commands: &[String]) -> Result<Vec<String>, String> {
    let address = ("127.0.0.1", RCON_PORT);
    let mut connection = TcpStream::connect(address).map_err(|error| {
        format!("no RCON on 127.0.0.1:{RCON_PORT} ({error}); is the server running?")
    })?;
    let timeout = Some(Duration::from_secs(10));
    connection
        .set_read_timeout(timeout)
        .map_err(|error| error.to_string())?;
    connection
        .set_write_timeout(timeout)
        .map_err(|error| error.to_string())?;
    if packet(&mut connection, 1, LOGIN, RCON_PASSWORD)?.0 == -1 {
        return Err("RCON refused the password".to_owned());
    }
    commands
        .iter()
        .map(|command| Ok(packet(&mut connection, 2, COMMAND, command)?.1))
        .collect()
}

pub fn rcon(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        return Err("rcon needs at least one command".to_owned());
    }
    for (command, reply) in args.iter().zip(send(args)?) {
        println!("{command} -> {reply}");
    }
    Ok(())
}
