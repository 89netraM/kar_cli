mod bucket;
mod kar_client;
mod microdeb_client;

use core::future::Future;
use std::io::{stdin, stdout, Write};

use anyhow::Result;
use clap::{command, Parser, Subcommand};

use kar_client::{BalanceResponse, Error, KarClient, Unauthorized};
use microdeb_client::MicrodebClient;
use qrcode::{render::unicode, QrCode};

use crate::microdeb_client::SwishStatus;

#[derive(Parser)]
#[command(
	about = "Gets current balance without explicit command. Top up command lets you specify\namount of SEK to add to your card."
)]
struct Cli {
	#[command(subcommand)]
	command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
	#[command(name = "topup")]
	TopUp { amount: u64 },
}

#[tokio::main]
async fn main() {
	let cli = Cli::parse();

	let result = match cli.command {
		Some(Commands::TopUp { amount }) => topup(amount).await,
		None => show_balance().await,
	};
	if let Err(err) = result {
		eprintln!("Error: {err}.");
	}
}

async fn show_balance() -> Result<()> {
	let (client, response) = get_balance_response().await?;

	println!("Your balance is: {:.2} SEK", response.balance);

	bucket::save(&client.access_token, &client.refresh_token)?;

	Ok(())
}

async fn topup(amount: u64) -> Result<()> {
	let (kar_client, balance_response) = get_balance_response().await?;

	let microdeb_client = MicrodebClient::new()?;

	let user_info = microdeb_client.login(&balance_response.shortpass).await?;
	let mut swish_response = microdeb_client
		.swish_create(amount, &user_info.user.id, &user_info.card.number)
		.await?;
	let swish_id = swish_response.id;
	println!("Swish request ID: {swish_id}");

	let swish_link = format!("swish://paymentrequest?token={}", swish_response.data.token);
	let swish_qr = QrCode::new(swish_link)?;
	let qr_text = swish_qr
		.render::<unicode::Dense1x2>()
		.dark_color(unicode::Dense1x2::Light)
		.light_color(unicode::Dense1x2::Dark)
		.build();
	println!("{}", qr_text);

	while swish_response.data.status != SwishStatus::Settled {
		swish_response = microdeb_client.swish_status(&swish_id).await?;
	}

	println!("Payment complete!");

	bucket::save(&kar_client.access_token, &kar_client.refresh_token)?;

	Ok(())
}

async fn get_balance_response() -> Result<(KarClient, BalanceResponse), Error> {
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

	Ok((client, result?))
}

async fn make_client() -> Result<KarClient, Error> {
	if let Some(token_pair) = bucket::read().ok().flatten() {
		Ok(KarClient::new(token_pair.access_token, token_pair.refresh_token)?)
	} else {
		Ok(login_client().await?)
	}
}

fn login_client() -> impl Future<Output = Result<KarClient, Error>> {
	let cid = ask("CID: ", read_answer);
	let password = ask("Password: ", read_password);

	KarClient::login(cid, password)
}

fn ask<R, O>(prompt: &str, read: R) -> O
where
	R: Fn() -> Option<O>,
{
	loop {
		print!("{}", prompt);
		stdout().flush().unwrap();
		if let Some(output) = read() {
			return output;
		}
	}
}

fn read_answer() -> Option<String> {
	let mut answer = String::new();
	stdin().read_line(&mut answer).unwrap();
	trim_to_none(answer)
}

fn read_password() -> Option<String> {
	rpassword::read_password().ok().and_then(trim_to_none)
}

fn trim_to_none(mut s: String) -> Option<String> {
	let l = s.trim_end().len();
	s.truncate(l);
	if s.is_empty() {
		None
	} else {
		Some(s)
	}
}
