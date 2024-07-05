use {super::Interactor, crate::plate::*};

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
			false => println!("FAILED {}", plate.goal().iter().collect::<String>()),
			true => println!("CORRECT {}", plate.count()),
		}
	}
}
