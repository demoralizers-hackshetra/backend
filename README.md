# Backend

This repo hosts the backend for the Productathon 2023 project from team Tech Titans.

It is being written in Rust using [Axum web framework](https://lib.rs/crates/axum) and [sqlx](https://lib.rs/crates/sqlx) for interfacing with a Postgres database instance, along with other miscellaneous dependencies like Tokio.

## Setup

Make sure you have Postgres instance and Rust toolchain running on your system.

First, populate ```setup.env``` with DATABASE_URL according to [PostgreSQL standards](https://www.postgresql.org/docs/current/libpq-connect.html#LIBPQ-CONNSTRING), and a SECRET (which is a random string which will be used to generate JWTs)

Then, rename ```setup.env``` to anything that begins with .env, like ```.env```.

Then, run the following commands related to creating the database and tables (one time measure to setup development environment):
```
createdb <dbname you gave in DATABASE_URL>
psql <dbname you gave in DATABASE_URL> -f src/schema.sql
```

Feel free to add some dummy data at this stage or use the dummy data contained in ```src/dummydata.sql``` to get some sample data by running the following command:

```
psql <dbname you gave in DATABASE_URL> -f src/dummydata.sql
```

Note: this does NOT contain a single record for the login table! You will need to use the ```/newdoctor``` or ```/newpatient``` endpoints to create a new doctor/patient which will also insert into these tables. You can then use these credentials in the API testing to make sure authentication works as intended

Then, run the project using ```cargo run```. It will run on port 3000. For log messages, use the ```RUST_LOG``` env variable (setting to debug usually prints good messages to understand what is going on)

## Endpoints

|URL| Type | Description | Parameters | Authentication Needed? | Output
---|---|---|---|---|---
|/cities | GET | Gets all cities where doctors are available according to us | Nothing | No |  Array of 'city' key and value is name of city
|/find| GET | Finds doctors in city specified who can give appointment for specified appointment type | city, apptype (both as queries in URL) | No | Array of address, appid (appointment ID), apptype (Appointment type), city, docid (doctor ID), docname (doctor's name), price (price they charge for that service), specname (speciality name of the doctor)
|/emergency/find| GET | Finds doctors in city specified who can give appointment in emergency for specified appointment type | city, apptype (both as queries in URL) | No | Array of address, appid (appointment ID), apptype (Appointment type), city, docid (doctor ID), docname (doctor's name), price (price they charge for that service), specname (speciality name of the doctor)
|/doctors | POST | Displays doctors in a particular city | city (POST request) | No | address, docid (doctor ID), docname (doctor's name), specname (specialization name)
|/doctor/timeslots | POST | Gets the timeslots in which doctor is available along with whether or not it has already been booked | doctor_id, date (specific format of YYYY-MM-DD)| No | Array of time_start (which is when the timeslot actually starts) and available (boolean of whether or not the doctor is available, ie that appointment slot is available), along with slot_id
|/doctor/newtoken | POST | Gets the current next token number for doctor on particular day| doctor_id, date (specific format of YYYY-MM-DD)| No | num (which is next token that would be generated)
|/doctor/curtoken | POST | Gets the current token number for doctor on particular day that is being served | doctor_id, date (specific format of YYYY-MM-DD)| No | num (which is current token that is being serviced by doctor)
|/newappointment | POST | Add new appointment to database | doctor_id, patient_id, apptype (as an ID), date (specific format of YYYY-MM-DD), phyorvirt (just write either physical or virtual checkup), slot_id, symptom | Yes | HTTP Status Code 200 if booked, something else if not, refer to table below to interpret status codes
|/newtoken | POST | Add new token to database | doctor_id, patient_id, apptype (as an ID), date (specific format of YYYY-MM-DD), symptom | Yes | HTTP Status Code 200 if booked, something else if not, refer to table below to interpret status codes, note that duplicate combos of (doctor_id, patient_id, apptype, date) are NOT allowed to prevent someone hoarding tokens for same appointment
|/newemergency | POST | Add new emergency to database | doctor_id, patient_id, apptype (as an ID), date (specific format of YYYY-MM-DD), symptom | Yes | HTTP Status Code 200 if booked, something else if not, refer to table below to interpret status codes, note that duplicate combos of (doctor_id, patient_id, apptype, date) are NOT allowed to prevent someone hoarding tokens for same appointment
|/doctorappointments | POST | Gets the doctor's appointments | doctor_id, date| Yes | apptype, date, id (appointment ID),patient_id, phyorvirt, slot_id, status, symptom
|/emergency/appointments | POST | Gets the doctor's emergency appointments | patient_id (it recycles the same struct so just name it as such, it is interpreted as a doctor's ID only) | Yes | id (emergency no),patient_id, symptom, apptype
|/login | POST | Generate JWT for a user (doctor or patient) | email, password | No (JWT is used as token to get authentication implemented) | Gets a JWT in case login was successful, else check HTTP status code
|/newpatient | POST | Adds patient details to database | name, phone, email, password | Will be used for signup process | Status Code based
|/newdoctor | POST | Adds doctor details to database | name, speciality (as an ID), city, address, phone, email, password | Will be used for signup process | Status Code based
|/patient | POST | Displays info about patient | patient_id (POST request) | Yes | name, email, phone, gender, weight (in kg), blood_group
|/patient/token | POST | Displays the token booked by patient | patient_id, doctor_id, date | Yes | num (token number the patient has been assigned)
|/patient/token | POST | Displays the token booked by patient | patient_id, doctor_id, date | Yes | num (token number the patient has been assigned)
|/patient/update | POST | Updates patient details all at once | patient_id, gender, weight, age, blood_group | Yes | Status code based
|/apptypes | GET | Gets appointment types | Nothing | No | id (appointment ID) and name
|/specialities | GET | Gets speciality details | Nothing | No | id (speciality ID), desc (description), name
|/prevapp | POST | Displays the previous appointments for particular patient | patient_id (POST request) | Yes | appname (appointment type), status, phyorvirt, date, docname, prescription_id
|/newprescription | POST | Creates a new prescription for the patient | patient_id, doctor_id, prescription, date | Yes | Status code based
|/prescriptions | POST | Get the prescriptions issued to patient | patient_id | Yes | docname, date, prescription
|/cancelappointment | POST | Cancel a previously booked appointment | doctor_id, patient_id, date | Yes | Status code based

## Response Codes

|Number|Name|Description|
---|---|---
200|OK|Everything checked out, request is good
500|Internal Server Error| There is a problem with connecting to the database
401| Unauthorized| You didn't provide the right authorization token (the JWT) or it was not provided properly. In whatever case, you don't have the right to view what you requested so it was denied
400| Bad Request | This is returned whenever the database has no records for your request. It's intended as a shorthand to save you time to check whether you received *any* records
405 | Method Not Allowed| You should only make a POST request to an endpoint that expects a POST request and a GET request to one that expects a GET request
