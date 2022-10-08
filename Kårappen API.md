# Kårappen API

Base URL: `https://backend.csu1.helops.net/api`

## Endpoints

* `POST` `/v1/auth/login`  
  Request body `{ "chalmersId": "<cid>", "password": "<plain text password>" }`  
  Response body `{ "accessToken": "<accessToken>", ... }`
* `POST` `/v1/auth/refresh-token`
* `GET` `/v1/events`*
* `GET` `/v1/microdeb/balance`*
* `POST` `/v1/feedback`*
* `GET` `/v1/onboarding`*
* `GET` `/v1/restaurants`*
* `GET` `/v1/restaurants/<restaurant id>`*
* `GET` `/v1/fysiken/training-card`*
* `GET` `/v1/app-version`

\* Requires `Authorization` header on the format `Bearer <accessToken>`.
