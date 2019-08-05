use std::fs::File;
use std::io::Write;

use crossterm::{InputEvent, KeyEvent, RawScreen, input, cursor};
use colored::*;

use crate::{SmoothlyError, Command, Repo, Mod, Transaction, State};

pub struct Interact {}

impl Interact {
    fn tick(mods: &[(String, Transaction, State)], index: usize) -> Result<(), SmoothlyError> {
        let mut cursor = cursor();
        cursor.move_up(mods.len() as u16 + 3);
        for (i, arma_mod) in mods.iter().enumerate() {
            let (name, trans, state) = arma_mod;
            if index == i {
                println!("\r{} {:20} {}                       ", "*".cyan(), color!(name, trans), state!(state));
            } else {
                println!("\r  {:20} {}                       ", color!(name, trans), state!(state));
            }
        }
        println!();
        let end = mods.len();
        if index == end {
            println!("\r{} Apply", "*".cyan());
        } else {
            println!("\r  Apply");
        }
        if index == end + 1 {
            println!("\r{} Cancel", "*".cyan());
        } else {
            println!("\r  Cancel");
        }
        cursor.hide().unwrap();
        Ok(())
    }
}

impl Command for Interact {
    fn register(&self) -> (&str, clap::App) {
        ("interact",
            clap::SubCommand::with_name("interact")
                .about("Interactively manage the mods in the repository")
        )
    }

    fn run(&self, args: &clap::ArgMatches, repo_path: String) -> Result<(), SmoothlyError> {
        let mut repo = Repo::new(repo_path.clone())?;
        println!("Name: {}", repo.repoName);
        println!("{} - {} - {} - {}", "Existing".purple(), "New".green(), "Remove".red(), "Ignored".white());

        let mut mods = Vec::new();

        for entry in std::fs::read_dir(&repo.basePath)? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_dir() { continue; }
            let name = path.file_name().unwrap().to_str().unwrap().to_owned();
            mods.push((
                name.clone(),
                if repo.has_mod(&name) { Transaction::Existing } else { Transaction::Ignore },
                repo.mod_state(&name),
            ));
        }

        let screen = RawScreen::into_raw_mode();

        let input = input();
        let mut stdin = input.read_async();

        let mut index = 0;

        for _ in 0..mods.len()+4 {
            println!();
        }

        loop {
            Interact::tick(&mods, index)?;
            if let Some(key_event) = stdin.next() {
                match key_event {
                    InputEvent::Keyboard(e) => { match e{
                        KeyEvent::Up => {
                            index = if index != 0 { index - 1 } else { mods.len() + 1 };
                        },
                        KeyEvent::Down => {
                            index = if index != mods.len() + 1 { index + 1 } else { 0 };
                        },
                        KeyEvent::Char(c) => {
                            match c {
                                ' ' => {
                                    if index >= mods.len() { continue; }
                                    mods[index].2 = match mods[index].2 {
                                        State::Disabled => State::Enabled,
                                        State::Enabled => State::OptionalDisabled,
                                        State::OptionalDisabled => State::OptionalEnabled,
                                        State::OptionalEnabled => State::Disabled,
                                    };
                                },
                                'q' => {
                                    drop(screen);
                                    std::process::exit(0);
                                },
                                '\n' => {
                                    if index < mods.len() { 
                                        mods[index].1 = if mods[index].1 == Transaction::Remove || mods[index].1 == Transaction::Existing { Transaction::Existing } else { Transaction::Add };
                                        mods[index].2 = State::Enabled;
                                    } else if index == mods.len() {
                                        repo.requiredMods = Vec::new();
                                        repo.optionalMods = Vec::new();
                                        for arma_mod in mods {
                                            if arma_mod.1 == Transaction::Add || arma_mod.1 == Transaction::Existing {
                                                let new_mod = Mod {
                                                    modName: arma_mod.0,
                                                    checkSum: "".to_owned(),
                                                    Enabled: arma_mod.2 == State::Enabled || arma_mod.2 == State::OptionalEnabled,
                                                };
                                                match arma_mod.2 {
                                                    State::Enabled | State::Disabled => {
                                                        repo.requiredMods.push(new_mod);
                                                    },
                                                    State::OptionalDisabled | State::OptionalEnabled => {
                                                        repo.optionalMods.push(new_mod);
                                                    }
                                                }
                                            }
                                        }
                                        let j = serde_json::to_string_pretty(&repo).unwrap();
                                        let mut fout = File::create(repo_path).unwrap();
                                        fout.write_all(j.as_bytes()).unwrap();
                                        drop(screen);
                                        std::process::exit(0);
                                    } else {
                                        drop(screen);
                                        std::process::exit(0);
                                    }
                                },
                                _ => {}
                            }
                        }
                        KeyEvent::Delete => {
                            if index >= mods.len() { continue; }
                            let current = mods[index].1.clone();
                            mods[index].1 = match current {
                                Transaction::Existing => Transaction::Remove,
                                Transaction::Add => Transaction::Ignore,
                                _ => current,
                            };
                            mods[index].2 = State::Disabled;
                        }
                        KeyEvent::Insert => {
                            if index >= mods.len() { continue; }
                            mods[index].1 = if mods[index].1 == Transaction::Remove || mods[index].1 == Transaction::Existing { Transaction::Existing } else { Transaction::Add };
                            mods[index].2 = State::Enabled;
                        }
                        KeyEvent::End => {
                            index = mods.len();
                        },
                        _ => {}
                    }},
                    _ => {}
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
    }
}
