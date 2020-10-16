# Elkato API

This is an **unofficial** API for the [Elkato](https://www.elkato.de) car sharing booking system. 

## Authentication

Elkato uses HTTP Basic Authentication, and the API proxy simply forwards the authorization header.

## Search

### GET `/{club}/bookings/current`

Get the bookings, starting 7 days back to 7 days into the future.
