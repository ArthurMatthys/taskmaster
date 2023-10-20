use daemonize::Result;
use logger::{log, LogInfo};
use std::{
    io::{self, BufRead, BufReader, BufWriter, Write},
    net::{SocketAddr, TcpStream},
    time::Duration,
};
use supervisor::{Action, ParseActionError, Programs};

use crate::{Client, Clients};

const NBR_CLIENT_MAX: usize = 3;
const READ_DURATION: Duration = Duration::from_millis(100);

impl Clients {
    pub(crate) fn add_client(&mut self, stream: TcpStream, addr: SocketAddr) -> Result<bool> {
        Ok(if self.clients.len() >= NBR_CLIENT_MAX {
            log(
                format!(
                    "There can be at most {} clients connected\n",
                    NBR_CLIENT_MAX
                ),
                LogInfo::Warn,
            )?;
            false
        } else {
            let new_client = Client::new(stream, addr)?;
            log(
                format!("Connecting to new client with address {}\n", addr),
                LogInfo::Info,
            )?;
            self.clients.push(new_client);
            true
        })
    }

    /// Go through every clients and try to read from them.
    /// Remove every clients that are not connected anymore
    /// Return true if one of the client ask to shut down the program
    pub(crate) fn read_clients(&mut self, programs: &mut Programs) -> Result<bool> {
        let mut to_clear = vec![];
        for (i, client) in self.clients.iter_mut().enumerate() {
            match client.read_promt(programs)? {
                ClientResponse::Continue => (),
                ClientResponse::Disconnected => to_clear.push(i),
                ClientResponse::Exit => return Ok(false),
            }
        }
        for i in to_clear.into_iter().rev() {
            log(
                format!(
                    "Disconnecting form client with address {:?}\n",
                    self.clients
                        .get(i)
                        .map(|c| c.addr.to_string())
                        .unwrap_or_default()
                ),
                LogInfo::Info,
            )?;
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
        })
    }

    fn print(&mut self, buf: &[u8]) -> Result<()> {
        let mut writer = BufWriter::new(&self.stream);
        writer.write_all(buf)?;
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
    fn read_promt(&mut self, programs: &mut Programs) -> Result<ClientResponse> {
        let mut buf = String::new();
        match self.reader.read_line(&mut buf) {
            Err(e) => match e.kind() {
                io::ErrorKind::WouldBlock => (),
                _ => return Err(e.into()),
            },
            Ok(m) => {
                if m == 0 {
                    // doesn't reach here.
                    return Ok(ClientResponse::Disconnected);
                }
                let action: Action = match buf.try_into() {
                    Ok(a) => a,
                    Err(e) => {
                        self.print_error(e)?;
                        return Ok(ClientResponse::Continue);
                    }
                };

                if action == Action::Quit {
                    return Ok(ClientResponse::Exit);
                };

                self.print(programs.handle_action(action)?.as_bytes())?;
                // TODO
                // Send action to programs
            }
        };
        Ok(ClientResponse::Continue)
    }
}
