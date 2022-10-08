use core::convert::From;
use core::fmt::{self, Display};
use std::env;
use std::error;
use std::io::{stdin, stdout, Write};

use keyring::{Entry, Error as KeyError};
use serde_json::Error as SerdeJsonError;

use super::client::{TokenPairRequest, TokenPairResponse};

pub fn read() -> Result<Option<TokenPairResponse>, Error> {
	let entry = get_entry();
	let json = match entry.get_password() {
		Ok(json) => json,
		Err(_) => return Ok(None),
	};

	Ok(serde_json::from_str(&json)?)
}

pub fn save(access_token: &str, refresh_token: &str) -> Result<(), Error> {
	match read()? {
		Some(old) if old.access_token != access_token || old.refresh_token != refresh_token => {
			write(access_token, refresh_token)?;
		}
		None if ask_should_save() => write(access_token, refresh_token)?,
		_ => {}
	}

	Ok(())
}

fn ask_should_save() -> bool {
	let mut answer = String::new();
	loop {
		print!("Save access token? (Y/n) ");
		stdout().flush().unwrap();
		stdin().read_line(&mut answer).unwrap();

		if answer.chars().all(|c| c.is_ascii_whitespace())
			|| answer.starts_with(|c: char| c.to_ascii_lowercase() == 'y')
		{
			return true;
		} else if answer.starts_with(|c: char| c.to_ascii_lowercase() == 'n') {
			return false;
		}

		answer.clear();
	}
}

fn write(access_token: &str, refresh_token: &str) -> Result<(), Error> {
	let json = serde_json::to_string(&TokenPairRequest {
		access_token,
		refresh_token,
	})?;

	let entry = get_entry();
	entry.set_password(&json)?;

	Ok(())
}

fn get_entry() -> Entry {
	let username = whoami::username();
	let current_exe = env::current_exe().unwrap();
	let service = current_exe.file_name().unwrap().to_string_lossy();
	Entry::new(&service, &username)
}

#[derive(Debug)]
pub enum Error {
	Keyring(KeyError),
	Json(SerdeJsonError),
}

impl Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Keyring(_) => write!(f, "Interaction with keyring failed"),
			Self::Json(_) => write!(f, "Failed to format/parse keyring value"),
		}
	}
}

impl error::Error for Error {
	fn source(&self) -> Option<&(dyn error::Error + 'static)> {
		match self {
			Error::Keyring(err) => Some(err),
			Error::Json(err) => Some(err),
		}
	}
}

impl From<KeyError> for Error {
	fn from(err: KeyError) -> Self {
		Self::Keyring(err)
	}
}

impl From<SerdeJsonError> for Error {
	fn from(err: SerdeJsonError) -> Self {
		Self::Json(err)
	}
}
