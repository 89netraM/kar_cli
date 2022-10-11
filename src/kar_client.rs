use core::convert::From;
use core::fmt::{self, Display};
use core::str::FromStr;
use reqwest::{Client as ReqClient, RequestBuilder, Response, StatusCode, Url};
use serde::Serialize;
use serde::{de::DeserializeOwned, Deserialize};
use std::error;
#[cfg(debug_assertions)]
use std::error::Error as StdError;
use time::{macros::format_description, PrimitiveDateTime};

pub struct KarClient {
	base_url: Url,
	client: ReqClient,
	pub access_token: String,
	pub refresh_token: String,
}

impl KarClient {
	pub fn new(auth_token: String, refresh_token: String) -> Result<Self, Error> {
		Ok(Self {
			base_url: option_env!("KÅR_CLI_BASE_URL")
				.unwrap_or("https://backend.csu1.helops.net/api/v1/")
				.parse()?,
			client: ReqClient::new(),
			access_token: auth_token,
			refresh_token,
		})
	}

	pub async fn get_card_balance(&self) -> Result<BalanceResponse, Error> {
		self.make_request(self.client.get(self.base_url.join("microdeb/balance")?))
			.await
	}

	pub async fn login(cid: String, password: String) -> Result<Self, Error> {
		let mut client = Self::new(String::new(), String::new())?;

		let response: TokenPairResponse = client
			.make_request(
				client
					.client
					.post(client.base_url.join("auth/login")?)
					.json(&LoginRequest { cid, password }),
			)
			.await?;

		client.access_token = response.access_token;
		client.refresh_token = response.refresh_token;

		Ok(client)
	}

	pub async fn refresh_token(&mut self) -> Result<(), Error> {
		let response: TokenPairResponse = self
			.make_request(
				self.client
					.post(self.base_url.join("auth/refresh-token")?)
					.json(&TokenPairRequest {
						access_token: &self.access_token,
						refresh_token: &self.refresh_token,
					}),
			)
			.await?;

		self.access_token = response.access_token;
		self.refresh_token = response.refresh_token;

		Ok(())
	}

	async fn make_request<T: DeserializeOwned>(&self, builder: RequestBuilder) -> Result<T, Error> {
		let response = builder
			.header("Authorization", format!("Bearer {}", self.access_token))
			.send()
			.await?;

		let response = Error::for_unauthorized(response)?;

		let response = response.error_for_status()?;

		Ok(response.json().await?)
	}
}

#[derive(Deserialize)]
pub struct BalanceResponse {
	#[serde(rename = "balance")]
	pub balance: f64,
	#[serde(rename = "shortPass")]
	pub shortpass: String,
}

#[derive(Serialize)]
struct LoginRequest {
	#[serde(rename = "chalmersId")]
	pub cid: String,
	#[serde(rename = "password")]
	pub password: String,
}

#[derive(Deserialize, Serialize)]
pub struct TokenPair<T> {
	#[serde(rename = "accessToken")]
	pub access_token: T,
	#[serde(rename = "refreshToken")]
	pub refresh_token: T,
}

pub type TokenPairResponse = TokenPair<String>;
pub type TokenPairRequest<'s> = TokenPair<&'s str>;

#[derive(Debug)]
pub enum Error {
	UrlParse(url::ParseError),
	Reqwest(reqwest::Error),
	Unauthorized(Unauthorized),
}

impl Error {
	fn for_unauthorized(response: Response) -> Result<Response, Self> {
		if response.status() == StatusCode::UNAUTHORIZED {
			let reason = response
				.headers()
				.get("www-authenticate")
				.and_then(|h| h.to_str().ok())
				.and_then(|s| s.parse().ok());
			Err(Self::Unauthorized(reason.unwrap_or_else(|| {
				Unauthorized::ReqwestError(response.error_for_status().unwrap_err())
			})))
		} else {
			Ok(response)
		}
	}
}

impl Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::UrlParse(_) => write!(f, "Failed to resolve URL"),
			Self::Reqwest(_) => write!(f, "Request to service failed"),
			Self::Unauthorized(err) => {
				write!(f, "Unauthorized request ({err})")
			}
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
			Error::Unauthorized(_) => None,
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

#[derive(Debug)]
pub enum Unauthorized {
	ReqwestError(reqwest::Error),
	InvalidToken(),
	ExpiredToken(PrimitiveDateTime),
}

impl Display for Unauthorized {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Unauthorized::ReqwestError(err) => write!(f, "{err}"),
			Unauthorized::InvalidToken() => write!(f, "invalid_token"),
			Unauthorized::ExpiredToken(expiration) => write!(f, "token expired at {expiration}"),
		}
	}
}

impl FromStr for Unauthorized {
	type Err = ();

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let s = s.strip_prefix("Bearer").ok_or(())?;
		let ae = AuthenticationError::from_str(s);

		return if let Some(expiration) = ae.error_description.and_then(extract_expiration) {
			Ok(Unauthorized::ExpiredToken(expiration))
		} else if ae.error == Some("invalid_token") {
			Ok(Unauthorized::InvalidToken())
		} else {
			Err(())
		};

		fn extract_expiration(s: &str) -> Option<PrimitiveDateTime> {
			let format = format_description!("[month]/[day]/[year] [hour]:[minute]:[second]");

			let (_, s) = s.split_once('\'')?;
			let (s, _) = s.split_once('\'')?;
			PrimitiveDateTime::parse(s, format).ok()
		}
	}
}

#[derive(Debug)]
struct AuthenticationError<'s> {
	error: Option<&'s str>,
	error_description: Option<&'s str>,
}

impl<'s> AuthenticationError<'s> {
	pub fn from_str(mut s: &'s str) -> Self {
		let mut result = Self {
			error: None,
			error_description: None,
		};

		while let Some((name, value, next_s)) = read_field(s) {
			result.update_field(name, value);
			s = next_s;
		}

		return result;

		fn read_field(s: &str) -> Option<(&str, &str, &str)> {
			let s = s.trim_start();

			let (name, s) = s.split_once('=')?;

			let s = s.strip_prefix('"')?;

			let (value, s) = s.split_once('"')?;

			Some((name, value, s.strip_prefix(',').unwrap_or(s)))
		}
	}

	fn update_field(&mut self, name: &str, value: &'s str) {
		match name {
			"error" => self.error = trim_to_none(value),
			"error_description" => self.error_description = trim_to_none(value),
			_ => {}
		}

		fn trim_to_none(s: &str) -> Option<&str> {
			match s.trim() {
				"" => None,
				s => Some(s),
			}
		}
	}
}
