use daemonize::Result;
use std::{
    io::{self, BufRead, BufReader, BufWriter, Write},
    net::{SocketAddr, TcpStream},
    time::Duration,
};
use supervisor::{Action, ParseActionError};

use crate::{Client, Clients};

const NBR_CLIENT_MAX: usize = 3;
const PROMPT: &str = "\x1B[94mTaskMaster>\x1B[0m";
const READ_DURATION: Duration = Duration::from_millis(100);

impl Clients {
    pub(crate) fn add_client(&mut self, stream: TcpStream, addr: SocketAddr) -> Result<bool> {
        Ok(if self.clients.len() >= NBR_CLIENT_MAX {
            false
        } else {
            let mut new_client = Client::new(stream, addr)?;
            new_client.print_prompt()?;
            self.clients.push(new_client);
            true
        })
    }

    /// Go through every clients and try to read from them.
    /// Remove every clients that are not connected anymore
    /// Return true if one of the client ask to shut down the program
    pub(crate) fn read_clients(&mut self) -> Result<bool> {
        let mut to_clear = vec![];
        for (i, client) in self.clients.iter_mut().enumerate() {
            eprintln!("Reading client {:?}", i);

            match client.read_promt()? {
                ClientResponse::Continue => (),
                ClientResponse::Disconnected => to_clear.push(i),
                ClientResponse::Exit => return Ok(false),
            }
            client.print_prompt()?;
        }
        for i in to_clear.into_iter().rev() {
            eprintln!("removed client");
            self.clients.remove(i);
        }
        Ok(true)
    }
}

enum ClientResponse {
    Continue,
    Disconnected,
    Exit,
}

impl Client {
    fn new(stream: TcpStream, addr: SocketAddr) -> Result<Self> {
        stream.set_read_timeout(Some(READ_DURATION))?;
        Ok(Self {
            reader: BufReader::new(stream.try_clone()?),
            stream,
            addr,
            prompt_needed: true,
        })
    }

    fn print(&mut self, buf: &[u8]) -> Result<()> {
        let mut writer = BufWriter::new(&self.stream);
        writer.write_all(buf)?;
        Ok(())
    }
    // fn get_addr(&self) -> String {
    //     self.addr.to_string()
    // }
    /// Display the promt of taskmaster when needed
    fn print_prompt(&mut self) -> Result<()> {
        if self.prompt_needed {
            eprintln!("printing prompt");
            self.print(PROMPT.as_bytes())?;
            self.prompt_needed = false;
        }
        Ok(())
    }

    /// Print the error we got trying to parse the command given
    fn print_error(&mut self, e: ParseActionError) -> Result<()> {
        self.print(e.to_string().as_bytes())?;
        Ok(())
    }

    /// Try to read from the promt.
    /// Return a status corresponding of the what has been read.
    /// If nothing has been read, that means that the client has disconnected
    /// If `exit` was read, then the client wants the server to stop
    /// Otherwise, the server continues
    fn read_promt(&mut self) -> Result<ClientResponse> {
        let mut buf = String::new();
        match self.reader.read_line(&mut buf) {
            Err(e) => match e.kind() {
                io::ErrorKind::WouldBlock => (),
                _ => return Err(e.into()),
            },
            Ok(m) => {
                println!("Received {:?}, {:?}", m, buf);
                if m == 0 {
                    // doesn't reach here.
                    return Ok(ClientResponse::Disconnected);
                }
                let action: Action = match buf.try_into() {
                    Ok(a) => a,
                    Err(e) => {
                        self.print_error(e)?;
                        self.prompt_needed = true;
                        return Ok(ClientResponse::Continue);
                    }
                };
                if action == Action::Quit {
                    return Ok(ClientResponse::Exit);
                }
                // TODO
                // Send action to programs

                self.prompt_needed = true;
            }
        };
        Ok(ClientResponse::Continue)
    }
}
