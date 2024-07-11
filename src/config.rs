use {
	crate::{builtin_words, plate::*},
	anyhow::{anyhow, Result},
	clap::Parser,
	serde::Deserialize,
	serde_json::from_str,
	std::{collections::HashSet, io::BufRead, str::from_utf8},
};

#[derive(Default, Deserialize, Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
	/// select answer mode, conflict to -r
	#[arg(short, long)]
	word: Option<String>,

	/// random mode, conflict to -s
	#[arg(short, long, default_value_t = false)]
	random: bool,

	/// random seed, requires -r
	#[arg(short, long)]
	seed: Option<u64>,
	/// random date, requires -r
	#[arg(short, long, value_name = "DAY")]
	day:  Option<u32>,

	/// difficult mode
	#[arg(short = 'D', long, default_value_t = false)]
	difficult: bool,

	/// print statistic
	#[arg(short = 't', long, default_value_t = false)]
	stats: bool,

	/// final word set
	#[arg(short = 'f', long = "final-set", value_name = "FINAL_SET_FILE")]
	#[serde(rename = "final_set")]
	final_set_src: Option<String>,

	/// acceptable word set
	#[arg(
		short = 'a',
		long = "acceptable-set",
		value_name = "ACCEPTABLE_SET_FILE"
	)]
	#[serde(rename = "acceptable_set")]
	acceptable_set_src: Option<String>,

	/// state json file
	#[arg(short = 'S', long = "state", value_name = "STATE_FILE")]
	#[serde(rename = "state")]
	state_src: Option<String>,

	/// default config file
	#[arg(short, long = "config", value_name = "CONFIG_FILE")]
	#[serde(rename = "config")]
	config_src: Option<String>,
}

#[derive(Debug)]
pub enum WordSrc {
	/// ask on each round
	Ask,
	/// selected in argument
	Select(Word),
	/// random(seed, start_date)
	Random(u64, u32),
}

#[derive(Debug)]
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
	let args0 = Args::parse();
	let args1: Args = match args0.config_src {
		None => Default::default(),
		Some(path) => from_str(&std::fs::read_to_string(&path)?)?,
	};
	let args = Args {
		word:               args0.word.or(args1.word),
		random:             args0.random || args1.random,
		seed:               args0.seed.or(args1.seed),
		day:                args0.day.or(args1.day),
		difficult:          args0.difficult || args1.difficult,
		stats:              args0.stats || args1.stats,
		final_set_src:      args0.final_set_src.or(args1.final_set_src),
		acceptable_set_src: args0.acceptable_set_src.or(args1.acceptable_set_src),
		state_src:          args0.state_src.or(args1.state_src),
		config_src:         None,
	};

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

	let word_src: WordSrc = match (args.word, args.random, args.seed, args.day) {
		(None, false, _, _) => WordSrc::Ask,
		(None, true, seed, date) => WordSrc::Random(seed.unwrap_or(0), date.unwrap_or(1)),
		(Some(word_str), false, None, None) => WordSrc::Select(word_from_str(word_str.as_str())?),
		_ => Err(anyhow!("arguments conflict!"))?,
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
