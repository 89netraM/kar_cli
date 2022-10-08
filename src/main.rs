mod bucket;
mod client;

use std::io::{stdin, stdout, Write};
use core::future::Future;

use anyhow::Result;
use rpassword::read_password;

use client::{Client, Error, Unauthorized};

#[tokio::main]
async fn main() {
	let result = act().await;
	if let Err(err) = result {
		eprintln!("Error: {err}.");
	}
}

async fn act() -> Result<()> {
	let mut client = make_client().await?;

	let mut result = client.get_card_balance().await;

	if let Err(Error::Unauthorized(Unauthorized::ExpiredToken(_))) = result {
		if client.refresh_token().await.is_ok() {
			result = client.get_card_balance().await;
		}
	}

	if result.is_err() {
		client = login_client().await?;
		result = client.get_card_balance().await;
	}

	let response = result?;

	println!("Your balance is: {:.2} SEK", response.balance);

	bucket::save(&client.access_token, &client.refresh_token)?;

	Ok(())
}

async fn make_client() -> Result<Client> {
	if let Some(token_pair) = bucket::read().ok().flatten() {
		Ok(Client::new(token_pair.access_token, token_pair.refresh_token)?)
	} else {
		Ok(login_client().await?)
	}
}

fn login_client() -> impl Future<Output = Result<Client, Error>> {
	let cid = ask("CID: ", read_answer);
	let password = ask("Password: ", || read_password().unwrap());

	Client::login(cid, password)
}

fn ask<R>(prompt: &str, read: R) -> String where R: Fn() -> String {
	loop {
		print!("{}", prompt);
		stdout().flush().unwrap();
		let answer = read();
		if !answer.is_empty() {
			return answer;
		}
	}
}

fn read_answer() -> String {
	let mut answer = String::new();
	stdin().read_line(&mut answer).unwrap();
	remove_last_new_line(&mut answer);
	answer
}

fn remove_last_new_line(s: &mut String) {
	s.pop();
	match s.pop() {
		Some('\r') => {}
		Some(c) => s.push(c),
		_ => {}
	}
}
