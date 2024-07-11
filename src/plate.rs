use {
	crate::util::LetterMap,
	anyhow::{anyhow, Result},
	LetterState::*,
};

pub type Letter = char;
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum LetterState {
	Correct,
	Occured,
	Redundant,
	Unkown,
}

pub type Word = [Letter; 5];
pub type WordState = [LetterState; 5];

pub fn word_from_str(s: &str) -> Result<Word> {
	s.to_ascii_uppercase()
		.chars()
		.collect::<Vec<char>>()
		.try_into()
		.map_err(|_| anyhow!("invalid word: {:?}", s))
}

pub fn word_to_str(s: &Word) -> String {
	s.iter().collect()
}

pub fn word_eq(lhs: &Word, rhs: &Word) -> bool {
	lhs.iter().zip(rhs.iter()).all(|(a, b)| a == b)
}

pub struct Plate {
	goal:       Word,
	letter_cnt: LetterMap<u32>,
	keyboard:   LetterMap<LetterState>,
	is_win:     bool,
	history:    Vec<(Word, WordState)>,
	difficult:  bool,
}

impl Default for LetterState {
	fn default() -> Self {
		Self::Unkown
	}
}

impl LetterState {
	fn or(lhs: LetterState, rhs: LetterState) -> LetterState {
		match (lhs, rhs) {
			(Correct, _) => Correct,
			(_, Correct) => Correct,
			(Occured, _) => Occured,
			(_, Occured) => Occured,
			(Redundant, _) => Redundant,
			(_, Redundant) => Redundant,
			(Unkown, Unkown) => Unkown,
		}
	}
}

impl Plate {
	/// new Plate with candidate `word`
	pub fn new(word: &Word, difficult: bool) -> Plate {
		let mut letter_cnt: LetterMap<u32> = Default::default();
		for &c in word {
			letter_cnt[c] += 1;
		}

		return Plate {
			goal: word.clone(),
			letter_cnt,
			keyboard: Default::default(),
			is_win: false,
			history: vec![],
			difficult,
		};
	}

	pub fn goal(&self) -> &Word {
		&self.goal
	}

	pub fn is_win(&self) -> bool {
		self.is_win
	}

	/// number of rounds
	pub fn count(&self) -> u32 {
		self.history.len() as u32
	}

	/// all history words and wordstates
	pub fn history(&self) -> &Vec<(Word, WordState)> {
		&self.history
	}

	/// all history words and wordstates
	pub fn keyboard(&self) -> &LetterMap<LetterState> {
		&self.keyboard
	}

	fn is_compatible(&self, word: &Word) -> Result<()> {
		for (prev_word, prev_state) in &self.history {
			let mut word_cnt: LetterMap<u32> = Default::default();
			for &c in word {
				word_cnt[c] += 1;
			}

			// pass 1: check Correct
			for i in 0..5usize {
				if prev_state[i] == Correct {
					if word[i] != prev_word[i] {
						return Err(anyhow!(
							"{} is not compatible with previous {}",
							word_to_str(word),
							word_to_str(prev_word)
						));
					}
					word_cnt[word[i]] -= 1;
				}
			}

			// pass 2: check Occur
			for i in 0..5usize {
				if prev_state[i] == Occured {
					if word_cnt[prev_word[i]] == 0 {
						return Err(anyhow!(
							"{} is not compatible with previous {}",
							word_to_str(word),
							word_to_str(prev_word)
						));
					}
					word_cnt[prev_word[i]] -= 1;
				}
			}
		}

		return Ok(());
	}

	pub fn guess(&mut self, word: &Word) -> Result<()> {
		if self.difficult {
			self.is_compatible(word)?;
		}

		let mut word_state: WordState = Default::default();
		let mut letter_cnt = self.letter_cnt.clone();

		let mut set_state = |index, state| {
			word_state[index] = state;
			self.keyboard[word[index]] = LetterState::or(self.keyboard[word[index]], state);
		};

		// pass 1: mark Correct
		for i in 0..5usize {
			if word[i] == self.goal[i] {
				letter_cnt[word[i]] -= 1;
				set_state(i, Correct);
			}
		}

		// pass 2: mark Redundant
		for i in 0..5usize {
			if word[i] != self.goal[i] {
				if letter_cnt[word[i]] > 0 {
					letter_cnt[word[i]] -= 1;
					set_state(i, Occured);
				} else {
					set_state(i, Redundant);
				}
			}
		}

		if word_eq(word, &self.goal) {
			self.is_win = true;
		}

		self.history.push((word.clone(), word_state));
		return Ok(());
	}
}
