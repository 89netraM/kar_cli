# Kår CLI

Fetch the balance of your Kårkort from the terminal!

Simply run `kar_cli`, enter CID and password, and you got your balance right
there!

Can optionally save your access token in your keyring (Credential Manager on
Windows).

## Top up

> **Warning**  
> Use at own risk!

With the `kar_cli topup <amount>` command you can top up your card via Swish.

Run the command, maybe enter CID+password, scan QR code (not with built-in Swish
scanner), pay, wait for command to confirm payment.

If the command fails without confirming payment please use the printed "Swish
request ID" with the status endpoint from the [Microdeb API](./Microdeb%20API.md)
documentation.
