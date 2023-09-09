// use crate::controller::load_config;
// use crate::model::{Args, Error, Programs};
// use std::io::Read;
// use std::net::Shutdown;
// use std::time::Duration;
// use std::{
//     fs::File,
//     io::{stdin, stdout, BufReader, Write},
//     thread,
// };

// use libc::getppid;
// use nix::sys::signal::{self, signal};
// use nix::{
//     self,
//     sys::ptrace,
//     unistd::{self, fork, ForkResult},
//     Result,
// };
// use std::os::unix::net::{UnixListener, UnixStream};

// const SOCKET: &str = "/tmp/taskmaster_socket.sock";

// fn handle_instruction(mut stream: UnixStream) -> std::io::Result<()> {
//     let mut msg = String::new();
//     stream.read_to_string(&mut msg)?;

//     println!("Message received : {msg}");

//     stream.write(b"Received")?;

//     Ok(())
// }

// fn wait_instructions() -> std::io::Result<()> {
//     if std::fs::metadata(SOCKET).is_ok() {
//         println!("A socket is already present. Deleting...");
//         std::fs::remove_file(SOCKET)?;
//     }
//     let unix_listener = UnixListener::bind(SOCKET).expect("Could not create the unix socket");
//     // match unix_listener.set_nonblocking(true) {
//     //     Ok(_) => (),
//     //     Err(e) => println!("{e}"),
//     // };
//     for stream in unix_listener.incoming() {
//         match stream {
//             Ok(stream) => {
//                 let handler = thread::spawn(|| handle_instruction(stream));
//                 handler.join().unwrap();
//             }
//             Err(err) => {
//                 break;
//             }
//         }
//     }
//     match signal::killpg(unistd::getpid(), signal::SIGKILL) {
//         Ok(_) => (),
//         Err(e) => {
//             println!("Failed to kill daemon : {e}")
//         }
//     };
//     Ok(())
// }

// fn handle_cli() -> std::io::Result<()> {
//     let mut stdout = stdout();
//     let mut unix_stream;
//     loop {
//         let tmp = UnixStream::connect(SOCKET);
//         match tmp {
//             Ok(stream) => {
//                 unix_stream = stream;
//                 break;
//             }
//             Err(e) => thread::sleep(Duration::from_secs(1)),
//         }
//     }
//     loop {
//         print!("Supervisor> ");
//         stdout.flush()?;

//         let mut input = String::new();
//         stdin().read_line(&mut input).unwrap();

//         unix_stream.write(b"Hello")?;

//         match unix_stream.flush() {
//             Ok(_) => (),
//             Err(e) => println!("flushed : {e}"),
//         }
//         // stream.write_all(b"hello world")?;
//         let mut parts = input.split_whitespace();
//         let cmd = if let Some(cmd) = parts.next() {
//             cmd.to_lowercase()
//         } else {
//             eprintln!("No command supplied");
//             continue;
//         };
//         // let args = parts;
//         match cmd.as_str() {
//             // "status" => programs.status(),
//             // "start" => programs.action("start", args),
//             // "stop" => programs.action("stop", args),
//             // "relaunch" => programs.action("relaunch", args),
//             // "reload" => programs = load_config(args, &programs)?,
//             "exit" | "quit" => break,
//             _ => {
//                 eprintln!("Supervisor: Unknown command : {cmd}.");
//                 continue;
//             }
//         };
//     }
//     // unix_stream.shutdown(Shutdown::Read);
//     Ok(())
// }

// fn create_deamon() -> Result<()> {
//     match fork() {
//         Ok(ForkResult::Parent { child, .. }) => {
//             get_info("Parent")?;
//             // handle_cli();
//             let handler = thread::spawn(|| handle_cli());
//             handler.join().unwrap();
//         }
//         Ok(ForkResult::Child) => {
//             let _ = unistd::setsid()?;
//             let child = match fork() {
//                 Ok(ForkResult::Parent { child, .. }) => {
//                     get_info("First Child")?;
//                     child
//                 }
//                 Ok(ForkResult::Child) => {
//                     get_info("Second Child")?;

//                     // wait_instructions();
//                     let handler = thread::spawn(|| wait_instructions());
//                     handler.join().unwrap();
//                     unistd::getpid()
//                 }
//                 Err(_) => {
//                     println!("Fork failed");
//                     return Ok(());
//                 }
//             };
//             match ptrace::kill(child) {
//                 Ok(_) => println!("Second child killed"),
//                 Err(e) => println!("Second child not killed : {e}"),
//             };
//         }
//         Err(_) => {
//             println!("Fork failed")
//         }
//     };
//     Ok(())
// }

// pub fn supervisor(args: Args) -> Result<()> {
//     create_deamon();
//     // let mut programs = match File::open(args.path) {
//     //     Ok(f) => {
//     //         let rdr = BufReader::new(f);
//     //         match serde_yaml::from_reader::<_, Programs>(rdr) {
//     //             Ok(p) => p,
//     //             Err(err) => return Err(Error::De(format!("{err}"))),
//     //         }
//     //     }
//     //     Err(err) => return Err(Error::Read(format!("{err}"))),
//     // };
//     // let mut stdout = stdout();

//     // println!("{programs:#?}");
//     // loop {
//     //     print!("Supervisor> ");
//     //     stdout.flush();

//     //     let mut input = String::new();
//     //     stdin().read_line(&mut input).unwrap();

//     //     let mut parts = input.split_whitespace();
//     //     let cmd = if let Some(cmd) = parts.next() {
//     //         cmd.to_lowercase()
//     //     } else {
//     //         eprintln!("No command supplied");
//     //         continue;
//     //     };

//     //     let args = parts;
//     //     match cmd.as_str() {
//     //         "status" => programs.status(),
//     //         "start" => programs.action("start", args),
//     //         "stop" => programs.action("stop", args),
//     //         "relaunch" => programs.action("relaunch", args),
//     //         "reload" => programs = load_config(args, &programs)?,
//     //         "exit" | "quit" => break,
//     //         _ => {
//     //             eprintln!("Supervisor: Unknown command : {cmd}.");
//     //             continue;
//     //         }
//     //     };
//     // }
//     Ok(())
// }
