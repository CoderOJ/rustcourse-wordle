use {
	super::Interactor,
	crate::{error::Error, plate::*, statistic::*},
	console::{style, StyledObject},
};

pub struct Tty;

impl Tty {
	pub fn new() -> Self {
		Self {}
	}
}

fn format_char(cs: (&Letter, &LetterState)) -> StyledObject<char> {
	match cs {
		(&c, LetterState::Correct) => style(c).green(),
		(&c, LetterState::Occured) => style(c).yellow(),
		(&c, LetterState::Redundant) => style(c).red(),
		(&c, LetterState::Unkown) => style(c),
	}
}

fn println_iter<T: Iterator>(t: T)
where
	<T as Iterator>::Item: std::fmt::Display,
{
	for c in t {
		print!("{}", c);
	}
	println!();
}

impl Interactor for Tty {
	fn print_guess(&self, plate: &Plate) {
		println!("---");
		for &state in plate.history() {
			println_iter(state.0.iter().zip(state.1.iter()).map(format_char));
		}
		println!("---");
		println_iter(('A'..='Z').map(|c| format_char((&c, &plate.keyboard()[c]))));
		println!("---");
	}
	fn print_result(&self, plate: &Plate) {
		match plate.is_win() {
			false => println!("{} {}", style("FAILED").red(), word_to_str(plate.goal())),
			true => println!("{} {}", style("CORRECT").green(), plate.count()),
		}
	}
	fn print_statistic(&self, s: &Statistic) {
		println!("Statistic:");
		println!(
			"success: {} fail: {} average attempts: {:.2}",
			s.success_cnt(),
			s.fail_cnt(),
			s.success_attempt_average()
		);
		println!(
			"Top words: {}",
			s.top5_words()
				.map(|x| format!("{}*{}", x.str, x.cnt))
				.collect::<Vec<String>>()
				.join(" ")
		);
	}
	fn print_err(&self, e: Error) {
		println!("{} {}", style("error:").red(), e);
	}
}
