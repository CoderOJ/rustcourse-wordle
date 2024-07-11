use {
	anyhow::{anyhow, Result},
	std::{cell::Cell, rc::Rc},
	web_sys::{wasm_bindgen::JsValue, window, FormData, HtmlFormElement},
	wordle::{builtin_words, config::*, plate::*, util::LetterMap, word_gen::rand_words},
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

		fn parse_builtin_list<T: FromIterator<Word>>(list: &[&str]) -> T {
			list.iter().map(|&s| word_from_str(s).unwrap()).collect()
		}
		return Ok(Config {
			difficult: form.get("difficult") == JsValue::from_str("on"),
			stats: true,
			word_src,
			set_acceptable: parse_builtin_list(builtin_words::ACCEPTABLE),
			set_final: Default::default(),
			list_final: parse_builtin_list(builtin_words::FINAL),
			state_src: Some(format!("{:?}", form.get("seed"))),
		});
	});

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

	let send_word = Rc::new(Cell::new(Callback::from({
		let update_flag = update_flag.clone();
		let plate = plate.clone();
		let set_acceptable = props.config.set_acceptable.clone();
		move |word: Word| {
			if set_acceptable.contains(&word) {
				update_flag.set(*update_flag ^ 1);
				let res = plate.borrow_mut().guess(&word);
				if let Err(err) = res {
					alert(&err.to_string());
				}
				// TODO: move alert to appropriate time
				if plate.borrow().is_win() {
					alert("You win!");
				} else if plate.borrow().history().len() == 6 {
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
