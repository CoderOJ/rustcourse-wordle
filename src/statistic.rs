use {
	crate::plate::*,
	std::collections::{BTreeMap, BTreeSet},
};

#[derive(PartialEq, Eq)]
pub struct WordCnt {
	pub str: String,
	pub cnt: u64,
}
impl PartialOrd for WordCnt {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some((other.cnt, &self.str).cmp(&(self.cnt, &other.str)))
	}
}
impl Ord for WordCnt {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		self.partial_cmp(other).unwrap()
	}
}

pub struct Statistic {
	success_cnt:        u64,
	fail_cnt:           u64,
	success_attemp_cnt: u64,

	word_cnt:  BTreeMap<String, u64>,
	top_words: BTreeSet<WordCnt>,
}

impl Statistic {
	pub fn new() -> Statistic {
		Statistic {
			success_cnt:        0,
			fail_cnt:           0,
			success_attemp_cnt: 0,
			word_cnt:           Default::default(),
			top_words:          Default::default(),
		}
	}

	pub fn success_cnt(&self) -> u64 {
		self.success_cnt
	}
	pub fn fail_cnt(&self) -> u64 {
		self.fail_cnt
	}
	pub fn success_attempt_average(&self) -> f64 {
		match self.success_cnt {
			0 => 0f64,
			_ => (self.success_attemp_cnt as f64) / (self.success_cnt as f64),
		}
	}

	fn add_word(&mut self, word: &Word) {
		let word_str = word_to_str(word);
		match self.word_cnt.get_mut(&word_str) {
			None => {
				self.word_cnt.insert(word_str.clone(), 1);
				self.top_words.insert(WordCnt {
					str: word_str,
					cnt: 1,
				});
			}
			Some(c) => {
				// take word_str out of top_words to prevent cloning it
				let wc = self
					.top_words
					.take(&WordCnt {
						str: word_str,
						cnt: *c,
					})
					.unwrap();
				self.top_words.insert(WordCnt {
					cnt: wc.cnt + 1,
					str: wc.str,
				});
				*c += 1;
			}
		}
	}

	pub fn add_plate(&mut self, plate: &Plate) {
		match plate.is_win() {
			true => {
				self.success_cnt += 1;
				self.success_attemp_cnt += plate.history().len() as u64;
			}
			false => {
				self.fail_cnt += 1;
			}
		};
		for (word, _) in plate.history() {
			self.add_word(word);
		}
	}

	pub fn top5_words(&self) -> impl Iterator<Item = &WordCnt> {
		self.top_words.iter().take(5)
	}
}
