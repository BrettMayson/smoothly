use crate::{SmoothlyError, Command, Repo};

pub struct Add {}

impl Command for Add {
    fn register(&self) -> clap::App {
        clap::SubCommand::with_name("add")
            .about("Add mods to the repository")
            .arg(clap::Arg::with_name("mod")
                .help("Mod directory")
                .required(true)
            ).arg(clap::Arg::with_name("optional")
                .long("--optional")
                .help("Add to Optional mods")
            )
    }

    fn run(&self, args: &clap::ArgMatches, repo: String) -> Result<(), SmoothlyError> {
        let repo = Repo::new(repo)?;
        println!("Name: {}", repo.repoName);
        println!("Add");
        Ok(())
    }
}
