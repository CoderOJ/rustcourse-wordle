use crate::{error::Error, plate::*, statistic::Statistic};

mod cmd;
mod tty;
pub use {cmd::Cmd, tty::Tty};

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
	fn print_statistic(&self, _: &Statistic);
	fn print_err(&self, _: Error);
}
