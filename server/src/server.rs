use crate::connections::Connections;
use daemonize::{get_err, Error, LogInfo, Result, TintinReporter};
use std::collections::HashMap;
use std::env;
use std::io::Write;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::os::unix::prelude::AsRawFd;
use std::process::Command;

use strum::FromRepr;

const ADDRESS: &str = "127.0.0.1:4242";
const SIZE_BUFF: usize = 400;

const EXIT: &str = "exit";
const QUIT: &str = "quit";
const SHELL: &str = "shell";

#[derive(FromRepr, PartialEq, PartialOrd, Clone)]
enum ShellMode {
    None = 1,
    Shell = 2,
    Bash = 3,
}

pub struct Client {
    stream: TcpStream,
    addr: SocketAddr,
    shell_mode: ShellMode,
}

impl Client {
    fn new(stream: TcpStream, addr: SocketAddr) -> Self {
        Self {
            stream,
            addr,
            shell_mode: ShellMode::None,
        }
    }
    fn change_shell_mode(&mut self, nb: isize) -> Result<()> {
        let tmp = ShellMode::from_repr((self.shell_mode.clone() as isize + nb) as usize)
            .ok_or(Error::ShellModeOverflow)?;
        self.shell_mode = tmp;
        Ok(())
    }

    fn get_addr(&self) -> String {
        self.addr.to_string()
    }

    fn print_prompt(&mut self) -> Result<()> {
        match self.shell_mode {
            ShellMode::None => Ok(()),
            ShellMode::Shell => self
                .stream
                .write_all(b"matt_daemon>")
                .map_err(Error::WriteToStream),
            ShellMode::Bash => self
                .stream
                .write_all(b"bash>")
                .map_err(Error::WriteToStream),
        }
    }
}

fn read_from_fd(
    fd: i32,
    idx: usize,
    clients: &mut [Client],
    logger: &TintinReporter,
) -> Result<bool> {
    let mut data = [0_u8; SIZE_BUFF];
    let client = clients.get_mut(idx - 1).ok_or(Error::ClientGetter)?;
    unsafe {
        let nb = get_err(
            libc::read(fd, data.as_mut_ptr() as _, SIZE_BUFF),
            Error::Read,
        )?;

        let msg =
            String::from_utf8(data[0..(nb as usize)].to_vec()).map_err(|_| Error::ConvertToUTF8)?;
        let msg = msg.trim().to_string();
        // TODO
        // if client.shell_mode > ShellMode::None {
        //     let res = handle_remote_shell(msg, client, logger);
        //     match res {
        //         Err(Error::CommandFailed(_)) => {
        //             eprintln!("Wrong Command");
        //             Ok(false)
        //         }
        //         Ok(_) => Ok(false),
        //         Err(e) => Err(e),
        //     }
        // } else {
        //     handle_client(msg, client, logger)
        // }
        // If true then exit connection with client
        Ok(true)
    }
}

fn handle_revent_error(a: i16, addr: String, logger: &TintinReporter) -> Result<()> {
    if a & libc::POLLHUP != 0 {
        logger.log(format!("Hanging up from {addr}\n"), LogInfo::Warn, false)?;
    } else if a & libc::POLLERR != 0 {
        logger.log(
            format!("Error condition from {addr}\n"),
            LogInfo::Warn,
            false,
        )?;
    } else if a & libc::POLLNVAL != 0 {
        logger.log(
            format!("Invalid request: fd not open from {addr}\n"),
            LogInfo::Warn,
            false,
        )?;
    } else {
        logger.log(
            format!("Stream socket peer closed connection from {addr}\n"),
            LogInfo::Warn,
            false,
        )?;
    }
    Ok(())
}

fn add_client(
    fds: &mut Connections,
    listener: &TcpListener,
    streams: &mut Vec<Client>,
    logger: &TintinReporter,
) -> Result<()> {
    let (stream, addr) = listener.accept().map_err(Error::AcceptClient)?;
    if fds.len() >= 3 {
        logger.log("Already 2 clients connected\n", LogInfo::Warn, false)?;
        return Ok(());
    }
    fds.push_from_fd(stream.as_raw_fd());
    logger.log(
        format!("Connecting to new address : {addr}\n"),
        LogInfo::Info,
        false,
    )?;
    streams.push(Client::new(stream, addr));
    Ok(())
}

pub fn server(logger: TintinReporter) -> Result<()> {
    let listener = TcpListener::bind(ADDRESS).map_err(Error::ClientErrorBinding)?;

    let listener_fd = listener.as_raw_fd();
    let mut fds = Connections::new();
    fds.push_from_fd(listener_fd);

    let mut clients: Vec<Client> = vec![];
    loop {
        unsafe {
            let _ = libc::poll(fds.as_mut_ptr(), fds.len() as u32, -1);
            for (i, poll_fd) in fds.clone().iter().enumerate() {
                let fd = poll_fd.fd;
                match poll_fd.revents {
                    0 => (),
                    a => {
                        if a & libc::POLLIN != 0 && a & libc::POLLRDHUP == 0 {
                            if fd == listener_fd {
                                add_client(&mut fds, &listener, &mut clients, &logger)?;
                            } else if read_from_fd(fd, i, &mut clients, &logger)? {
                                return Ok(());
                            }
                        }
                        if a & libc::POLLRDHUP != 0
                            || a & libc::POLLHUP != 0
                            || a & libc::POLLERR != 0
                        {
                            eprintln!("{a}");
                            let addr = clients
                                .get(i - 1)
                                .map(|client| client.get_addr())
                                .unwrap_or_else(|| "Can't find address".to_string());

                            let stream = clients.remove(i - 1);
                            drop(stream);
                            fds.remove(i);
                            handle_revent_error(a, addr, &logger)?;
                        }
                    }
                }
            }
        }
    }
}
