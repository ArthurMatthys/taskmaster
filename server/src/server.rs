use core::time;
use daemonize::Result;
use libc::SIGCHLD;
use signal_hook::consts::{FORBIDDEN, TERM_SIGNALS};
use signal_hook::flag;
use signal_hook::iterator::exfiltrator::WithOrigin;
use signal_hook::iterator::SignalsInfo;
use std::io::{self, BufRead, BufReader};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender};
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::Duration;

const ADDRESS: &str = "127.0.0.1:4242";
const NBR_CLIENT_MAX: usize = 3;
const READ_DURATION: Duration = Duration::from_millis(100);

pub struct Client {
    stream: TcpStream,
    addr: SocketAddr,
    prompt_needed: bool,
}

#[derive(Default)]
pub struct Clients {
    clients: Vec<Client>,
}

impl Clients {
    fn add_client(&mut self, stream: TcpStream, addr: SocketAddr) -> Result<bool> {
        Ok(if self.clients.len() >= NBR_CLIENT_MAX {
            false
        } else {
            self.clients.push(Client::new(stream, addr)?);
            true
        })
    }

    /// Go through every clients and try to read from them.
    /// Remove every clients that are not connected anymore
    fn read_clients(&mut self) -> Result<()> {
        let mut to_clear = vec![];
        for (i, client) in self.clients.iter().enumerate() {
            eprintln!("Reading client {:?}", i);

            if !client.read_promt()? {
                to_clear.push(i);
            }
        }
        for i in to_clear.into_iter().rev() {
            eprintln!("removed client");
            self.clients.remove(i);
        }
        Ok(())
    }
}

impl Client {
    fn new(stream: TcpStream, addr: SocketAddr) -> Result<Self> {
        stream.set_read_timeout(Some(READ_DURATION))?;
        Ok(Self {
            stream,
            addr,
            prompt_needed: true,
        })
    }

    // fn get_addr(&self) -> String {
    //     self.addr.to_string()
    // }

    /// Try to read from the promt.
    /// Return Error if there was an issue reading the stream
    /// Return false if we read 0 bytes (i-e the client is disconnected),
    /// otherwise return true
    fn read_promt(&self) -> Result<bool> {
        let mut reader = BufReader::new(&self.stream);
        let mut buf = String::new();
        match reader.read_line(&mut buf) {
            Err(e) => match e.kind() {
                io::ErrorKind::WouldBlock => (),
                _ => return Err(e.into()),
            },
            Ok(m) => {
                println!("Received {:?}, {:?}", m, buf);
                if m == 0 {
                    // doesn't reach here.
                    return Ok(false);
                }
            }
        };
        Ok(true)
    }
}

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
    let listener = TcpListener::bind(ADDRESS)?;
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
                clients.add_client(stream, addr)?;
            }
            Err(e) => {
                if e.kind() != io::ErrorKind::WouldBlock {
                    eprintln!("Error : {:?}", e)
                }
            }
        }

        clients.read_clients()?;

        // check status of children
        // check_child_status

        thread::sleep(time::Duration::from_millis(300));
    }
    // thread.join().expect("joining thread");
}
