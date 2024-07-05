use wordle::{
	builtin_words,
	error::Error,
	interactor::*,
	plate::{word_eq, word_from_str, Plate, Word},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let is_tty = atty::is(atty::Stream::Stdout);

	let inter: Box<dyn Interactor> = if is_tty {
		todo!();
	} else {
		Box::new(Cmd::new())
	};

	let read_candidate = |list: &Vec<Word>| -> Result<Word, Error> {
		let word = inter.read_word()?;
		return if list.iter().any(|s| word_eq(&word, s)) {
			Ok(word)
		} else {
			Err(Error::Unkown)
		};
	};
	let list_final: Vec<Word> = builtin_words::FINAL
		.iter()
		.map(|&s| word_from_str(s).unwrap())
		.collect();
	let list_acceptalbe: Vec<Word> = builtin_words::ACCEPTABLE
		.iter()
		.map(|&s| word_from_str(s).unwrap())
		.collect();

	let mut plate = Plate::new(&read_candidate(&list_final)?);
	while !plate.is_win() && plate.count() < 6 {
		let word = loop {
			match read_candidate(&list_acceptalbe) {
				Ok(word) => break word,
				Err(_) => println!("INVALID"),
			}
		};
		plate.guess(&word);
		inter.print_guess(&plate);
	}
	inter.print_result(&plate);

	return Ok(());
}
