use core::time;
use daemonize::{get_err, log, Error, LogInfo, Result};
use libc::{SIGCHLD, SIGKILL};
use signal_hook::consts::{FORBIDDEN, TERM_SIGNALS};
use signal_hook::flag;
use signal_hook::iterator::exfiltrator::WithOrigin;
use signal_hook::iterator::SignalsInfo;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::os::unix::prelude::AsRawFd;
use std::process::Command;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender};
use std::sync::{mpsc, Arc};
use std::{env, thread};

use strum::FromRepr;

const ADDRESS: &str = "127.0.0.1:4242";
const SIZE_BUFF: usize = 400;

// pub struct Client {
//     stream: TcpStream,
//     addr: SocketAddr,
// }

// impl Client {
//     fn new(stream: TcpStream, addr: SocketAddr) -> Self {
//         Self { stream, addr }
//     }

//     fn get_addr(&self) -> String {
//         self.addr.to_string()
//     }
// }

// fn read_from_fd(fd: i32, idx: usize, clients: &mut [Client]) -> Result<bool> {
//     let mut data = [0_u8; SIZE_BUFF];
//     let client = clients.get_mut(idx - 1).ok_or(Error::ClientGetter)?;
//     unsafe {
//         let nb = get_err(
//             libc::read(fd, data.as_mut_ptr() as _, SIZE_BUFF),
//             Error::Read,
//         )?;

//         let msg =
//             String::from_utf8(data[0..(nb as usize)].to_vec()).map_err(|_| Error::ConvertToUTF8)?;
//         let msg = msg.trim().to_string();
//         // TODO
//         // if client.shell_mode > ShellMode::None {
//         //     let res = handle_remote_shell(msg, client, logger);
//         //     match res {
//         //         Err(Error::CommandFailed(_)) => {
//         //             eprintln!("Wrong Command");
//         //             Ok(false)
//         //         }
//         //         Ok(_) => Ok(false),
//         //         Err(e) => Err(e),
//         //     }
//         // } else {
//         //     handle_client(msg, client, logger)
//         // }
//         // If true then exit connection with client
//         Ok(true)
//     }
// }

// fn handle_revent_error(a: i16, addr: String) -> Result<()> {
//     if a & libc::POLLHUP != 0 {
//         log(format!("Hanging up from {addr}\n"), LogInfo::Warn)?;
//     } else if a & libc::POLLERR != 0 {
//         log(format!("Error condition from {addr}\n"), LogInfo::Warn)?;
//     } else if a & libc::POLLNVAL != 0 {
//         log(
//             format!("Invalid request: fd not open from {addr}\n"),
//             LogInfo::Warn,
//         )?;
//     } else {
//         log(
//             format!("Stream socket peer closed connection from {addr}\n"),
//             LogInfo::Warn,
//         )?;
//     }
//     Ok(())
// }

// fn add_client(
//     fds: &mut Connections,
//     listener: &TcpListener,
//     streams: &mut Vec<Client>,
// ) -> Result<()> {
//     let (stream, addr) = listener.accept().map_err(Error::AcceptClient)?;
//     if fds.len() >= 3 {
//         log("Already 2 clients connected\n", LogInfo::Warn)?;
//         return Ok(());
//     }
//     fds.push_from_fd(stream.as_raw_fd());
//     log(
//         format!("Connecting to new address : {addr}\n"),
//         LogInfo::Info,
//     )?;
//     streams.push(Client::new(stream, addr));
//     Ok(())
// }

fn register_signal_hook(sender: Sender<i32>) -> std::io::Result<()> {
    let term_now = Arc::new(AtomicBool::new(false));
    for sig in TERM_SIGNALS {
        flag::register_conditional_shutdown(*sig, 1, Arc::clone(&term_now))?;
        flag::register(*sig, Arc::clone(&term_now))?;
    }
    let mut forbidden = vec![SIGCHLD];
    forbidden.extend(FORBIDDEN);
    let sigs = (1..32).filter(|s| !forbidden.contains(&s));
    let mut signals = SignalsInfo::<WithOrigin>::new(sigs)?;
    let handle = signals.handle();

    for info in &mut signals {
        sender.send(info.signal);
    }

    handle.close();
    Ok(())
}

pub fn server() -> Result<()> {
    let listener = TcpListener::bind(ADDRESS).map_err(Error::ClientErrorBinding)?;
    let (tx, rx): (Sender<i32>, Receiver<i32>) = mpsc::channel();

    let thread = thread::spawn(|| register_signal_hook(tx));
    let mut streams = vec![];

    listener.set_nonblocking(true);

    loop {
        // eprintln!("sleeping");
        //// check_sigup();
        //check_channel_sig();
        //handle_sig(); // -> sigup (reload conf) // -> le reste (kill programm)
        //check_child_status();

        // read signals, from channels, with timeout of 100ms
        // treat all signals at once
        loop {
            let v = rx.recv_timeout(time::Duration::from_millis(100));
            match v {
                Ok(_) => eprintln!("received : {:?}", v), // sigup et down to handle here
                Err(RecvTimeoutError::Timeout) => break,
                Err(e) => {
                    eprintln!("Unknown error : {:?}", e);
                    // quit program with proper error management / clean state
                    break;
                }
            }
        }

        match listener.accept() {
            Ok((stream, addr)) => {
                stream
                    .set_read_timeout(Some(time::Duration::from_millis(2000)))
                    .unwrap();
                streams.push(stream); // add to list, but need the cleanup too
                eprintln!("Connection from {:?}", addr)
            }
            Err(e) => {
                if e.kind() != io::ErrorKind::WouldBlock {
                    eprintln!("Error : {:?}", e)
                }
            }
        }

        for mut stream in &streams {
            stream
                .set_read_timeout(Some(time::Duration::from_millis(100)))
                .unwrap();
            // let mut buf = vec![];
            let mut reader = BufReader::new(stream);
            let mut buf = String::new();
            let _ = match reader.read_line(&mut buf) {
                Err(e) => match e.kind() {
                    io::ErrorKind::WouldBlock => {
                        // if timeout
                        println!("would have blocked");
                        continue;
                    }
                    _ => panic!("Got an error: {}", e),
                },
                Ok(m) => {
                    // read a line
                    println!("Received {:?}, {:?}", m, buf);
                    if m == 0 {
                        // doesn't reach here.
                        continue;
                    }
                    m
                }
            };
        }

        // check status of children
        // check_child_status

        thread::sleep(time::Duration::from_millis(300));
    }
    thread.join().expect("joining thread");
}
