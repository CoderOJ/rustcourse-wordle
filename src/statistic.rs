use {
	crate::plate::*,
	anyhow::Result,
	serde::{Deserialize, Serialize},
	serde_json::{from_str, to_string},
	std::{
		collections::{BTreeMap, BTreeSet},
		path::Path,
	},
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

#[derive(Clone, Serialize, Deserialize)]
struct StatisticGame {
	answer:  String,
	guesses: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct StatisticJSON {
	total_rounds: Option<u64>,
	games:        Option<Vec<StatisticGame>>,
}

#[derive(Default)]
pub struct Statistic {
	success_cnt:        u64,
	fail_cnt:           u64,
	success_attemp_cnt: u64,

	word_cnt:  BTreeMap<String, u64>,
	top_words: BTreeSet<WordCnt>,
	games:     Vec<StatisticGame>,
}

impl Statistic {
	pub fn new() -> Self {
		Self {
			success_cnt:        0,
			fail_cnt:           0,
			success_attemp_cnt: 0,
			word_cnt:           Default::default(),
			top_words:          Default::default(),
			games:              Default::default(),
		}
	}

	pub fn load_from_json(json_str: &str) -> Result<Self> {
		let state: StatisticJSON = from_str(json_str)?;
		return match state.games {
			None => Ok(Default::default()),
			Some(games) => {
				let mut result: Statistic = Default::default();
				for game in games {
					result._add_plate(game.answer, game.guesses);
				}
				return Ok(result);
			}
		};
	}
	pub fn load_from_file(path: &Path) -> Result<Self> {
		if !path.exists() {
			return Ok(Self::new());
		}
		return Self::load_from_json(&std::fs::read_to_string(path)?);
	}

	pub fn store_to_json(&self) -> String {
		to_string(&StatisticJSON {
			total_rounds: Some(self.success_cnt + self.fail_cnt),
			games:        Some(self.games.clone()),
		})
		.unwrap()
	}
	pub fn store_to_file(&self, path: &Path) -> Result<()> {
		std::fs::write(path, &self.store_to_json())?;
		return Ok(());
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

	fn add_word(&mut self, word: &String) {
		match self.word_cnt.get_mut(word) {
			None => {
				self.word_cnt.insert(word.clone(), 1);
				self.top_words.insert(WordCnt {
					str: word.clone(),
					cnt: 1,
				});
			}
			Some(c) => {
				// take word_str out of top_words to prevent cloning it
				let wc = self
					.top_words
					.take(&WordCnt {
						str: word.clone(),
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

	fn _add_plate(&mut self, goal: String, history: Vec<String>) {
		// is_win?
		match goal == *history.last().unwrap() {
			true => {
				self.success_cnt += 1;
				self.success_attemp_cnt += history.len() as u64;
			}
			false => {
				self.fail_cnt += 1;
			}
		};
		for word in &history {
			self.add_word(word);
		}
		self.games.push(StatisticGame {
			answer:  goal,
			guesses: history,
		});
	}
	
	/// update statistic by a WHOLE plate
	pub fn add_plate(&mut self, plate: &Plate) {
		self._add_plate(
			word_to_str(plate.goal()),
			plate
				.history()
				.iter()
				.map(|(w, _)| word_to_str(w))
				.collect(),
		)
	}

	pub fn top5_words(&self) -> impl Iterator<Item = &WordCnt> {
		self.top_words.iter().take(5)
	}
}
