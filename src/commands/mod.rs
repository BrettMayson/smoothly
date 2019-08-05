use crate::SmoothlyError;

pub trait Command {
    // (name, description)
    fn register(&self) -> (&str, clap::App);
    
    fn run(&self, _: &clap::ArgMatches, _: String) -> Result<(), SmoothlyError> {
        unimplemented!();
    }
}

mod new;
pub use new::New;

mod add;
pub use add::Add;

mod interact;
pub use interact::Interact;

mod push;
pub use push::Push;
