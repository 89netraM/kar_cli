# Kårappen API

Base URL: `https://backend.csu1.helops.net/api`

## Endpoints

* `POST` `/v1/auth/login`  
  Request and response body:  
  ```json
  {
  	"chalmersId": "<cid>",
  	"password": "<plain text password>"
  }
  ```  
  ```json
  {
  	"accessToken": "<accessToken>",
  	...
  }
  ```
* `POST` `/v1/auth/refresh-token`
* `GET` `/v1/events`*
* `GET` `/v1/microdeb/balance`*  
  Response body:  
  ```json
  {
  	"balance": <SEK>,
  	"shortpass": "<shortpass>",
  	...
  }
  ```
* `POST` `/v1/feedback`*
* `GET` `/v1/onboarding`*
* `GET` `/v1/restaurants`*
* `GET` `/v1/restaurants/<restaurant id>`*
* `GET` `/v1/fysiken/training-card`*
* `GET` `/v1/app-version`

\* Requires `Authorization` header on the format `Bearer <accessToken>`.
