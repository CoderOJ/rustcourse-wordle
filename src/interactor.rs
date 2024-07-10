use {
	crate::{plate::*, statistic::Statistic},
	anyhow::{Error, Result},
};

mod cmd;
mod tty;
pub use {cmd::Cmd, tty::Tty};

pub trait Interactor {
	fn read_word(&self) -> Result<Word> {
		let mut buf = String::new();
		std::io::stdin().read_line(&mut buf)?;
		return word_from_str(buf.trim());
	}
	fn print_guess(&self, _: &Plate);
	fn print_result(&self, _: &Plate);
	fn print_statistic(&self, _: &Statistic);
	fn print_err(&self, _: Error);
}
