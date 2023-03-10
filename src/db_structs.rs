use serde::de::{self, Deserializer};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Postgres};
use std::fmt::Display;
use std::str::FromStr;

//inputs; input JSON -> serde -> these structs
#[derive(Deserialize)]
pub struct Login {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct PatientID {
    #[serde(deserialize_with = "from_str")]
    pub patient_id: i64,
}

#[derive(Deserialize)]
pub struct Patient {
    pub name: String,
    pub email: String,
    pub phone: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct PatientInfoInput {
     #[serde(deserialize_with = "from_str")]
    pub patient_id: i64,
    pub gender: String,
    #[serde(deserialize_with = "from_str")]
    pub weight: i32,
    #[serde(deserialize_with = "from_str")]
    pub age: i32,
    pub blood_group: String
}

#[derive(Deserialize)]
pub struct PrescriptionInfoInput {
     #[serde(deserialize_with = "from_str")]
    pub patient_id: i32,
    #[serde(deserialize_with = "from_str")]
    pub doctor_id: i32,
    pub prescription: String,
    pub date: String
}


#[derive(Deserialize)]
pub struct DoctorDate {
    #[serde(deserialize_with = "from_str")]
    pub doctor_id: i64,
    pub date: String
}

#[derive(Deserialize)]
pub struct DoctorPatientDate {
    #[serde(deserialize_with = "from_str")]
    pub doctor_id: i64,
    #[serde(deserialize_with = "from_str")]
    pub patient_id: i64,
    pub date: String
}

#[derive(Deserialize)]
pub struct Doctor {
    pub name: String,
    #[serde(deserialize_with = "from_str")]
    pub speciality: i64,
    pub city: String,
    pub address: String,
    pub email: String,
    pub phone: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct City {
    pub city: String,
}

#[derive(Deserialize)]
pub struct CityApptype {
    pub city: String,
    pub apptype: String,
}

#[derive(Deserialize)]
pub struct Appointment {
    #[serde(deserialize_with = "from_str")]
    pub doctor_id: i64,
    #[serde(deserialize_with = "from_str")]
    pub patient_id: i64,
    #[serde(deserialize_with = "from_str")]
    pub apptype: i64,
    #[serde(deserialize_with = "from_str")]
    pub slot_id: i64,
    pub date: String,
    pub phyorvirt: String,
    pub symptom: String,
}

#[derive(Deserialize)]
pub struct Token {
    #[serde(deserialize_with = "from_str")]
    pub doctor_id: i64,
    #[serde(deserialize_with = "from_str")]
    pub patient_id: i64,
    #[serde(deserialize_with = "from_str")]
    pub apptype: i64,
    #[serde(deserialize_with = "from_str")]
    pub date: String,
    pub symptom: String,
}

#[derive(Deserialize)]
pub struct CancelAppointment {
    #[serde(deserialize_with = "from_str")]
    pub doctor_id: i64,
    #[serde(deserialize_with = "from_str")]
    pub patient_id: i64,
    pub date: String,
}

#[derive(Deserialize)]
pub struct Registration {
    pub email: String,
    pub password: String,
    pub isdoctor: bool,
}

//outputs; SQL query -> sqlx -> these structs -> serde -> output JSON
#[derive(FromRow, Serialize)]
pub struct TokenNumber {
    pub num: i32,
}

#[derive(FromRow, Serialize)]
pub struct TokenNumberPrimary {
    pub num: i64,
}


#[derive(FromRow, Serialize)]
pub struct Timeslots {
    time_start: String,
    available: bool,
    slot_id: i64,
}

#[derive(FromRow, Serialize)]
pub struct Prescriptions {
    docname: String,
    date: String,
    prescription: String,
}

#[derive(FromRow, Serialize)]
pub struct PrevAppointments {
    docname: String,
    date: String,
    phyorvirt: String,
    appstatus: String,
    prescription_id: i32,
    appname: String,
}

#[derive(FromRow, Serialize)]
pub struct DoctorInfo {
    docid: i64,
    docname: String,
    specname: String,
    address: String,
}

#[derive(FromRow, Serialize)]
pub struct PatientInfo {
    name: String,
    email: String,
    phone: String,
    gender: String,
    weight: i32,
    age: i32,
    blood_group: String
}

#[derive(FromRow, Serialize)]
pub struct DoctorPrices {
    docid: i64,
    docname: String,
    city: String,
    address: String,
    apptype: String,
    appid: i64,
    price: i32,
    specname: String,
}

#[derive(FromRow, Serialize)]
pub struct Apptypes {
    id: i64,
    name: String
}

#[derive(FromRow, Serialize)]
pub struct Cities {
    city: String
}


#[derive(FromRow, Serialize)]
pub struct DoctorAppointments {
    #[serde(deserialize_with = "from_str")]
    id: i64,
    #[serde(deserialize_with = "from_str")]
    patient_id: i32,
    #[serde(deserialize_with = "from_str")]
    apptype: i32,
    pub date: String,
    phyorvirt: String,
    pub status: String,
    #[serde(deserialize_with = "from_str")]
    pub slot_id: i32,
    pub symptom: String,
}

#[derive(FromRow, Serialize)]
pub struct EmergencyAppointments {
    #[serde(deserialize_with = "from_str")]
    id: i32,
    #[serde(deserialize_with = "from_str")]
    patient_id: i32,
    #[serde(deserialize_with = "from_str")]
    apptype: i32,
    pub symptom: String,
}

#[derive(FromRow, Serialize)]
pub struct Specialities {
    id: i64,
    name: String,
    desc: String,
}

#[derive(FromRow, Serialize)]
pub struct LoginTable {
    pub salt: String,
    pub hashedpass: String,
    pub isdoctor: bool,
}

#[derive(Serialize, Deserialize)]
pub struct JWT {
    pub isdoctor: bool,
    #[serde(deserialize_with = "from_str")]
    pub id: i64,
}

#[derive(Serialize, Deserialize)]
pub struct InternalJWT {
    pub isdoctor: bool,
    pub id: String,
    pub exp: usize,
}

//function to convert the input string into a number with some Serde magic
fn from_str<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: FromStr,
    T::Err: Display,
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    T::from_str(&s).map_err(de::Error::custom)
}
