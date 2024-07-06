use wordle::{
	builtin_words,
	error::Error,
	interactor::*,
	plate::{word_eq, word_from_str, Plate, Word},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let is_tty = atty::is(atty::Stream::Stdout);

	let inter: Box<dyn Interactor> = if is_tty {
		Box::new(Tty::new())
	} else {
		Box::new(Cmd::new())
	};

	// `inter` must be passed because
	// 1. functions can not capture it
	// 2. specific lifetime for closures is still experimental
	fn get_reader<'a>(
		list: &'a Vec<Word>,
		inter: &'a Box<dyn Interactor>,
	) -> impl 'a + FnMut() -> Result<Word, Error> {
		|| {
			let word = inter.read_word()?;
			return if list.iter().any(|s| word_eq(&word, s)) {
				Ok(word)
			} else {
				Err(Error::Unkown)
			};
		}
	}

	// rust does not automaticly extend lifetime like cpp does to a rebinded xvalue
	// so it is required to explicitly bind it to `list_final`, which has a explicit lifetime
	let parse_list = |list: &[&str]| list.iter().map(|&s| word_from_str(s).unwrap()).collect();
	let list_final: Vec<Word> = parse_list(builtin_words::FINAL);
	let list_acceptalbe: Vec<Word> = parse_list(builtin_words::ACCEPTABLE);
	let mut read_final = get_reader(&list_final, &inter);
	let mut read_acceptable = get_reader(&list_acceptalbe, &inter);

	let mut plate = Plate::new(&read_final()?);
	while !plate.is_win() && plate.count() < 6 {
		let word = loop {
			match read_acceptable() {
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
