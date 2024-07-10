/// Various Erorr regarding wordle game
#[derive(Debug)]
pub enum Error {
  Unkown,
}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "error[todo]")
	}
}

impl std::error::Error for Error {
	fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
		None
	}
}

pub type ErrorAll = Box<dyn std::error::Error>;