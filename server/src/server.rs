use core::time;
use daemonize::Result;
use libc::{SIGCHLD, SIGHUP};
use signal_hook::consts::{FORBIDDEN, TERM_SIGNALS};
use signal_hook::flag;
use signal_hook::iterator::exfiltrator::WithOrigin;
use signal_hook::iterator::SignalsInfo;
use std::net::TcpListener;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender};
use std::sync::{mpsc, Arc};
use std::{io, thread};

use supervisor::Programs;

use crate::Clients;

/// Send any signal received into a channel for the main loop to deal with.
fn register_signal_hook(sender: Sender<i32>) -> Result<()> {
    let term_now = Arc::new(AtomicBool::new(false));
    for sig in TERM_SIGNALS {
        flag::register_conditional_shutdown(*sig, 1, Arc::clone(&term_now))?;
        flag::register(*sig, Arc::clone(&term_now))?;
    }
    let mut forbidden = vec![SIGCHLD];
    forbidden.extend(FORBIDDEN);
    let sigs = (1..32).filter(|s| !forbidden.contains(s));
    let mut signals = SignalsInfo::<WithOrigin>::new(sigs)?;
    let handle = signals.handle();

    for info in &mut signals {
        sender.send(info.signal)?;
    }

    handle.close();
    Ok(())
}

pub fn server() -> Result<()> {
    let mut programs = Programs::new(true)?;

    let addr = match std::env::var("SERVER_ADDRESS") {
        Ok(addr) => addr,
        Err(_) => {
            logger::log(
                "SERVER_ADDRESS environment variable is not set, using localhost:4242 default",
                logger::LogInfo::Error,
            )?;
            "127.0.0.1:4242".to_string()
        }
    };

    let listener = TcpListener::bind(addr)?;
    let (tx, rx): (Sender<i32>, Receiver<i32>) = mpsc::channel();

    let _ = thread::spawn(|| register_signal_hook(tx));

    listener.set_nonblocking(true)?;
    let mut clients = Clients::default();

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
                Ok(SIGHUP) => programs = programs.update_config()?, // sigup et down to handle here
                Ok(sig) => {
                    logger::log(
                        format!("Received signal {} on server", sig),
                        logger::LogInfo::Info,
                    )?;
                    break;
                }

                Err(RecvTimeoutError::Timeout) => break,
                Err(e) => {
                    logger::log(
                        format!("Signal handling error : {:?}", e),
                        logger::LogInfo::Error,
                    )?;
                    break;
                }
            }
        }

        match listener.accept() {
            Ok((stream, addr)) => {
                clients.add_client(stream, addr)?;
            }
            Err(e) => {
                if e.kind() != io::ErrorKind::WouldBlock {
                    eprintln!("Error : {:?}", e)
                }
            }
        }

        if !clients.read_clients(&mut programs)? {
            logger::log("Exiting server".to_string(), logger::LogInfo::Info)?;
            break;
        };

        // check status of children
        // check_child_status
        programs.check()?;

        thread::sleep(time::Duration::from_millis(300));
    }
    Ok(())
}
