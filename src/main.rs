use anyhow::Result;
use clap::{Parser, Subcommand};
use std::io::{self, Read, Write};
use std::process;

mod env_file;
use env_file::EnvFile;

#[derive(Parser)]
#[command(name = "envq")]
#[command(about = "A jq/yq-like tool for .env files", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    List {
        /// arguments: [(keys)|values] [file]
        args: Vec<String>,
    },
    Get {
        /// arguments: [(key)|comment|header] [key] [file]
        args: Vec<String>,
    },
    Set {
        /// arguments: [(key)|comment|header] [key] value [file]
        args: Vec<String>,
    },
    Del {
        /// arguments: [(key)|comment|header] [key] [file]
        args: Vec<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::List { args } => {
            let (list_mode, file) = parse_list_args(&args)?;
            let content = read_input(file)?;
            let env_file = EnvFile::parse(&content)?;
            match list_mode {
                ListMode::Keys => {
                    for key in env_file.list_keys() {
                        println!("{}", key);
                    }
                }
                ListMode::Values => {
                    for key in env_file.list_keys() {
                        if let Some(value) = env_file.get_value(key) {
                            println!("{}={}", key, value);
                        }
                    }
                }
            }
        }
        Commands::Get { args } => {
            let (target, file) = parse_get_del_args(&args)?;
            let content = read_input(file)?;
            let env_file = EnvFile::parse(&content)?;

            let found = match target {
                Target::Key(key) => {
                    if let Some(value) = env_file.get_value(key) {
                        println!("{}", value);
                        true
                    } else {
                        false
                    }
                }
                Target::Comment(key) => {
                    // we need to check if the key exists
                    if env_file.get_value(key).is_some() {
                        if let Some(comment) = env_file.get_comment(key) {
                            println!("{}", comment);
                        }
                        true
                    } else {
                        false
                    }
                }
                Target::Header => {
                    if let Some(header) = env_file.get_header() {
                        print!("{}", header);
                    }
                    true
                }
            };

            if !found {
                process::exit(1);
            }
        }
        Commands::Set { args } => {
            let (target, value, file) = parse_set_args(&args)?;
            let content = read_input(file)?;
            let mut env_file = EnvFile::parse(&content)?;

            match target {
                Target::Key(key) => {
                    env_file.set_value(key, &value);
                }
                Target::Comment(key) => {
                    env_file.set_comment(key, &value);
                }
                Target::Header => {
                    env_file.set_header(&value);
                }
            }

            write_output(file, &env_file.to_string())?;
        }
        Commands::Del { args } => {
            let (target, file) = parse_get_del_args(&args)?;
            let content = read_input(file)?;
            let mut env_file = EnvFile::parse(&content)?;

            match target {
                Target::Key(key) => {
                    env_file.delete_key(key);
                }
                Target::Comment(key) => {
                    env_file.delete_comment(key);
                }
                Target::Header => {
                    env_file.delete_header();
                }
            }

            write_output(file, &env_file.to_string())?;
        }
    }

    Ok(())
}

enum ListMode {
    Keys,
    Values,
}

enum Target<'a> {
    Key(&'a str),
    Comment(&'a str),
    Header,
}

fn parse_list_args(args: &[String]) -> Result<(ListMode, Option<&str>)> {
    if args.is_empty() {
        // envq list (defaults to values)
        return Ok((ListMode::Values, None));
    }

    let first = args[0].as_str();
    match first {
        "keys" => {
            // envq list keys [file]
            let file = args.get(1).map(|s| s.as_str());
            Ok((ListMode::Keys, file))
        }
        "values" => {
            // envq list values [file]
            let file = args.get(1).map(|s| s.as_str());
            Ok((ListMode::Values, file))
        }
        _ => {
            // envq list [file] (defaults to values mode)
            Ok((ListMode::Values, Some(first)))
        }
    }
}

fn parse_get_del_args(args: &[String]) -> Result<(Target<'_>, Option<&str>)> {
    if args.is_empty() {
        return Err(anyhow::anyhow!(
            "You need to provide what to get [key|comment|header].\nExample: envq get key FOO"
        ));
    }

    let first = args[0].as_str();

    match first {
        "header" => {
            // envq get/del header [file]
            let file = args.get(1).map(|s| s.as_str());
            Ok((Target::Header, file))
        }
        "comment" => {
            // envq get/del comment KEY [file]
            if args.len() < 2 {
                return Err(anyhow::anyhow!(
                    "You need to provide the name of key.\nExample: envq get comment FOO"
                ));
            }
            let key = args[1].as_str();
            let file = args.get(2).map(|s| s.as_str());
            Ok((Target::Comment(key), file))
        }
        "key" => {
            // envq get/del key KEY [file]
            if args.len() < 2 {
                return Err(anyhow::anyhow!(
                    "You need to provide the name of key.\nExample: envq get key FOO"
                ));
            }
            let key = args[1].as_str();
            let file = args.get(2).map(|s| s.as_str());
            Ok((Target::Key(key), file))
        }
        _ => {
            // envq get/del KEY [file]
            let file = args.get(1).map(|s| s.as_str());
            Ok((Target::Key(first), file))
        }
    }
}

fn parse_set_args(args: &[String]) -> Result<(Target<'_>, String, Option<&str>)> {
    if args.is_empty() {
        return Err(anyhow::anyhow!(
            "You need to provide what to set [key|comment|header].\nExample: envq set key FOO VALUE"
        ));
    }

    let first = args[0].as_str();

    match first {
        "header" => {
            // envq set header VALUE [file]
            if args.len() < 2 {
                return Err(anyhow::anyhow!(
                    "You need to provide a value for header.\nExample: envq set header VALUE"
                ));
            }
            let value = args[1].clone();
            let file = args.get(2).map(|s| s.as_str());
            Ok((Target::Header, value, file))
        }
        "comment" => {
            // envq set comment KEY VALUE [file]
            if args.len() < 3 {
                return Err(anyhow::anyhow!(
                    "You need to provide the key and value.\nExample: envq set comment FOO VALUE"
                ));
            }
            let key = args[1].as_str();
            let value = args[2].clone();
            let file = args.get(3).map(|s| s.as_str());
            Ok((Target::Comment(key), value, file))
        }
        "key" => {
            // envq set key KEY VALUE [file]
            if args.len() < 3 {
                return Err(anyhow::anyhow!(
                    "You need to provide the key and value.\nExample: envq set key FOO VALUE"
                ));
            }
            let key = args[1].as_str();
            let value = args[2].clone();
            let file = args.get(3).map(|s| s.as_str());
            Ok((Target::Key(key), value, file))
        }
        _ => {
            // envq set KEY VALUE [file]
            if args.len() < 2 {
                return Err(anyhow::anyhow!(
                    "You need to provide a value.\nExample: envq set FOO VALUE"
                ));
            }
            let value = args[1].clone();
            let file = args.get(2).map(|s| s.as_str());
            Ok((Target::Key(first), value, file))
        }
    }
}

fn read_input(file_path: Option<&str>) -> Result<String> {
    match file_path {
        Some(path) => Ok(std::fs::read_to_string(path)?),
        None => {
            // check if stdin is a terminal (no piped input)
            if atty::is(atty::Stream::Stdin) {
                return Err(anyhow::anyhow!("Missing file or stdin."));
            }
            let mut buffer = String::new();
            io::stdin().lock().read_to_string(&mut buffer)?;
            Ok(buffer)
        }
    }
}

fn write_output(file_path: Option<&str>, content: &str) -> Result<()> {
    match file_path {
        Some(path) => {
            let temp_path = format!("{}.tmp", path);
            std::fs::write(&temp_path, content)?;
            std::fs::rename(&temp_path, path)?;
        }
        None => {
            io::stdout().write_all(content.as_bytes())?;
        }
    }
    Ok(())
}
