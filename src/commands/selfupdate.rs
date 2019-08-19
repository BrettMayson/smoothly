use crate::{Command, SmoothlyError};

pub struct SelfUpdate {}

impl Command for SelfUpdate {
    fn register(&self) -> clap::App {
        clap::SubCommand::with_name("self-update")
            .about("Update Smoothly")
    }

    fn run(&self, _: &clap::ArgMatches, _: String) -> Result<(), SmoothlyError> {
        let status = self_update::backends::github::Update::configure()
            .repo_owner("synixebrett")
            .repo_name("smoothly")
            .bin_name(if cfg!(windows) {"smoothly.exe"} else {"smoothly"})
            .show_download_progress(true)
            .current_version(env!("CARGO_PKG_VERSION"))
            .build().unwrap()
            .update().unwrap();
        println!("Update status: `{}`!", status.version());
        Ok(())
    }
}
