use {
	crate::{builtin_words, plate::*},
	anyhow::{anyhow, Result},
	clap::Parser,
	std::{collections::HashSet, io::BufRead, str::from_utf8},
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

	/// final word set
	#[arg(short = 'f', long = "final-set", value_name = "FINAL_SET_FILE")]
	final_set_src: Option<String>,

	/// acceptable word set
	#[arg(
		short = 'a',
		long = "acceptable-set",
		value_name = "ACCEPTABLE_SET_FILE"
	)]
	acceptable_set_src: Option<String>,

	/// state json file
	#[arg(short = 'S', long = "state", value_name = "STATE_FILE")]
	state_src: Option<String>,
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
	pub difficult:      bool,
	pub stats:          bool,
	pub word_src:       WordSrc,
	pub set_acceptable: HashSet<Word>,
	pub set_final:      HashSet<Word>,
	pub list_final:     Vec<Word>,
	pub state_src:      Option<String>,
}

pub fn config() -> Result<Config> {
	let args = Args::parse();

	let parse_builtin_list =
		|list: &[&str]| list.iter().map(|&s| word_from_str(s).unwrap()).collect();
	let read_list_src = |list_src| -> Result<Vec<Word>> {
		std::io::BufReader::new(std::fs::File::open(list_src)?)
			.split(b'\n')
			.map(|r| -> Result<Word> { Ok(word_from_str(from_utf8(&r?)?)?) })
			.collect()
	};
	let list_acceptable: Vec<Word> = match args.acceptable_set_src {
		None => parse_builtin_list(builtin_words::ACCEPTABLE),
		Some(src) => read_list_src(src)?,
	};
	let list_final: Vec<Word> = match args.final_set_src {
		None => parse_builtin_list(builtin_words::FINAL),
		Some(src) => read_list_src(src)?,
	};
	let set_acceptable: HashSet<Word> = list_acceptable.into_iter().collect();
	if !list_final.iter().all(|s| set_acceptable.contains(s)) {
		return Err(anyhow!("list_fianl is not subset of list_acceptable"));
	}

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
		set_acceptable,
		set_final: list_final.iter().cloned().collect(),
		list_final,
		state_src: args.state_src,
	});
}
