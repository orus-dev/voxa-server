use std::{
    fs::{self, File},
    path::PathBuf,
    str::FromStr,
    sync::Arc,
};

use rustyline::{DefaultEditor, error::ReadlineError};
use zip::ZipArchive;

use crate::{logger, plugin::loader::PluginLoader, server::Server};

logger!(LOGGER "CLI");

macro_rules! commands {
    (($cmdi:expr) $($cmd:literal $desc:literal => $body:block)*) => {
        match $cmdi {
            "help" => {
                LOGGER.info("List of commands:");
                $(
                    println!("\x1b[34m{}\x1b[0m: {}", $cmd, format!($desc));
                )*
            },
            $($cmd => $body)*
            _ => LOGGER.error(format!("Command not found: {}", $cmdi)),
        }
    };
}

pub fn parse_args(input: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();

    let mut in_dq = false;
    let mut in_q = false;
    let mut escape = false;

    for c in input.chars() {
        if escape {
            current.push(c);
            escape = false;
            continue;
        }

        match c {
            '\\' => {
                escape = true;
            }
            '"' if !in_q && !escape => {
                in_dq = !in_dq;
            }
            '\'' if !in_dq && !escape => {
                in_q = !in_q;
            }
            ' ' if !in_dq && !in_q => {
                if !current.is_empty() {
                    args.push(current.clone());
                    current.clear();
                }
            }
            _ => {
                current.push(c);
            }
        }
    }

    if escape {
        current.push('\\');
    }

    if !current.is_empty() {
        args.push(current);
    }

    args
}

pub fn require_args(args: &[String], required: &[&str]) -> bool {
    if required.len() + 1 > args.len() {
        LOGGER.error(format!("Required arguments: {}", required.join(" ")));
        return false;
    }

    true
}

pub fn start_cli(server: Arc<Server>, plugin_loader: PluginLoader) {
    std::thread::spawn(move || {
        let mut rl = DefaultEditor::new().unwrap();

        loop {
            let line = match rl.readline("\n> ") {
                Ok(line) => {
                    rl.add_history_entry(line.as_str()).ok();
                    line
                }
                Err(ReadlineError::Interrupted) => continue,
                Err(ReadlineError::Eof) => break,
                Err(err) => {
                    LOGGER.error(format!("CLI error: {:?}", err));
                    break;
                }
            };

            let args = parse_args(line.trim());
            if args.is_empty() {
                continue;
            }

            commands! {
                (args[0].as_str())
                "install" "Installs the specified plugin .vxp package" => {
                    if require_args(&args, &["<path.vxp>"]) {
                        let path = PathBuf::from_str(&args[1]).unwrap();
                        if path.extension().and_then(|s| s.to_str()) == Some("vxp") {
                            LOGGER.info(format!("Installing {:?}", path));
                            let d = path.with_extension("");
                            if !d.exists() {
                                let file = File::open(&path).unwrap();
                                let mut archive = ZipArchive::new(file).unwrap();
                                archive.extract(&d).unwrap();
                            }
                            LOGGER.info("Installed");
                        } else {
                            LOGGER.error("File must be .vxp");
                        }
                    }
                }
                "load" "loads a plugin that has been installed" => {
                    if require_args(&args, &["<plugin-id>"]) {
                        let mut p = plugin_loader.load(&server.root.join("plugins").join(&args[1]));
                        let s = server.clone();
                        server.plugins.lock().unwrap().push(p.clone());
                        std::thread::spawn(move || p.run(&s));
                    }
                }
                "stop" "stops a plugin that has been installed" => {
                    if require_args(&args, &["<plugin-id>"]) {
                        let target = args[1].as_str();
                        let mut stopped_plugins = Vec::new();

                        plugin_loader.remove(target);
                        for (i, plugin) in server.plugins.lock().unwrap().iter_mut().enumerate() {
                            if plugin.get_id() == target {
                                LOGGER.info("Stopping plugin");
                                if let Err(e) = plugin.stop() {
                                    LOGGER.warn(e.context("Couldn't stop plugin"));
                                }
                                stopped_plugins.push(i);
                            }
                        }

                        for i in stopped_plugins {
                            server.plugins.lock().unwrap().remove(i);
                        }
                        LOGGER.info("Plugin stopped!");
                    }
                }
                "reload" "restarts a plugin that has been installed" => {
                    if require_args(&args, &["<plugin-id>"]) {
                        let target = args[1].as_str();
                        let mut stopped_plugins = Vec::new();

                        plugin_loader.remove(target);
                        for (i, plugin) in server.plugins.lock().unwrap().iter_mut().enumerate() {
                            if plugin.get_id() == target {
                                LOGGER.info("Stopping plugin");
                                if let Err(e) = plugin.stop() {
                                    LOGGER.warn(e.context("Couldn't stop plugin"));
                                }
                                stopped_plugins.push(i);
                            }
                        }

                        for i in stopped_plugins {
                            server.plugins.lock().unwrap().remove(i);
                        }

                        let mut p = plugin_loader.load(&server.root.join("plugins").join(&args[1]));
                        let s = server.clone();
                        server.plugins.lock().unwrap().push(p.clone());
                        std::thread::spawn(move || p.run(&s));
                    }
                }
                "reload-all" "reloads all of the plugins" => {
                    LOGGER.info("Stopping all plugins");
                    for plugin in server.plugins.lock().unwrap().iter_mut() {
                        LOGGER.info(format!("Stopping plugin '{}'", plugin.get_id()));
                        if let Err(e) = plugin.stop() {
                            LOGGER.warn(e.context(format!("Couldn't stop plugin '{}'", plugin.get_id())));
                        }
                    }

                    server.plugins.lock().unwrap().clear();
                    plugin_loader.clear();

                    plugin_loader.load_all(&server);
                }
                "shutdown" "Softly shuts the server down, may not fully shut everything down" => {
                    server.shutdown();
                    break;
                }

                "ping" "replies with pong" => {
                    LOGGER.info("pong");
                }
            };
        }
    });
}
