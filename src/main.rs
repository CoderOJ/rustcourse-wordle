use {
	anyhow::{Error, Result},
	wordle::{
		config::{self, WordSrc},
		interactor::*,
		plate::*,
		statistic::Statistic,
		util::loop_on_err_with,
		word_gen::*,
	},
};

fn main() -> Result<()> {
	let is_tty = atty::is(atty::Stream::Stdout);
	let config = config::config()?;
	let inter: &dyn Interactor = if is_tty { &Tty::new() } else { &Cmd::new() };

	let word_generator: &mut dyn Iterator<Item = Word> = match config.word_src {
		WordSrc::Select(word) => &mut std::iter::repeat(word).take(1),
		WordSrc::Ask => &mut RepeatReader::new(reader_from_set(&config.set_final, inter)),
		WordSrc::Random(seed, date) => {
			&mut RepeatReader::new(rand_words(&config.list_final, seed, date))
		}
	};
	let mut statistic = match &config.state_src {
		None => Statistic::new(),
		Some(src) => Statistic::load_from_file(&std::path::Path::new(src))?,
	};
	let mut read_acceptable = reader_from_set(&config.set_acceptable, inter);

	while let Some(word) = word_generator.next() {
		let mut plate = Plate::new(&word, config.difficult);
		inter.new_round();
		while !plate.is_win() && plate.count() < 6 {
			loop_on_err_with(
				|| {
					plate.guess(&read_acceptable()?)?;
					return Ok(());
				},
				|e: Error| {
					inter.print_err(e);
				},
			);
			inter.print_guess(&plate);
		}
		statistic.add_plate(&plate);
		inter.print_result(&plate);
		if config.stats {
			inter.print_statistic(&statistic);
		}
		if let Some(path) = &config.state_src {
			statistic.store_to_file(&std::path::Path::new(path))?;
		}
	}

	return Ok(());
}
