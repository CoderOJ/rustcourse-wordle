use {
	super::Interactor,
	crate::{error::Error, plate::*, statistic::*},
};

pub struct Cmd;

impl Cmd {
	pub fn new() -> Self {
		Self {}
	}
}

fn state_to_char(state: &LetterState) -> char {
	match state {
		LetterState::Correct => 'G',
		LetterState::Occured => 'Y',
		LetterState::Redundant => 'R',
		LetterState::Unkown => 'X',
	}
}

impl Interactor for Cmd {
	fn print_guess(&self, plate: &Plate) {
		let state_fmt: String = plate
			.history()
			.last()
			.unwrap()
			.1
			.iter()
			.map(state_to_char)
			.collect();
		let keyboard_fmt: String = plate
			.keyboard()
			.as_arr()
			.iter()
			.map(state_to_char)
			.collect();
		println!("{} {}", state_fmt, keyboard_fmt);
	}
	fn print_result(&self, plate: &Plate) {
		match plate.is_win() {
			false => println!("FAILED {}", word_to_str(plate.goal())),
			true => println!("CORRECT {}", plate.count()),
		}
	}
	fn print_statistic(&self, s: &Statistic) {
		println!(
			"{} {} {:.2}",
			s.success_cnt(),
			s.fail_cnt(),
			s.success_attempt_average()
		);
		println!(
			"{}",
			s.top5_words()
				.map(|x| format!("{} {}", x.str, x.cnt))
				.collect::<Vec<String>>()
				.join(" ")
		);
	}
	fn print_err(&self, _: Error) {
		println!("INVALID");
	}
}
