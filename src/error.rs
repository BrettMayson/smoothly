pub trait PrintableError<T, E> {
    fn unwrap_or_print(self) -> T;
}
impl<T, E: std::fmt::Debug + std::fmt::Display> PrintableError<T, E> for Result<T, E> {
    fn unwrap_or_print(self) -> T {
        if let Err(error) = &self {
            println!("{}", error);
            std::process::exit(1);
        }
        self.unwrap()
    }
}

#[derive(Debug)]
pub struct IOPathError {
    pub source: std::io::Error,
    pub path: std::path::PathBuf,
    pub message: Option<String>,
}

#[derive(Debug)]
pub enum SmoothlyError {
    GENERIC(String),
    CONFIG(serde_json::error::Error),
    MESSAGE(String, Box<SmoothlyError>),
    IO(std::io::Error),
    IOPath(IOPathError),
}

impl std::fmt::Display for SmoothlyError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            SmoothlyError::GENERIC(ref s) => write!(f, "{}", s),
            SmoothlyError::CONFIG(ref e) => write!(f, "JSON error: {}", e),
            SmoothlyError::MESSAGE(ref s, ref _e) => write!(f, "{}", s),
            SmoothlyError::IO(ref e) => write!(f, "IO error: {}", e),
            SmoothlyError::IOPath(ref e) => write!(f, "IO error: `{:#?}`\n{}", e.path, e.source),
        }
    }
}

impl std::error::Error for SmoothlyError {
    fn cause(&self) -> Option<&std::error::Error> {
        match *self {
            SmoothlyError::GENERIC(ref _s,) => Some(self),
            SmoothlyError::CONFIG(ref e) => Some(e),
            SmoothlyError::MESSAGE(ref _s, ref e) => Some(e),
            SmoothlyError::IO(ref e) => Some(e),
            SmoothlyError::IOPath(ref e) => Some(&e.source),
        }
    }
}

impl From<std::io::Error> for SmoothlyError {
    fn from(err: std::io::Error) -> SmoothlyError {
        SmoothlyError::IO(err)
    }
}

impl From<serde_json::error::Error> for SmoothlyError {
    fn from(err: serde_json::error::Error) -> SmoothlyError {
        SmoothlyError::CONFIG(err)
    }
}

