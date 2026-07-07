use serde::Deserialize;

//================================================================

pub enum Command {
    Run(Argument),
    Help,
}

impl Command {
    pub const HELP_TEXT: &str = r#"
Nimbus - version {version}

  --file {path}   : Use a given server file.
  --port {port}   : Use a given number port.
  --uPnP          : Use Universal Plug and Play (uPnP) port forwarding.
  --verbose       : Log every client/server command.
  --server {name} : Run a given server ({name}.data/{name}.json pair).
  --help          : Display this help message.
"#;

    pub fn new() -> Self {
        let mut argument = Argument::default();
        let mut list = std::env::args();
        list.next();

        while let Some(next) = list.next() {
            match next.as_str() {
                "--file" => {
                    if let Some(file) = list.next() {
                        argument.file = file;
                    } else {
                        println!("missing argument \"{{file}}\" for command \"--file\".");
                    }
                }
                "--port" => {
                    if let Some(port) = list.next() {
                        if let Ok(port) = port.parse() {
                            argument.port = port;
                        } else {
                            println!(
                                "invalid numerical argument \"{next}\" for command \"--port\"."
                            );
                        }
                    } else {
                        println!("missing argument \"{{port}}\" for command \"--port\".");
                    }
                }
                "--uPnP" => {
                    argument.uPnP = true;
                }
                "--verbose" => {
                    argument.verbose = true;
                }
                "--server" => {
                    if let Some(path) = list.next() {
                        if let Ok(argument) = Argument::new(&path) {
                            return Self::Run(argument);
                        } else {
                            println!("could not load path \"{next}\" for command \"--server\".");
                        }
                    } else {
                        println!("missing argument \"{{path}}\" for command \"--server\".");
                    }
                }
                "--help" => return Self::Help,
                x => {
                    println!("unknown argument \"{x}\".");
                }
            }
        }

        Self::Run(argument)
    }
}

#[derive(Deserialize)]
pub struct Argument {
    pub file: String,
    pub port: u16,
    pub uPnP: bool,
    pub verbose: bool,
}

impl Argument {
    fn new(path: &str) -> anyhow::Result<Self> {
        Ok(serde_json::from_slice(&std::fs::read(path)?)?)
    }
}

impl Default for Argument {
    fn default() -> Self {
        Self {
            file: "server.data".to_string(),
            port: 8080,
            uPnP: false,
            verbose: false,
        }
    }
}
