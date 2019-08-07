use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use question::{Question, Answer};
use serde_json;

use crate::{SmoothlyError, Command, Repo, Server};

pub struct New {}
impl Command for New {
    fn register(&self) -> (&str, clap::App) {
        ("new",
            clap::SubCommand::with_name("new")
                .about("Create a new Swifty repository")
                .arg(clap::Arg::with_name("name")
                    .help("Repository Name")
                ).arg(clap::Arg::with_name("path")
                    .help("Path to the mods")
                )
        )
    }

    fn run(&self, args: &clap::ArgMatches, repopath: String) -> Result<(), SmoothlyError> {
        let repo = Repo {
            repoName: if let Some(name) = args.value_of("name") {
                name.to_owned()
            } else {
                let mut x = String::new();
                while x.is_empty() {
                    x = if let Answer::RESPONSE(n) = Question::new("Name:").ask().unwrap() { n } else { unreachable!() };
                }
                x
            },
            basePath: if let Some(path) = args.value_of("path") {
                path.to_owned()  
            } else {
                let mut x = String::new();
                while x.is_empty() || !PathBuf::from(&x).exists() {
                    x = if let Answer::RESPONSE(n) = Question::new("Path:").ask().unwrap() { n } else { unreachable!() };
                }
                x
            },
            clientParameters: "-noSplash -skipIntro".to_owned(),
            optionalMods: Vec::new(),
            requiredMods: Vec::new(),
            servers: {
                let mut servers = Vec::new();
                while Question::new("Add a server?").default(Answer::YES).show_defaults().confirm() == question::Answer::YES {
                    servers.push(Server {
                        name: {
                            let mut x = String::new();
                            while x.is_empty() {
                                x = if let Answer::RESPONSE(n) = Question::new("Server Name:").ask().unwrap() { n } else { unreachable!() };
                            }
                            x
                        },
                        address: {
                            let mut x = String::new();
                            while x.is_empty() {
                                x = if let Answer::RESPONSE(n) = Question::new("Server Address:").ask().unwrap() { n } else { unreachable!() };
                            }
                            x
                        },
                        password: {
                            let mut x = String::new();
                            while x.is_empty() {
                                x = if let Answer::RESPONSE(n) = Question::new("Server Password:").ask().unwrap() { n } else { unreachable!() };
                            }
                            x
                        },
                        port: {
                            let mut x = String::new();
                            while x.is_empty() {
                                x = if let Answer::RESPONSE(n) = Question::new("Server Port (2302):").default(Answer::RESPONSE("2302".to_owned())).ask().unwrap() { n } else { unreachable!() };
                            }
                            x
                        },
                        battleEye: Question::new("BattleEye").default(Answer::YES).show_defaults().confirm() == question::Answer::YES,
                    });
                }
                servers
            },
            version: crate::repo::default_version(),
        };
        let j = serde_json::to_string_pretty(&repo).unwrap();
        let mut fout = File::create(repopath).unwrap();
        fout.write_all(j.as_bytes()).unwrap();
        Ok(())
    }
}
