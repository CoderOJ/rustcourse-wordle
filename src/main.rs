use {
	clap::Parser,
	wordle::{
		builtin_words,
		error::Error,
		interactor::*,
		plate::{word_eq, word_from_str, Plate, Word},
		util::loop_on_err_with,
	},
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
	/// select answer mode, conflict to -r
	#[arg(short, long)]
	word: Option<String>,

	/// random mode, conflict to -s
	#[arg(short, long, default_value_t = false)]
	random: bool,

	/// difficult mode
	#[arg(short = 'D', long, default_value_t = false)]
	difficult: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let is_tty = atty::is(atty::Stream::Stdout);
	let args = Args::parse();

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
	let list_acceptalbe: Vec<Word> = parse_list(builtin_words::ACCEPTABLE);
	let mut read_acceptable = get_reader(&list_acceptalbe, &inter);

	let mut plate = Plate::new(
		&{
			match (args.word, args.random) {
				(Some(_), true) => Err(Error::Unkown)?,
				(Some(word_str), false) => word_from_str(word_str.as_str())?,
				(None, true) => todo!(),
				(None, false) => {
					let list_final: Vec<Word> = parse_list(builtin_words::FINAL);
					let mut read_final = get_reader(&list_final, &inter);
					read_final()?
				}
			}
		},
		args.difficult,
	);

	while !plate.is_win() && plate.count() < 6 {
		loop_on_err_with(
			|| {
				plate.guess(&read_acceptable()?)?;
				return Ok(());
			},
			|_: Error| {
				println!("INVALID");
			},
		);
		inter.print_guess(&plate);
	}
	inter.print_result(&plate);

	return Ok(());
}
