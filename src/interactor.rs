mod cmd;
use crate::{error::Error, plate::*};

pub use cmd::Cmd;

pub trait Interactor {
	fn read_word(&self) -> Result<Word, Error> {
		let mut buf = String::new();
		std::io::stdin()
			.read_line(&mut buf)
			.map_err(|_| Error::Unkown)?;
		return word_from_str(buf.trim());
	}
	fn print_guess(&self, _: &Plate);
	fn print_result(&self, _: &Plate);
}
