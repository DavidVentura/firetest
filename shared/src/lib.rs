use bincode::{deserialize, serialize};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

#[derive(Serialize, Deserialize, Debug)]
pub enum Pid1Message {
    Booted {
        cmdline: String,
    },
    NetworkUp {
        interface: String,
        ip_address: String,
        netmask: String,
        gateway: String,
    },
    StartingUserProcess {
        process_id: u32,
        command: String,
    },
    UserProcessFinished {
        stdout: Vec<u8>,
        stderr: Vec<u8>,
        exit_code: i32,
    },
    Exiting {
        reason: String,
    },
}
#[derive(Serialize, Deserialize, Debug)]
pub enum HypervisorMessage {
    Exit,
}

pub fn send_message<T, W: Write>(stream: &mut W, message: &T) -> std::io::Result<()>
where
    T: Serialize,
{
    let encoded: Vec<u8> =
        serialize(message).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    let length = encoded.len() as u32;
    stream.write_all(&length.to_be_bytes())?;
    stream.write_all(&encoded)?;
    stream.flush()?;
    Ok(())
}

fn receive_message<T: Read>(stream: &mut T) -> std::io::Result<Option<Vec<u8>>> {
    let mut length_bytes = [0u8; 4];
    if stream.read_exact(&mut length_bytes).is_err() {
        return Ok(None); // Connection closed
    }
    let length = u32::from_be_bytes(length_bytes) as usize;

    let mut buffer = vec![0u8; length];
    stream.read_exact(&mut buffer)?;

    Ok(Some(buffer))
}

// generics are hard OK
pub fn recv_pid1_msg<T: Read>(stream: &mut T) -> std::io::Result<Option<Pid1Message>> {
    let opt = receive_message(stream)?;
    match opt {
        None => Ok(None),
        buf => {
            let msg = deserialize::<Pid1Message>(buf.unwrap().as_slice())
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            Ok(Some(msg))
        }
    }
}

pub fn recv_hypervisor_message<T: Read>(
    stream: &mut T,
) -> std::io::Result<Option<HypervisorMessage>> {
    let opt = receive_message(stream)?;
    match opt {
        None => Ok(None),
        buf => {
            let msg = deserialize::<HypervisorMessage>(buf.unwrap().as_slice())
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            Ok(Some(msg))
        }
    }
}
