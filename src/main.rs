use clap;
use hashbrown::HashMap;

#[cfg(windows)]
use ansi_term;

use smoothly::error::PrintableError;
use smoothly::Command;

fn main() {
    if cfg!(windows) {
        ansi_support();
    }

    let mut version = env!("CARGO_PKG_VERSION").to_string();
    if cfg!(debug_assertions) {
        version.push_str("-debug");
    }

    let mut app = clap::App::new("smoothly")
        .version(version.as_ref())
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(clap::Arg::with_name("repo")
            .help("Repository file (repo.json)")
            .global(true)
            .long("repo")
        );

    let mut commands: Vec<Box<dyn Command>> = Vec::new();
    let mut hash_commands: HashMap<String, &Box<dyn Command>> = HashMap::new();

    commands.push(Box::new(smoothly::commands::New {}));
    //commands.push(Box::new(smoothly::commands::Add {}));
    commands.push(Box::new(smoothly::commands::Interact {}));
    commands.push(Box::new(smoothly::commands::Push {}));
    commands.push(Box::new(smoothly::commands::SelfUpdate {}));

    for command in commands.iter() {
        let sub = command.register();
        let name = sub.get_name().to_owned();
        app = app.subcommand(sub);
        hash_commands.insert(name, command);
    }

    let matches = app.get_matches();

    let repo = if !matches.is_present("repo") {
        if std::path::PathBuf::from("repo.json").exists() {
            "repo.json"
        } else if std::path::PathBuf::from("repo.toml").exists() {
            "repo.toml"
        } else {
            println!("No repo specified");
            std::process::exit(1);
        }
    } else {
        matches.value_of("repo").unwrap()
    };
    println!("Using `{}`", repo);

    match matches.subcommand_name() {
        Some(v) => {
            match hash_commands.get(v) {
                Some(c) => {
                    let sub_matches = matches.subcommand_matches(v).unwrap();
                    c.run(sub_matches, repo.to_string()).unwrap_or_print();
                },
                None => println!("Unknown Command"),
            }
        },
        None => println!("No command"),
    }
}

#[cfg(windows)]
fn ansi_support() {
    // Attempt to enable ANSI support in terminal
    // Disable colored output if failed
    if ansi_term::enable_ansi_support().is_err() {
        colored::control::set_override(false);
    }
}

#[cfg(not(windows))]
fn ansi_support() {
    unreachable!();
}
