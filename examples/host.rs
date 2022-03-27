//! Simple host-side protocol example
#![deny(warnings, clippy::all)]

use byteorder::ReadBytesExt;
use electricui_embedded::prelude::*;
use err_derive::Error;
use serial::prelude::*;
use std::io::{self, Write};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    mpsc::{self, channel},
    Arc, Mutex,
};
use std::time::Duration;
use std::{process, str, thread};
use structopt::StructOpt;

#[derive(Debug, Error)]
enum Error {
    #[error(display = "Control-C error")]
    Ctrlc(#[error(source)] ctrlc::Error),

    #[error(display = "Serial error")]
    Serial(#[error(source)] serial::Error),

    #[error(display = "EUI packet error")]
    Packet(#[source] electricui_embedded::wire::packet::Error),

    #[error(display = "EUI framing error")]
    Framing(#[source] electricui_embedded::wire::framing::Error),

    #[error(display = "EUI decoder error")]
    Decoder(#[source] electricui_embedded::decoder::Error),

    #[error(display = "IO error")]
    Io(#[source] io::Error),

    #[error(display = "Recv timeout error")]
    RecvTimeout(#[source] mpsc::RecvTimeoutError),

    #[error(display = "Thread join error")]
    Join,
}

#[derive(Debug, StructOpt)]
#[structopt(about = "ElectricUI host example.")]
struct Opts {
    /// Serial device path
    #[structopt(name = "device")]
    device: String,
}

const BUFFER_SIZE: usize = Framing::max_encoded_len(Packet::<&[u8]>::MAX_PACKET_SIZE);
const RX_TIMEOUT: Duration = Duration::from_millis(500);

fn main() -> Result<(), Error> {
    let opts = Opts::from_args();

    let running = Arc::new(AtomicUsize::new(0));
    let r = running.clone();
    ctrlc::set_handler(move || {
        let prev = r.fetch_add(1, Ordering::SeqCst);
        if prev == 0 {
            println!("Shutting down");
        } else {
            println!("Force exit");
            process::exit(1);
        }
    })?;

    let mut port = serial::open(&opts.device)?;
    port.reconfigure(&|settings| {
        settings.set_baud_rate(serial::Baud115200)?;
        settings.set_char_size(serial::Bits8);
        settings.set_parity(serial::ParityNone);
        settings.set_stop_bits(serial::Stop1);
        settings.set_flow_control(serial::FlowNone);
        Ok(())
    })?;
    port.set_timeout(Duration::from_millis(1))?;
    let port = Arc::new(Mutex::new(port));

    let reader = port.clone();

    let (tx, rx) = channel();

    let r = running.clone();
    let sender = thread::spawn(move || {
        let mut dec_buf = [0_u8; BUFFER_SIZE];
        let mut dec = Decoder::new(&mut dec_buf);

        while r.load(Ordering::SeqCst) == 0 {
            let mut lock = reader.try_lock();
            let rd = if let Ok(ref mut rd) = lock {
                rd
            } else {
                continue;
            };

            match rd.read_u8() {
                Ok(b) => match dec.decode(b) {
                    Ok(Some(pkt)) => {
                        println!("<< {}", pkt);
                        tx.send(pkt.as_ref().to_vec()).unwrap();
                    }
                    Err(e) => eprint!("{}", e),
                    _ => (),
                },
                Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                Err(e) => eprintln!("{:?}", e),
            }
        }
    });

    let mut buf = vec![0_u8; BUFFER_SIZE];

    let mut state = State::BoardId;
    while running.load(Ordering::SeqCst) == 0 {
        match state {
            State::BoardId => {
                let size = board_id_req(&mut buf)?;
                port.lock().unwrap().write_all(&buf[..size])?;
                board_id_resp(&rx.recv_timeout(RX_TIMEOUT)?)?;
                state = State::Name;
            }
            State::Name => {
                let size = name_req(&mut buf)?;
                port.lock().unwrap().write_all(&buf[..size])?;
                name_resp(&rx.recv_timeout(RX_TIMEOUT)?)?;
                state = State::AnnounceIds;
            }
            State::AnnounceIds => {
                let size = am_req(&mut buf)?;
                port.lock().unwrap().write_all(&buf[..size])?;
                am_list_resp(&rx.recv_timeout(RX_TIMEOUT)?)?;
                let num_ids = am_end_resp(&rx.recv_timeout(RX_TIMEOUT)?)?;
                state = State::TrackedVars(num_ids);
            }
            State::TrackedVars(num_ids) => {
                let size = tracked_vars_req(&mut buf)?;
                port.lock().unwrap().write_all(&buf[..size])?;
                for _ in 0..num_ids {
                    tracked_vars_resp(&rx.recv_timeout(RX_TIMEOUT)?)?;
                }
                state = State::Heartbeat;
            }
            State::Heartbeat => {
                let val = 3;
                let size = heartbeat_req(val, &mut buf)?;
                port.lock().unwrap().write_all(&buf[..size])?;
                let resp_val = heartbeat_resp(&rx.recv_timeout(RX_TIMEOUT)?)?;
                assert_eq!(val, resp_val);
                state = State::Done
            }
            State::Done => {
                let _ = running.fetch_add(1, Ordering::SeqCst);
            }
        }
    }

    sender.join().map_err(|_| Error::Join)?;

    Ok(())
}

enum State {
    BoardId,
    Name,
    AnnounceIds,
    TrackedVars(usize),
    Heartbeat,
    // TODO - add query and action
    Done,
}

fn board_id_req(buf: &mut [u8]) -> Result<usize, Error> {
    let mut pkt = [0_u8; 6];
    let mut p = Packet::new_unchecked(&mut pkt[..]);
    p.set_data_length(0)?;
    p.set_typ(MessageType::U16);
    p.set_internal(true);
    p.set_offset(false);
    p.set_id_length(1)?;
    p.set_response(true);
    p.set_acknum(0);
    p.msg_id_mut()?
        .copy_from_slice(MessageId::INTERNAL_BOARD_ID.as_bytes());
    p.set_checksum(p.compute_checksum()?)?;
    println!("Requesting board ID");
    println!(">> {p}");
    Ok(Framing::encode_buf(p.as_ref(), buf))
}

fn board_id_resp(buf: &[u8]) -> Result<(), Error> {
    let p = Packet::new(buf)?;
    let id = p.payload()?;
    println!("Board ID: {:02X?}", id);
    Ok(())
}

fn name_req(buf: &mut [u8]) -> Result<usize, Error> {
    let mut pkt = [0_u8; 9];
    let mut p = Packet::new_unchecked(&mut pkt[..]);
    p.set_data_length(0)?;
    p.set_typ(MessageType::Callback);
    p.set_internal(false);
    p.set_offset(false);
    p.set_id_length(4)?;
    p.set_response(true);
    p.set_acknum(0);
    p.msg_id_mut()?
        .copy_from_slice(MessageId::BOARD_NAME.as_bytes());
    p.set_checksum(p.compute_checksum()?)?;
    println!("Requesting name");
    println!(">> {p}");
    Ok(Framing::encode_buf(p.as_ref(), buf))
}

fn name_resp(buf: &[u8]) -> Result<(), Error> {
    let p = Packet::new(buf)?;
    let n = p.payload()?;
    if let Ok(s) = str::from_utf8(n) {
        println!("Name: '{}'", s);
    } else {
        println!("Name: {:02X?}", n);
    }
    Ok(())
}

fn am_req(buf: &mut [u8]) -> Result<usize, Error> {
    let mut pkt = [0_u8; 6];
    let mut p = Packet::new_unchecked(&mut pkt[..]);
    p.set_data_length(0)?;
    p.set_typ(MessageType::Callback);
    p.set_internal(true);
    p.set_offset(false);
    p.set_id_length(1)?;
    p.set_response(true);
    p.set_acknum(0);
    p.msg_id_mut()?
        .copy_from_slice(MessageId::INTERNAL_AM.as_bytes());
    p.set_checksum(p.compute_checksum()?)?;
    println!("Requesting writable IDs announcement");
    println!(">> {p}");
    Ok(Framing::encode_buf(p.as_ref(), buf))
}

fn am_list_resp(buf: &[u8]) -> Result<(), Error> {
    let p = Packet::new(buf)?;
    let ids: Vec<&[u8]> = p
        .payload()?
        .split(|b| *b == b'\0')
        .filter(|&id| !id.is_empty())
        .collect();
    println!("Message IDs ({}):", ids.len());
    for id in ids.into_iter() {
        let mid =
            MessageId::new(id).ok_or(electricui_embedded::wire::packet::Error::InvalidMessageId)?;
        println!("  {}", mid);
    }
    Ok(())
}

fn am_end_resp(buf: &[u8]) -> Result<usize, Error> {
    let p = Packet::new(buf)?;
    assert_eq!(p.typ()?, MessageType::U8); // TODO - protocol allows for u16 too
    let num_ids = p.payload()?[0];
    println!("Got AM_END, count = {num_ids}");
    Ok(num_ids as _)
}

fn tracked_vars_req(buf: &mut [u8]) -> Result<usize, Error> {
    let mut pkt = [0_u8; 6];
    let mut p = Packet::new_unchecked(&mut pkt[..]);
    p.set_data_length(0)?;
    p.set_typ(MessageType::Callback);
    p.set_internal(true);
    p.set_offset(false);
    p.set_id_length(1)?;
    p.set_response(true);
    p.set_acknum(0);
    p.msg_id_mut()?
        .copy_from_slice(MessageId::INTERNAL_AV.as_bytes());
    p.set_checksum(p.compute_checksum()?)?;
    println!("Requesting tracked variables");
    println!(">> {p}");
    Ok(Framing::encode_buf(p.as_ref(), buf))
}

fn tracked_vars_resp(buf: &[u8]) -> Result<(), Error> {
    let p = Packet::new(buf)?;
    let id = p.msg_id()?;
    let typ = p.typ()?;
    let data = p.payload()?;
    println!("Got tracked var Id({id}), Type({typ:?}), Data({data:02X?})");
    Ok(())
}

fn heartbeat_req(val: u8, buf: &mut [u8]) -> Result<usize, Error> {
    let mut pkt = [0_u8; 7];
    let mut p = Packet::new_unchecked(&mut pkt[..]);
    p.set_data_length(1)?;
    p.set_typ(MessageType::U8);
    p.set_internal(true);
    p.set_offset(false);
    p.set_id_length(1)?;
    p.set_response(true);
    p.set_acknum(0);
    p.msg_id_mut()?
        .copy_from_slice(MessageId::INTERNAL_HEARTBEAT.as_bytes());
    p.payload_mut()?[0] = val;
    p.set_checksum(p.compute_checksum()?)?;
    println!("Requesting heartbeat val={val}");
    println!(">> {p}");
    Ok(Framing::encode_buf(p.as_ref(), buf))
}

fn heartbeat_resp(buf: &[u8]) -> Result<u8, Error> {
    let p = Packet::new(buf)?;
    assert_eq!(p.typ()?, MessageType::U8);
    let val = p.payload()?[0];
    println!("Got heartbeat val={val}");
    Ok(val)
}
