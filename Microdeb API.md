# Microdeb API

Base URL: `https://ragnarok.microdeb.se/api`  
`appId`: `bf9b5e70-ab62-49d8-95d9-59f8febc265a`

Full API "documentation": https://ragnarok.microdeb.se/swagger/v1/swagger.json

## Endpoints

* `GET`: `v1/login/<appId>/shortpass?q=<shortpass>`  
  Response body:  
  ```json
  {
  	"user" : {
  		"identifier": "<User GUID>",
  		...
  	},
  	"information": {
  		"cardNumber": "<Card number>",
  		...
  	},
  	...
  }
  ```
* `POST`: `v1/swish/<appId>/create`  
  Request and response body:  
  ```json
  {
  	"amount": <SEK>,
  	"message": "MPS Microdeb Me",
  	"reference": "<Card number>",
  	"cardNumber": "<Card number>",
  	"userIdentifier": "<User GUID>"
  }
  ```  
  ```json
  {
  	"data": {
  		"swish_token": "<Swish payment request token>",
  		...
  	},
  	"identifier": "<swishId>"
  }
  ```
* `GET`: `v1/swish/<appId>/status?identifier=<swishId>`  
  Response body:  
  ```json
  {
  	"data": {
  		"status": "<new | settled>",
  		...
  	},
  	...
  }
  ```

After the Swish payment has been completed, a request to the swish status
endpoint must be made in order for the system to update the balance.
