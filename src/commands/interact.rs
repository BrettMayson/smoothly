use std::fs::File;
use std::io::Write;

use crossterm::{InputEvent, KeyEvent, RawScreen, input, cursor};
use colored::*;

use crate::{SmoothlyError, Command, Repo, Mod, Transaction, State};

pub struct Interact {}

impl Interact {
    fn tick(mods: &[(String, Transaction, State)], index: usize, page: usize) -> Result<(), SmoothlyError> {
        let mut cursor = cursor();
        let c = cursor2index(index, mods.len());
        let start = page * 8;
        let mut end = start+8;
        if end > mods.len() {
            end = mods.len();
        }
        cursor.move_up(12);
        println!("\rPage: {} ({} - {})                             ", page, start + 1, end);
        for (i, arma_mod) in mods[start..end].iter().enumerate() {
            let (name, trans, state) = arma_mod;
            if let Cursor::Item(_) = c {
                if i == index - (10 * (index / 10)) {
                    println!("\r{} {:20} {}                                                                     ", "*".cyan(), color!(name, trans), state!(state));
                    continue;
                }
            }
            println!("\r  {:20} {}                                                                     ", color!(name, trans), state!(state));
        }
        for _ in 0..(8 - (end - start)) {
            println!("\r                                                                     ");
        }
        println!();
        if c == Cursor::Apply {
            println!("\r{} Apply (End)                                                                     ", "*".cyan());
        } else {
            println!("\r  Apply (End)                                                                     ");
        }
        if c == Cursor::Cancel {
            println!("\r{} Cancel (q)                                                                     ", "*".cyan());
        } else {
            println!("\r  Cancel (q)                                                                     ");
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

        let screen = RawScreen::into_raw_mode()?;

        let input = input();
        let mut stdin = input.read_async();

        let mut index = 0;
        let mut page = 0;

        for _ in 0..13 {
            println!();
        }

        loop {
            Interact::tick(&mods, index, page)?;
            if let Some(key_event) = stdin.next() {
                let cursor = cursor2index(index, mods.len());
                match key_event {
                    InputEvent::Keyboard(e) => { match e {
                        KeyEvent::Up => {
                            index = if index % 10 != 0 { index - 1 } else { index };
                        },
                        KeyEvent::Down => {
                            if cursor != Cursor::Cancel {
                                index = index + 1;
                            }
                            // index = if index - (10 * (index / 10)) != 9 { index + 1 } else { index };
                        },
                        KeyEvent::Right => {
                            page = if (page + 1) * 8 < mods.len() - 1 { page + 1 } else { page };
                            index = page * 10;
                        }
                        KeyEvent::Left => {
                            page = if page == 0 { 0 } else { page - 1};
                            index = page * 10;
                        }
                        KeyEvent::Char(c) => {
                            match c {
                                ' ' => {
                                    if let Cursor::Item(c) = cursor {
                                        mods[c].2 = match mods[c].2 {
                                            State::Disabled => State::Enabled,
                                            State::Enabled => State::OptionalDisabled,
                                            State::OptionalDisabled => State::OptionalEnabled,
                                            State::OptionalEnabled => State::Disabled,
                                        };
                                    }
                                },
                                'q' => {
                                    exit!(0, screen);
                                },
                                '\n' => {
                                    match cursor {
                                        Cursor::Item(c) => {
                                            mods[c].1 = if mods[c].1 == Transaction::Remove || mods[c].1 == Transaction::Existing { Transaction::Existing } else { Transaction::Add };
                                            mods[c].2 = State::Enabled;
                                        },
                                        Cursor::Apply => {
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
                                            exit!(0, screen);
                                        },
                                        Cursor::Cancel => {exit!(0, screen);},
                                    }
                                },
                                _ => {}
                            }
                        }
                        KeyEvent::Delete => {
                            if let Cursor::Item(c) = cursor {
                                let current = mods[c].1.clone();
                                mods[c].1 = match current {
                                    Transaction::Existing => Transaction::Remove,
                                    Transaction::Add => Transaction::Ignore,
                                    _ => current,
                                };
                                mods[c].2 = State::Disabled;
                            }
                        }
                        KeyEvent::Insert => {
                            if let Cursor::Item(c) = cursor {
                                mods[c].1 = if mods[c].1 == Transaction::Remove || mods[c].1 == Transaction::Existing { Transaction::Existing } else { Transaction::Add };
                                mods[c].2 = State::Enabled;
                            }
                        }
                        KeyEvent::End => {
                            index = (10 * (index / 10)) + 8;
                        },
                        _ => {}
                    }},
                    _ => {}
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Cursor {
    Item(usize),
    Apply,
    Cancel,
}

fn cursor2index(c: usize, max: usize) -> Cursor {
    match c - (10 * (c / 10)) {
        8 => Cursor::Apply,
        9 => Cursor::Cancel,
        _ => {
            let i = c - (2 * (c / 10));
            if i == max {
                Cursor::Apply
            } else if i == max + 1 {
                Cursor::Cancel
            } else {
                Cursor::Item(i)
            }
        },
    }
}
