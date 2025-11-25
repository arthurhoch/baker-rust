use crate::logger::Logger;
use crate::settings::VERSION;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct Options {
    pub verbose: bool,
    pub command: Command,
}

#[derive(Debug, Clone)]
pub enum Command {
    Help,
    Version,
    Configs { all: bool },
    Encrypt { plantexts: Vec<String>, file: Option<String> },
    GenKey { keypass: String },
    Pull { name: String, force: bool },
    Recipes { all: bool },
    Rm { recipe_id: String },
    Run { name: Option<String>, path: Option<String>, force: bool },
}

pub fn parse(args: &[String], logger: &Logger) -> Result<Options, Box<dyn Error>> {
    let mut args = args.to_vec();
    if args.is_empty() {
        print_help();
        return Ok(Options {
            verbose: false,
            command: Command::Help,
        });
    }

    let mut verbose = false;
    args.retain(|a| {
        if a == "--verbose" {
            verbose = true;
            false
        } else {
            true
        }
    });

    let cmd = match args[0].as_str() {
        "-h" | "--help" => Command::Help,
        "-v" | "--version" => Command::Version,
        "configs" => Command::Configs {
            all: args.get(1).map_or(false, |v| v == "-a" || v == "--all"),
        },
        "encrypt" => {
            let mut file = None;
            let mut plantexts = Vec::new();
            let mut idx = 1;
            while idx < args.len() {
                if args[idx] == "--file" {
                    idx += 1;
                    file = Some(
                        args.get(idx)
                            .ok_or("encrypt --file expects a path value")?
                            .to_string(),
                    );
                } else {
                    plantexts.push(args[idx].to_string());
                }
                idx += 1;
            }
            Command::Encrypt { plantexts, file }
        }
        "genkey" => {
            let keypass = args
                .get(1)
                .ok_or("genkey expects <keypass> argument")?
                .to_string();
            Command::GenKey { keypass }
        }
        "pull" => {
            let name = args.get(1).ok_or("pull expects <name> argument")?.to_string();
            let force = args.iter().any(|a| a == "-f" || a == "--force");
            Command::Pull { name, force }
        }
        "recipes" => Command::Recipes {
            all: args.iter().any(|a| a == "-a" || a == "--all"),
        },
        "rm" => {
            let recipe_id = args.get(1).ok_or("rm expects <recipe_id>")?.to_string();
            Command::Rm { recipe_id }
        }
        "run" => {
            let mut name: Option<String> = None;
            let mut path: Option<String> = None;
            let mut force = false;
            let mut idx = 1;
            while idx < args.len() {
                match args[idx].as_str() {
                    "-f" | "--force" => force = true,
                    "--path" => {
                        idx += 1;
                        path = Some(
                            args.get(idx)
                                .ok_or("run --path expects a recipe path")?
                                .to_string(),
                        );
                    }
                    other => {
                        if !other.starts_with('-') && name.is_none() {
                            name = Some(other.to_string());
                        }
                    }
                }
                idx += 1;
            }
            if name.is_some() && path.is_some() {
                return Err("run does not support both name and --path together".into());
            }
            Command::Run { name, path, force }
        }
        other => {
            logger.log(&format!("Unknown command '{}'", other));
            Command::Help
        }
    };

    Ok(Options {
        verbose,
        command: cmd,
    })
}

pub fn print_help() {
    println!(
        "baker-rust {}\n\
usage: baker [--verbose] <COMMAND> ...\n\n\
commands:\n  configs      list of configs\n  encrypt      encrypt values using secret key\n  genkey       generate a secret key from a key pass\n  pull         pull a recipe with configurations\n  recipes      list recipes locally\n  rm           remove recipes locally\n  run          run configurations from a recipe\n\n\
Run 'baker COMMAND --help' for more info on a command",
        VERSION
    );
}
