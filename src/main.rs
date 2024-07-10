use {
	anyhow::{anyhow, Error, Result},
	rand::{self, SeedableRng},
	std::collections::HashSet,
	wordle::{
		config::{self, WordSrc},
		interactor::*,
		plate::*,
		statistic::Statistic,
		util::loop_on_err_with,
	},
};

struct RepeatReader<F: FnMut() -> Result<Word>> {
	first_time: bool,
	reader:     F,
}

impl<F: FnMut() -> Result<Word>> RepeatReader<F> {
	fn new(reader: F) -> Self {
		Self {
			first_time: true,
			reader,
		}
	}
}

impl<F: FnMut() -> Result<Word>> Iterator for RepeatReader<F> {
	type Item = Word;
	fn next(&mut self) -> Option<Self::Item> {
		let is_next = match self.first_time {
			true => {
				self.first_time = false;
				true
			}
			false => {
				let mut s = String::new();
				std::io::stdin().read_line(&mut s).ok()?;
				s == "Y\n"
			}
		};
		return match is_next {
			true => Some((self.reader)().ok()?),
			false => None,
		};
	}
}

fn reader_from_set<'a>(
	set: &'a HashSet<Word>,
	inter: &'a dyn Interactor,
) -> impl 'a + FnMut() -> Result<Word> {
	|| {
		let word = inter.read_word()?;
		return if set.contains(&word) {
			Ok(word)
		} else {
			Err(anyhow!("word {} out of range", word_to_str(&word)))
		};
	}
}

fn rand_words(list: &Vec<Word>, seed: u64, date: u32) -> impl FnMut() -> Result<Word> {
	use rand::seq::SliceRandom;
	let mut list = list.clone();
	let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
	list.shuffle(&mut rng);
	let mut iter = list.into_iter().skip((date - 1) as usize);
	return move || Ok(iter.next().unwrap());
}

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
	let mut statistic = Statistic::new();
	let mut read_acceptable = reader_from_set(&config.set_acceptable, inter);

	while let Some(word) = word_generator.next() {
		let mut plate = Plate::new(&word, config.difficult);
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
	}

	return Ok(());
}
