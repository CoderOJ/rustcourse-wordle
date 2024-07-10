use {
	crate::{builtin_words, error::Error, plate::*},
	clap::Parser,
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
	/// select answer mode, conflict to -r
	#[arg(short, long, conflicts_with = "random")]
	word: Option<String>,

	/// random mode, conflict to -s
	#[arg(short, long, default_value_t = false, conflicts_with = "word")]
	random: bool,
	/// random seed, requires -r
	#[arg(
		short,
		long,
		default_value_t = 0,
		value_name = "SEED",
		requires = "random",
		// redundent for: clap seems to treat `-r`` as *provided* when `-w` presents
		conflicts_with = "word",
	)]
	seed:   u64,
	/// random date, requires -r
	#[arg(
		short,
		long = "day",
		default_value_t = 1,
		value_name = "DAY",
		requires = "random",
		// redundent for: clap seems to treat `-r`` as *provided* when `-w` presents
		conflicts_with = "word",
	)]
	date:   u32,

	/// difficult mode
	#[arg(short = 'D', long, default_value_t = false)]
	difficult: bool,

	/// print statistic
	#[arg(short = 't', long, default_value_t = false)]
	stats: bool,
}

pub enum WordSrc {
	/// ask on each round
	Ask,
	/// selected in argument
	Select(Word),
	/// random(seed, start_date)
	Random(u64, u32),
}

pub struct Config {
	pub difficult:       bool,
	pub stats:           bool,
	pub word_src:        WordSrc,
	pub list_acceptable: Vec<Word>,
	pub list_final:      Vec<Word>,
}

pub fn config() -> Result<Config, Error> {
	let args = Args::parse();

	let parse_list = |list: &[&str]| list.iter().map(|&s| word_from_str(s).unwrap()).collect();
	let list_acceptable: Vec<Word> = parse_list(builtin_words::ACCEPTABLE);
	let list_final: Vec<Word> = parse_list(builtin_words::FINAL);

	let word_src: WordSrc = match (args.word, args.random, args.seed, args.date) {
		(None, false, _, _) => WordSrc::Ask,
		(None, true, seed, date) => WordSrc::Random(seed, date),
		(Some(word_str), false, _, _) => WordSrc::Select(word_from_str(word_str.as_str())?),
		_ => unreachable!(),
	};

	return Ok(Config {
		difficult: args.difficult,
		stats: args.stats,
		word_src,
		list_acceptable,
		list_final,
	});
}
