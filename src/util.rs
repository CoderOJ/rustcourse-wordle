use {
	arr_macro::arr,
	std::ops::{Index, IndexMut},
};

/// Specialized map for 26 letters
#[derive(Clone, Copy, PartialEq)]
pub struct LetterMap<T> {
	_data: [T; 26],
}

impl<T> Default for LetterMap<T>
where
	T: Default,
{
	fn default() -> Self {
		Self {
			_data: arr![Default::default(); 26],
		}
	}
}

impl<T> Index<char> for LetterMap<T> {
	type Output = T;
	fn index(&self, index: char) -> &Self::Output {
		&self._data[(index as usize) - 65]
	}
}

impl<T> IndexMut<char> for LetterMap<T> {
	fn index_mut(&mut self, index: char) -> &mut Self::Output {
		&mut self._data[(index as usize) - 65]
	}
}

impl<T> LetterMap<T> {
	pub fn as_arr(&self) -> &[T; 26] {
		&self._data
	}
	pub fn as_mut_arr(&mut self) -> &mut [T; 26] {
		&mut self._data
	}
}

pub fn loop_on_err_with<T, E, F: FnMut() -> Result<T, E>, FE: FnMut(E) -> ()>(
	mut f: F,
	mut fe: FE,
) -> T {
	loop {
		match f() {
			Ok(r) => break r,
			Err(e) => fe(e),
		}
	}
}
