use std::env;
use std::io::{self, Read, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;

const UNLOCK_OPERATION: u32 = 1;
const RESPONSE_LENGTH: u32 = 8;

fn put_u32(buf: &mut Vec<u8>, n: u32) {
    buf.extend_from_slice(&n.to_be_bytes());
}

fn build_unlock_packet(password: &[u8]) -> Result<Vec<u8>, &'static str> {
    let packet_len = 12usize
        .checked_add(password.len())
        .ok_or("packet length overflow")?;

    let packet_len_u32 =
        u32::try_from(packet_len).map_err(|_| "packet length exceeds protocol limit")?;

    let password_len_u32 =
        u32::try_from(password.len()).map_err(|_| "password length exceeds protocol limit")?;

    let mut packet = Vec::with_capacity(packet_len);

    put_u32(&mut packet, packet_len_u32);
    put_u32(&mut packet, UNLOCK_OPERATION);
    put_u32(&mut packet, password_len_u32);
    packet.extend_from_slice(password);

    Ok(packet)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut password = String::new();
    io::stdin().read_to_string(&mut password)?;

    while password.ends_with('\n') || password.ends_with('\r') {
        password.pop();
    }

    if password.is_empty() {
        return Err("empty password".into());
    }

    let control_dir = match env::var_os("GNOME_KEYRING_CONTROL") {
        Some(path) => PathBuf::from(path),
        None => {
            let runtime = env::var_os("XDG_RUNTIME_DIR").ok_or("XDG_RUNTIME_DIR is not set")?;
            PathBuf::from(runtime).join("keyring")
        }
    };

    let socket_path = control_dir.join("control");
    let mut stream = UnixStream::connect(&socket_path)?;

    /*
     * GNOME Keyring control protocol.
     *
     * First byte lets the daemon obtain our Unix credentials.
     */
    stream.write_all(&[0])?;

    /*
     * Unlock request:
     *
     * uint32 total packet length
     * uint32 operation (1 = UNLOCK)
     * uint32 password length
     * password bytes (no terminating NUL)
     */
    let mut packet = build_unlock_packet(password.as_bytes())?;

    stream.write_all(&packet)?;

    /*
     * Response:
     *
     * uint32 packet length -- must be 8
     * uint32 result -- 0 = OK
     */
    let mut response = [0u8; 8];
    stream.read_exact(&mut response)?;

    let response_len = u32::from_be_bytes(response[0..4].try_into()?);
    let result = u32::from_be_bytes(response[4..8].try_into()?);

    packet.fill(0);
    password.replace_range(.., "");

    if response_len != RESPONSE_LENGTH {
        return Err(format!("invalid GNOME Keyring response length: {response_len}").into());
    }

    match result {
        0 => Ok(()),
        1 => Err("GNOME Keyring rejected the password".into()),
        2 => Err("GNOME Keyring operation failed".into()),
        3 => Err("GNOME Keyring daemon is unavailable".into()),
        other => Err(format!("unknown GNOME Keyring result: {other}").into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read_u32(bytes: &[u8]) -> u32 {
        u32::from_be_bytes(bytes.try_into().unwrap())
    }

    #[test]
    fn unlock_packet_has_correct_header() {
        let packet = build_unlock_packet(b"test").unwrap();

        assert_eq!(read_u32(&packet[0..4]), 16);
        assert_eq!(read_u32(&packet[4..8]), UNLOCK_OPERATION);
        assert_eq!(read_u32(&packet[8..12]), 4);
        assert_eq!(&packet[12..], b"test");
    }

    #[test]
    fn unlock_packet_contains_no_terminating_nul() {
        let packet = build_unlock_packet(b"abc").unwrap();

        assert_eq!(packet.len(), 15);
        assert_eq!(&packet[12..], b"abc");
    }

    #[test]
    fn unlock_packet_handles_empty_payload() {
        let packet = build_unlock_packet(b"").unwrap();

        assert_eq!(read_u32(&packet[0..4]), 12);
        assert_eq!(read_u32(&packet[4..8]), UNLOCK_OPERATION);
        assert_eq!(read_u32(&packet[8..12]), 0);
        assert_eq!(packet.len(), 12);
    }

    #[test]
    fn unlock_packet_uses_utf8_byte_length() {
        let password = "päss".as_bytes();
        let packet = build_unlock_packet(password).unwrap();

        assert_eq!(read_u32(&packet[0..4]) as usize, 12 + password.len());
        assert_eq!(read_u32(&packet[8..12]) as usize, password.len());
        assert_eq!(&packet[12..], password);
    }
}
