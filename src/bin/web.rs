use {
	anyhow::{anyhow, Result},
	std::{cell::Cell, collections::HashSet, rc::Rc},
	web_sys::{wasm_bindgen::JsValue, window, FormData, HtmlFormElement},
	wordle::{
		builtin_words, config::*, plate::*, statistic::Statistic, util::LetterMap,
		word_gen::rand_words,
	},
	yew::prelude::*,
};

fn alert(msg: &str) {
	(|| -> Option<()> {
		window()?.alert_with_message(msg).ok()?;
		None
	})();
}

enum WordleMsg {
	SetConfig(Result<Config>),
}
struct Wordle {
	config: Option<Config>,
}

impl Component for Wordle {
	type Message = WordleMsg;
	type Properties = ();

	fn create(_: &Context<Self>) -> Self {
		Self {
			config: None
		}
	}

	fn update(&mut self, _: &Context<Self>, msg: Self::Message) -> bool {
		match msg {
			WordleMsg::SetConfig(config) => match config {
				Ok(config) => self.config = Some(config),
				Err(err) => alert(&err.to_string()),
			},
		}
		return true;
	}

	fn view(&self, ctx: &Context<Self>) -> Html {
		// Adapted from https://github.com/yewstack/yew/blob/dbdd3b78e1f0aada1834dec5c6ee83449db9d220/examples/communication_child_to_parent/src/parent.rs#L45
		let set_config = ctx.link().callback(WordleMsg::SetConfig);

		return match &self.config {
			None => html!(<FormConfig {set_config} />),
			Some(config) => html!(<GameBoard config={config.clone()} />),
		};
	}
}

#[derive(PartialEq, Properties)]
struct FormConfigProps {
	set_config: Callback<Result<Config>>,
}

#[function_component]
fn FormConfig(props: &FormConfigProps) -> Html {
	// Adapted from https://github.com/yewstack/yew/blob/dbdd3b78e1f0aada1834dec5c6ee83449db9d220/examples/communication_child_to_parent/src/child.rs#L26
	let onsubmit = props.set_config.reform(|e: SubmitEvent| -> Result<Config> {
		e.prevent_default();
		// Adapted from https://github.com/yewstack/yew/issues/474
		let form: HtmlFormElement = e.target_unchecked_into();
		let form = FormData::new_with_form(&form).unwrap();

		let word_src = if form.get("word_src") == JsValue::from_str("select") {
			let err = anyhow!("invalid word {:?}", form.get("word"));
			WordSrc::Select(word_from_str(&form.get("word").as_string().ok_or(err)?)?)
		} else {
			let err = anyhow!("invalid random seed: {:?}", form.get("seed"));
			WordSrc::Random(form.get("seed").as_string().ok_or(err)?.parse::<u64>()?, 1)
		};

		fn parse_list<T: FromIterator<Word>>(list: &str) -> Result<T> {
			list.trim()
				.split('\n')
				.map(|str| word_from_str(str))
				.collect()
		}
		let set_acceptable: HashSet<Word> = parse_list(&form.get("list_acceptable").as_string().unwrap())?;
		let list_final: Vec<[char; 5]> = parse_list(&form.get("list_final").as_string().unwrap())?;
		for word in &list_final {
			if !set_acceptable.contains(word) {
				return Err(anyhow!("word {} in final list, but not in acceptable list", word_to_str(word)));
			}
		}

		return Ok(Config {
			difficult: form.get("difficult") == JsValue::from_str("on"),
			stats: true,
			word_src,
			set_acceptable,
			set_final: Default::default(),
			list_final,
			state_src: None,
		});
	});

	let default_acceptable = builtin_words::ACCEPTABLE.join("\n");
	let default_final = builtin_words::FINAL.join("\n");

	return html!(
		<form {onsubmit}>
			<div class="config-row">
			<label> {"Difficult mode: "} </label>
			<input type="checkbox" name="difficult" />
			</div>

			<div class="config-row">
			<label> {"Game mode: "} </label>
			<select name="word_src">
					<option value="select"> {"Select"} </option>
					<option value="random"> {"Random"} </option>
			</select>
			</div>

			<div class="config-row">
			<label> {"Select Word: "} </label>
			<input type="text" name="word" />
			</div>

			<div class="config-row">
			<label> {"Random seed: "} </label>
			<input type="number" name="seed" value="0"/>
			</div>

			<div class="config-row">
			<label> {"Acceptable list: "} </label>
			<br />
			<textarea name="list_acceptable" value={default_acceptable}/>
			</div>

			<div class="config-row">
			<label> {"Final list: "} </label>
			<br />
			<textarea name="list_final" value={default_final}/>
			</div>

			<div class="config-row">
			<input type="submit" value="Start Wordle!" />
			</div>
		</form>
	);
}

#[derive(PartialEq, Properties)]
struct GameBoardProps {
	config: Config,
}

#[function_component]
fn GameBoard(props: &GameBoardProps) -> Html {
	let goal = match &props.config.word_src {
		WordSrc::Select(word) => word,
		WordSrc::Random(seed, date) => {
			&rand_words(&props.config.list_final, *seed, *date)().unwrap()
		}
		_ => unreachable!(),
	};
	let update_flag = use_state(|| 0);
	let plate = use_mut_ref(|| Plate::new(goal, props.config.difficult));
	let statistic = use_mut_ref(|| {
		let result = (|| -> Option<Statistic> {
			let storage = window()?.local_storage().ok()??;
			let statistic_json = storage.get_item("statistic").ok()??;
			Statistic::load_from_json(&statistic_json).ok()
		})();
		result.unwrap_or(Default::default())
	});

	let statistic_store = |statistic: &Statistic| {
		let _ = (|| -> Option<()> {
			let storage = window()?.local_storage().ok()??;
			let _ = storage.set_item("statistic", &statistic.store_to_json());
			return None;
		})();
	};
	let statistic_clear = {
		let update_flag = update_flag.clone();
		let statistic = statistic.clone();
		move |e: MouseEvent| {
			e.prevent_default();
			update_flag.set(*update_flag ^ 1);
			*statistic.borrow_mut() = Default::default();
			let _ = (|| -> Option<()> {
				let storage = window()?.local_storage().ok()??;
				let _ = storage.set_item("statistic", "");
				return None;
			})();
		}
	};

	let send_word = Rc::new(Cell::new(Callback::from({
		let update_flag = update_flag.clone();
		let plate = plate.clone();
		let set_acceptable = props.config.set_acceptable.clone();
		let statistic = statistic.clone();
		move |word: Word| {
			if set_acceptable.contains(&word) {
				update_flag.set(*update_flag ^ 1);
				let res = plate.borrow_mut().guess(&word);
				if let Err(err) = res {
					alert(&err.to_string());
				}
				// TODO: move alert to appropriate time
				if plate.borrow().is_win() {
					statistic.borrow_mut().add_plate(&plate.borrow());
					statistic_store(&statistic.borrow());
					alert("You win!");
				} else if plate.borrow().history().len() == 6 {
					statistic.borrow_mut().add_plate(&plate.borrow());
					statistic_store(&statistic.borrow());
					alert(&format!(
						"You lose! Anwer is {}",
						word_to_str(plate.borrow().goal())
					));
				}
			} else {
				alert(&format!("{} is not in acceptable list", word_to_str(&word)));
			}
		}
	})));

	return html!(
		<div class="app">
			<div class="plate">
			{
				(0..6usize).into_iter()
					.map(|id| {
						if id < plate.borrow().history().len() {
							html!( <WordColor ws={plate.borrow().history()[id]} />)
						} else if id == plate.borrow().history().len() && !plate.borrow().is_win() {
							html!( <WordInput send_word={send_word.take()} /> )
						} else {
							html!( <WordBlank /> )
						}
					})
				.collect::<Html>()
			}
			</div>
			<hr />
			<Keyboard keyboard={plate.borrow().keyboard().clone()} />
			<hr />
			<div class="statistic">
				<div class="statistic-row">
					{format!("Total win: {} Total lose: {}: Average attempts: {:.2}",
						statistic.borrow().success_cnt(),
						statistic.borrow().fail_cnt(),
						statistic.borrow().success_attempt_average())}
				</div>
				<div class="statistic-row">
					{format!("Top words: {}",
						statistic.borrow().top5_words().map(|x| format!("{}*{}", x.str, x.cnt)).collect::<Vec<String>>().join(" "))}
				</div>
				<div class="statistic-row">
					<a href="/" onclick={statistic_clear}> {"Clear statistic"} </a>
				</div>
			</div>
		</div>
	);
}

#[derive(PartialEq, Properties)]
struct KeyboardProps {
	keyboard: LetterMap<LetterState>,
}

#[function_component]
fn Keyboard(props: &KeyboardProps) -> Html {
	let get_row = |s: &str| {
		s.chars()
			.map(|c| html!(<LetterColor {c} s={props.keyboard[c]} />))
			.collect::<Html>()
	};
	html!(
		<div class="keyboard">
			<div class="keyboard-row"> {get_row("QWERTYUIOP")} </div>
			<div class="keyboard-row"> {get_row("ASDFGHJKL")} </div>
			<div class="keyboard-row"> {get_row("ZXCVBNM")} </div>
		</div>
	)
}

#[derive(PartialEq, Properties)]
struct WordColorProps {
	ws: (Word, WordState),
}

#[function_component]
fn WordColor(props: &WordColorProps) -> Html {
	let children: Vec<Html> = props
		.ws
		.0
		.iter()
		.zip(props.ws.1.iter())
		.map(|(c, s)| html!(<LetterColor c={*c} s={*s} />))
		.collect();
	return html!( <div class="plate-row"> { children } </div> );
}

#[derive(PartialEq, Properties)]
struct WordInputProps {
	send_word: Callback<Word>,
}

#[function_component]
fn WordInput(props: &WordInputProps) -> Html {
	let update_flag = use_state(|| 0);
	let letters = use_mut_ref(|| -> Vec<Letter> { Default::default() });

	let children: Vec<Html> = (0..5usize)
		.map(|id| {
			if id < letters.borrow().len() {
				html!(<LetterColor c={letters.borrow()[id]} s={LetterState::Unknown} />)
			} else {
				html!(<LetterColor c={' '} s={LetterState::Unknown} />)
			}
		})
		.collect();

	let onkeydown = {
		let update_flag = update_flag.clone();
		let letters = letters.clone();
		let send_word = props.send_word.clone();
		Callback::from(move |e: KeyboardEvent| {
			let code = e.key_code();
			if 65 <= code && code <= 90 {
				let c = code as u8 as char;
				if letters.borrow().len() < 5 {
					update_flag.set(*update_flag ^ 1);
					letters.borrow_mut().push(c);
				}
			} else if code == 8 {
				update_flag.set(*update_flag ^ 1);
				letters.borrow_mut().pop();
			} else if code == 13 {
				if letters.borrow().len() == 5 {
					let word: Word = letters.borrow().clone().try_into().unwrap();
					send_word.emit(word);
				}
			}
		})
	};

	return html!( <div class="plate-row"> { children } <input {onkeydown} id={"focus-me"}/> </div> );
}

#[function_component]
fn WordBlank() -> Html {
	let children: Vec<Html> = (0..5usize)
		.map(|_| html!(<LetterColor c={' '} s={LetterState::Unknown} />))
		.collect();
	return html!( <div class="plate-row"> { children } </div> );
}

#[derive(PartialEq, Properties)]
struct LetterColorProps {
	c: Letter,
	s: LetterState,
}

#[function_component]
fn LetterColor(props: &LetterColorProps) -> Html {
	match props.s {
		LetterState::Correct => {
			html!(<div class="letterbox letterbox-correct">   {props.c.to_string()} </div>)
		}
		LetterState::Occured => {
			html!(<div class="letterbox letterbox-occured">   {props.c.to_string()} </div>)
		}
		LetterState::Redundant => {
			html!(<div class="letterbox letterbox-redundant"> {props.c.to_string()} </div>)
		}
		LetterState::Unknown => {
			html!(<div class="letterbox letterbox-unknown">    {props.c.to_string()} </div>)
		}
	}
}

fn main() {
	yew::Renderer::<Wordle>::new().render();
}
