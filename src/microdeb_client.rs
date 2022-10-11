use core::convert::From;
use core::fmt::{self, Display};
use reqwest::{Client as ReqClient, Url};
use serde::Deserialize;
use serde::Serialize;
use std::error;
#[cfg(debug_assertions)]
use std::error::Error as StdError;

pub struct MicrodebClient {
	base_url: Url,
	app_id: &'static str,
	client: ReqClient,
}

impl MicrodebClient {
	pub fn new() -> Result<Self, Error> {
		Ok(Self {
			base_url: option_env!("KÅR_CLI_MICRODEB_URL")
				.unwrap_or("https://ragnarok.microdeb.se/api/v1/")
				.parse()?,
			app_id: option_env!("KÅR_CLI_APP_ID").unwrap_or("bf9b5e70-ab62-49d8-95d9-59f8febc265a"),
			client: ReqClient::new(),
		})
	}

	pub async fn login(&self, shortpass: &str) -> Result<LoginResponse, Error> {
		self.client
			.get(self.base_url.join(&format!("login/{}/shortpass", self.app_id))?)
			.query(&[("q", shortpass)])
			.send()
			.await?
			.error_for_status()?
			.json()
			.await
			.map_err(From::from)
	}

	pub async fn swish_create(&self, amount: u64, user_id: &str, card_number: &str) -> Result<SwishResponse, Error> {
		self.client
			.post(self.base_url.join(&format!("swish/{}/create", self.app_id))?)
			.json(&SwishCreateRequest {
				amount,
				message: Some("MPS Microdeb Me"),
				reference: card_number,
				card_number,
				user_id,
			})
			.send()
			.await?
			.error_for_status()?
			.json()
			.await
			.map_err(From::from)
	}

	pub async fn swish_status(&self, id: &str) -> Result<SwishResponse, Error> {
		self.client
			.get(self.base_url.join(&format!("swish/{}/status", self.app_id))?)
			.query(&[("identifier", id)])
			.send()
			.await?
			.error_for_status()?
			.json()
			.await
			.map_err(From::from)
	}
}

#[derive(Deserialize)]
pub struct LoginResponse {
	#[serde(rename = "user")]
	pub user: User,
	#[serde(rename = "information")]
	pub card: Card,
}

#[derive(Deserialize)]
pub struct User {
	#[serde(rename = "identifier")]
	pub id: String,
}

#[derive(Deserialize)]
pub struct Card {
	#[serde(rename = "cardNumber")]
	pub number: String,
}

#[derive(Serialize)]
pub struct SwishCreateRequest<'m, 'c, 'u> {
	#[serde(rename = "amount")]
	pub amount: u64,
	#[serde(rename = "message")]
	pub message: Option<&'m str>,
	#[serde(rename = "reference")]
	pub reference: &'c str,
	#[serde(rename = "cardNumber")]
	pub card_number: &'c str,
	#[serde(rename = "userIdentifier")]
	pub user_id: &'u str,
}

#[derive(Deserialize)]
pub struct SwishResponse {
	#[serde(rename = "data")]
	pub data: SwishData,
	#[serde(rename = "identifier")]
	pub id: String,
}

#[derive(Deserialize)]
pub struct SwishData {
	#[serde(rename = "swish_token")]
	pub token: String,
	#[serde(rename = "status")]
	pub status: SwishStatus,
}

#[derive(Deserialize, PartialEq, Eq)]
pub enum SwishStatus {
	#[serde(rename = "new")]
	New,
	#[serde(rename = "settled")]
	Settled,
}

#[derive(Debug)]
pub enum Error {
	UrlParse(url::ParseError),
	Reqwest(reqwest::Error),
}

impl Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::UrlParse(_) => write!(f, "Failed to resolve URL"),
			Self::Reqwest(_) => write!(f, "Request to service failed"),
		}?;
		#[cfg(debug_assertions)]
		if let Some(err) = self.source() {
			write!(f, " ({err})")?;
		}
		Ok(())
	}
}

impl error::Error for Error {
	fn source(&self) -> Option<&(dyn error::Error + 'static)> {
		match self {
			Error::UrlParse(err) => Some(err),
			Error::Reqwest(err) => Some(err),
		}
	}
}

impl From<url::ParseError> for Error {
	fn from(err: url::ParseError) -> Self {
		Self::UrlParse(err)
	}
}

impl From<reqwest::Error> for Error {
	fn from(err: reqwest::Error) -> Self {
		Self::Reqwest(err)
	}
}
