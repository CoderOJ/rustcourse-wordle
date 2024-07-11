use {
	anyhow::{anyhow, Result},
	web_sys::{wasm_bindgen::JsValue, window, FormData, HtmlFormElement},
	wordle::{config::*, plate::word_from_str},
	yew::prelude::*,
};

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
				Err(err) => window()
					.unwrap()
					.alert_with_message(&err.to_string())
					.unwrap(),
			},
		}
		return true;
	}

	fn view(&self, ctx: &Context<Self>) -> Html {
		// Adapted from https://github.com/yewstack/yew/blob/dbdd3b78e1f0aada1834dec5c6ee83449db9d220/examples/communication_child_to_parent/src/parent.rs#L45
		let set_config = ctx.link().callback(WordleMsg::SetConfig);

		return match &self.config {
			None => html!(<FormConfig {set_config} />),
			Some(config) => html!(<div> { format!("start game with config: {:?}", config) } </div>),
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

		return Ok(Config {
			difficult: form.get("difficult") == JsValue::from_str("on"),
			stats: true,
			word_src,
			set_acceptable: Default::default(),
			set_final: Default::default(),
			list_final: Default::default(),
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
			<label> {"Random seed (default 0): "} </label>
			<input type="number" name="seed" />
			</div>

			<div class="config-row">
			<input type="submit" value="Start Wordle!" />
			</div>
		</form>
	);
}

fn main() {
	yew::Renderer::<Wordle>::new().render();
}
