use {
	crate::{interactor::Interactor, plate::*},
	anyhow::{anyhow, Result},
	rand::{self, SeedableRng},
	std::collections::HashSet,
};

/// RepeatReader implies Iterator trait for word reading
/// It takes a FnMut as word getter (e.g. reader/rander)
/// Wrap it with reading Y/N expect the first round
pub struct RepeatReader<F: FnMut() -> Result<Word>> {
	first_time: bool,
	reader:     F,
}

impl<F: FnMut() -> Result<Word>> RepeatReader<F> {
	pub fn new(reader: F) -> Self {
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

pub fn reader_from_set<'a>(
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

/// iter-like rand word getter
/// returns a FnMut, which returns next random word on each call
pub fn rand_words(list: &Vec<Word>, seed: u64, date: u32) -> impl FnMut() -> Result<Word> {
	use rand::seq::SliceRandom;
	let mut list = list.clone();
	let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
	list.shuffle(&mut rng);
	let mut iter = list.into_iter().skip((date - 1) as usize);
	return move || iter.next().ok_or(anyhow!("End of random list"));
}