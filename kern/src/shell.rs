use stack_vec::StackVec;

use crate::console::{kprint, kprintln, CONSOLE};

/// Error type for `Command` parse failures.
#[derive(Debug)]
enum Error {
    Empty,
    TooManyArgs,
}

/// A structure representing a single shell command.
struct Command<'v, 's> {
    args: StackVec<'v, &'s str>,
}

impl<'v, 's> Command<'v, 's> {
    /// Parse a command from a string `s` using `buf` as storage for the
    /// arguments.
    ///
    /// # Errors
    ///
    /// If `s` contains no arguments, returns `Error::Empty`. If there are more
    /// arguments than `buf` can hold, returns `Error::TooManyArgs`.
    fn parse(s: &'s str, buf: &'v mut [&'s str]) -> Result<Command<'v, 's>, Error> {
        let mut args = StackVec::new(buf);
        for arg in s.split(' ').filter(|a| !a.is_empty()) {
            args.push(arg).map_err(|_| Error::TooManyArgs)?;
        }

        if args.is_empty() {
            return Err(Error::Empty);
        }

        Ok(Command { args })
    }

    /// Returns this command's path. This is equivalent to the first argument.
    fn path(&self) -> &str {
        self.args[0]
    }
}

/// The maximum number of bytes that can fit in a command
const MAX_COMMAND_LEN: usize = 512;
/// The max number of arguments that a command can take
const MAX_ARGUMENTS: usize = 64;

/// Starts a shell using `prefix` as the prefix for each line. This function
/// returns if the `exit` command is called.
pub fn shell(prefix: &str) -> ! {
    // Each visible character entered will be buffered here
    let mut input_buf = [0u8; MAX_COMMAND_LEN];

    loop {
        // Set aside some memory to hold the argument strings the user enters.
        // We need to reset this every loop, or else there will be dangling
        // references to input_buf  left on every run through.
        let mut command_buf = [""; MAX_ARGUMENTS];

        kprint!("{} ", prefix);

        let input = match read_next_line(&mut input_buf) {
            Ok(s) => s,
            Err(e) => {
                kprintln!("Error parsing input: {}", e);
                continue;
            }
        };

        let command = match Command::parse(input, &mut command_buf) {
            Ok(command) => command,
            Err(e) => {
                match e {
                    Error::Empty => (),
                    Error::TooManyArgs => {
                        kprintln!("Error: too many arguments");
                    }
                };
                continue;
            }
        };

        match command.path() {
            "echo" => {
                echo(command);
            }
            s => {
                kprintln!("unknown command: {}", s);
            }
        };
    }
}

/// Reads the next line of input into a `str`, using the provided buffer as storage.
fn read_next_line(buf: &mut [u8]) -> Result<&str, core::str::Utf8Error> {
    let mut input = StackVec::new(buf);

    loop {
        let next_byte = CONSOLE.lock().read_byte();

        // we check is_full and is_empty in the conditionals so we always fall back on the bell
        if (next_byte.is_ascii_graphic() || next_byte == b' ') && !input.is_full() {
            kprint!("{}", next_byte as char);
            input.push(next_byte).unwrap();
        } else if next_byte == b'\r' || next_byte == b'\n' {
            kprintln!("");
            break;
        } else if (next_byte == 8 || next_byte == 127) && !input.is_empty() {
            kprint!("\u{8} \u{8}"); // remove from the screen
            let _ = input.pop();
        } else {
            kprint!("\u{7}");
        }
    }

    core::str::from_utf8(input.into_slice())
}

/// A simple echo program, printing arguments passed into the program.
///
/// Eventually this will be a separate binary, but for now the kernel can't handle that.
fn echo(command: Command) {
    // every word but the first will get a leading space
    let mut use_leading_space = false;
    for arg in &command.args[1..] {
        let prefix = if use_leading_space { " " } else { "" };
        kprint!("{}{}", prefix, arg);
        use_leading_space = true;
    }

    kprintln!("");
}
