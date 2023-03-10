//create structs for interfacing with the database
use argon_hash_password;
use chrono::NaiveDate;
use dotenvy::dotenv;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use sqlx::{postgres::PgPoolOptions, postgres::PgRow, Pool, Postgres, Row};
use std::env;
use tracing;

use crate::db_structs::*;

pub struct Database {
    jwt_secret: Vec<u8>,
    connection: Pool<Postgres>,
}

pub async fn init() -> Option<Database> {
    dotenv().ok();
    let Ok(url) = env::var("DATABASE_URL") else {
        tracing::error!("Couldn't find DATABASE_URL, aborting");
        return None;
    };
    let Ok(sec) = env::var("SECRET") else {
        tracing::error!("Couldn't find SECRET, aborting");
        return None;
    };
    tracing::debug!("Found database URL: {} and secret", url);
    match PgPoolOptions::new().connect(&url).await {
        Ok(pool) => {
            tracing::debug!("Connected to database!");
            return Some(Database {
                connection: pool,
                jwt_secret: sec.as_bytes().to_vec(),
            });
        }
        Err(e) => {
            tracing::error!("Could not connect using URL {}", url);
            tracing::error!("Error: {}", e);
            return None;
        }
    }
}

impl Database {
    async fn get_query_result<ResultStruct, DB>(&self, query: &String) -> Vec<ResultStruct>
    where
        ResultStruct: for<'r> sqlx::FromRow<'r, <DB as sqlx::Database>::Row>,
        ResultStruct: Unpin,
        ResultStruct: Send,
        DB: sqlx::Database<Row = PgRow>,
    {
        match sqlx::query_as::<_, ResultStruct>(&query)
            .fetch_all(&self.connection)
            .await
        {
            Ok(result) => result,
            Err(e) => {
                tracing::error!("Error while running query: {}", e);
                let empty: Vec<ResultStruct> = Vec::new();
                empty
            }
        }
    }

    //tries to find patient/doctor logging in with credentials and gives JWT if successful
    pub async fn login(&self, email: &String, password: &String) -> Option<String> {
        let query = format!(
            "
                    select salt, password as hashedpass, isdoctor from login where email = '{}';
                ",
            email
        );
        match sqlx::query_as::<_, LoginTable>(&query)
            .fetch_one(&self.connection)
            .await
        {
            Ok(result) => {
                let Ok(check) = argon_hash_password::check_password_matches_hash(
                    password,
                    &result.hashedpass,
                    &result.salt,
                ) else {
                    tracing::debug!("Couldn't check password matches hash");
                    return None;
                };
                if check {
                    let mut tablename = "patients";
                    if result.isdoctor {
                        tablename = "doctors";
                    }
                    let query = format!(
                        "
                                    select id from {} where email = '{}';
                                ",
                        tablename, email
                    );
                    let Ok(queryres) = sqlx::query(&query)
                        .fetch_one(&self.connection)
                        .await else {
                            tracing::error!("Error while checking login details in database");
                            return None;
                        };
                    let Ok(id): Result<i64, _> = queryres.try_get("id") else {
                        tracing::error!("Error while retrieving id from query result");
                        return None;
                    };
                    let jwt = InternalJWT {
                        isdoctor: result.isdoctor,
                        id: id.to_string(),
                        exp: 1000000,
                    };
                    let Ok(token) = encode(
                        &Header::default(),
                        &jwt,
                        &EncodingKey::from_secret(&self.jwt_secret),
                    ) else {
                        tracing::debug!("Error while trying to encode JWT");
                        return None;
                    };
                    Some(token)
                } else {
                    None
                }
            }
            Err(_) => {
                tracing::debug!("No such user found!");
                None
            }
        }
    }

    pub fn verify_jwt(&self, jwt: &str) -> Option<JWT> {
        let binding = match String::from(jwt)
            .split("Bearer")
            .collect::<Vec<&str>>()
            .get(1)
        {
            Some(x) => x.to_string(),
            None => jwt.to_string(),
        };
        let mut validation = Validation::default();
        validation.validate_exp = false;
        let token = binding.trim().to_string();
        tracing::debug!("jwt : '{}'", token);
        match decode::<InternalJWT>(
            &token,
            &DecodingKey::from_secret(&self.jwt_secret),
            &validation,
        ) {
            Ok(token) => {
                let Ok(id): Result<i64, _> = token.claims.id.parse() else {
                    tracing::error!("Could not parse id while verifiying JWT");
                    return None;
                };
                let res = JWT {
                    isdoctor: token.claims.isdoctor,
                    id,
                };
                Some(res)
            }
            Err(x) => {
                tracing::debug!("{}", x);
                None
            }
        }
    }

    //get time slots for a doctor
    pub async fn view_doctor_timeslots(&self, doctor_id: i64, date: &String) -> Vec<Timeslots> {
        let query = format!("
                    select TO_CHAR(s.time_start::timestamp, 'HH24:MI:SS') as time_start,
                    (CASE WHEN EXISTS (select 1 from appointments x where x.doctor_id = {} and x.slot_id = s.id and TO_CHAR(x.appointment_date, 'YYYY-MM-DD') = '{}') THEN false
                    ELSE true END) as available, s.id as slot_id
                    from doctor_slots s
                    where s.doctor_id = {}
                            ", doctor_id, date, doctor_id);
        self.get_query_result::<Timeslots, Postgres>(&query)
            .await
    }

    pub async fn view_prescriptions(&self, patient_id: i64) -> Vec<Prescriptions> {
        let query = format!("
                    select d.name as docname, TO_CHAR(p.appointment_date, 'YYYY-MM-DD') as date,
                    p.prescription from Prescriptions p
                    join Doctors d on d.id = p.patient_id
                    where p.patient_id = {}
                    order by p.appointment_date desc;
                    ;", patient_id);
        self.get_query_result::<Prescriptions, Postgres>(&query)
            .await
    }

    pub async fn add_new_prescription(
        &self,
        doctor_id: i32,
        patient_id: i32,
        prescription: &String,
        date: &String,
    ) -> bool {
        let Ok(naivedate) = NaiveDate::parse_from_str(date, "%Y-%m-%d") else {
            tracing::error!("Couldn't parse date into NaiveDateTime");
            return false;
        };
        let query = format!("
                    insert into Prescriptions(patient_id, doctor_id, prescription, appointment_date) values ({}, {}, '{}', '{}');
                            ", patient_id, doctor_id, prescription, naivedate);
        match sqlx::query(&query).execute(&self.connection).await {
            Ok(_) => return true,
            Err(_) => return false,
        }
    }

    pub async fn view_prev_appointments(&self, patient_id: i64) -> Vec<PrevAppointments> {
        let query = format!("
                    select d.name as docname, TO_CHAR(a.appointment_date, 'YYYY-MM-DD') as date, a.type as phyorvirt, a.status as appstatus, a.prescription_id as prescription_id, p.name as appname
                    from appointments a
                    join doctors d on d.id = a.doctor_id
                    join specialities p on p.id = a.appointment_type
                    where a.patient_id = {}
                    order by date desc
                    ;", patient_id);
        self.get_query_result::<PrevAppointments, Postgres>(&query)
            .await
    }

    pub async fn view_same_city_doctors(&self, city: String) -> Vec<DoctorInfo> {
        let query = format!("
                    select d.id as docid, d.name as docname, s.name as specname, d.address as address
                    from doctors d
                    join specialities s on s.id = d.speciality_id
                    where d.city = '{}'
                    ;", city);
        self.get_query_result::<DoctorInfo, Postgres>(&query).await
    }

    pub async fn view_patient_info(&self, patient_id: i64) -> Vec<PatientInfo> {
        let query = format!(
            "
                    select name, email, phone, age, gender, blood_group, weight
                    from patients
                    where id = {}
                    ;",
            patient_id
        );
        self.get_query_result::<PatientInfo, Postgres>(&query).await
    }

    pub async fn view_doctor_prices(&self, city: &String, apptype: &String) -> Vec<DoctorPrices> {
        let iscityspecified = match city.is_empty() {
            false => format!("and d.city = '{}'", city),
            true => String::new(),
        };
        let isapptypespecified = match apptype.is_empty() {
            false => format!("and t.name = '{}'", apptype),
            true => String::new(),
        };

        let query = format!(
            "
                    select d.id as docid, d.name as docname, d.city as city, d.address as address, t.name as apptype, t.id as appid, p.price, spec.name as specname
                    from doctors d
                    join appointment_types t on d.speciality_id = t.speciality_id
                    join specialities spec on spec.id = t.speciality_id
                    join appointment_prices p on d.id = p.doctor_id and t.id = p.appointment_type
                    where 1=1 {} {};
                    ",
            isapptypespecified, iscityspecified
        );
        self.get_query_result::<DoctorPrices, Postgres>(&query)
            .await
    }

    pub async fn view_doctor_prices_emergency(&self, city: &String, apptype: &String) -> Vec<DoctorPrices> {
        let iscityspecified = match city.is_empty() {
            false => format!("and d.city = '{}'", city),
            true => String::new(),
        };
        let isapptypespecified = match apptype.is_empty() {
            false => format!("and t.name = '{}'", apptype),
            true => String::new(),
        };

        let query = format!(
            "
                    select d.id as docid, d.name as docname, d.city as city, d.address as address, t.name as apptype, t.id as appid, 2*p.price as price, spec.name as specname
                    from doctors d
                    join appointment_types t on d.speciality_id = t.speciality_id
                    join specialities spec on spec.id = t.speciality_id
                    join doctors_emergency e on e.doctor_id = d.id
                    join appointment_prices p on d.id = p.doctor_id and t.id = p.appointment_type
                    where 1=1 {} {};
                    ",
            isapptypespecified, iscityspecified
        );
        self.get_query_result::<DoctorPrices, Postgres>(&query)
            .await
    }

    pub async fn view_specialities(&self) -> Vec<Specialities> {
        let query = String::from(
            "select id, name, description as desc
                    from specialities;",
        );
        self.get_query_result::<Specialities, Postgres>(&query)
            .await
    }

    pub async fn view_appointment_types(&self) -> Vec<Apptypes> {
        let query = String::from(
            "select id, name from appointment_types;"
        );
        self.get_query_result::<Apptypes, Postgres>(&query)
            .await
    }

    pub async fn view_cities(&self) -> Vec<Cities> {
        let query = String::from(
            "select distinct(city) as city from doctors"
        );
        self.get_query_result::<Cities, Postgres>(&query)
            .await
    }

    pub async fn view_new_token(&self, doctor_id: i64, date: &String) -> TokenNumberPrimary {
        let query = format!("select count(*) as num from tokens where doctor_id = {} and TO_CHAR(appointment_date, 'YYYY-MM-DD') = '{}'", doctor_id, date);
        match sqlx::query_as::<_, TokenNumberPrimary>(&query)
            .fetch_one(&self.connection)
            .await {
                Ok(mut tn) => {
                    tn.num += 1;
                    tn
                }
                Err(_) => {
                    TokenNumberPrimary {
                        num: 1
                    }
                }
            }
    }

    pub async fn view_current_token(&self,doctor_id: i64, date: &String) -> TokenNumberPrimary {
        let query = format!("select token_number as num from tokens where doctor_id = {} and TO_CHAR(appointment_date, 'YYYY-MM-DD') = '{}' and status = 'ongoing'", doctor_id, date);
        match sqlx::query_as::<_, TokenNumberPrimary>(&query)
            .fetch_one(&self.connection)
            .await {
                Ok(tn) => tn,
                Err(_) => {
                    TokenNumberPrimary {
                        num: 0
                    }
                }
            }
    }

    pub async fn get_patient_token(&self, doctor_id: i64, patient_id: i64, date: &String) -> TokenNumber {
        let query = format!("select token_number as num from tokens where doctor_id = {} and TO_CHAR(appointment_date, 'YYYY-MM-DD') = '{}' and patient_id = {}", doctor_id, date, patient_id);
        match sqlx::query_as::<_, TokenNumber>(&query)
            .fetch_one(&self.connection)
            .await {
                Ok(tn) => tn,
                Err(_) => {
                    TokenNumber {
                        num: 0
                    }
                }
            }
    }


    pub async fn view_doctor_appointments(&self, doctor_id: i64) -> Vec<DoctorAppointments> {
        let query = format!(
            "
            select id, patient_id, appointment_type as apptype,
            TO_CHAR(appointment_date, 'YYYY-MM-DD') as date,
            type as phyorvirt, status, slot_id, symptom from appointments where doctor_id = {} order by date
            ", doctor_id
        );
        self.get_query_result::<DoctorAppointments, Postgres>(&query)
            .await
    }

    pub async fn view_doctor_emergencies(&self, doctor_id: i64, date: &String) -> Vec<EmergencyAppointments> {
        let query = format!(
            "
            select emergency_no as id, patient_id, appointment_type as apptype,
            symptom from emergency_appointments where doctor_id = {}
            and TO_CHAR(appointment_date, 'YYYY-MM-DD') = '{}'
            order by appointment_date
            ", doctor_id, date
        );
        self.get_query_result::<EmergencyAppointments, Postgres>(&query)
            .await
    }

    pub async fn register(&self, email: &String, password: &String, isdoctor: bool) -> bool {
        let Ok((hash, salt)) = argon_hash_password::create_hash_and_salt(&password) else {
            tracing::error!("Hash and salt were not able to be created, registration error");
            return false;
        };
        let query = format!(
            "
                    insert into login(email, password, salt, isdoctor) values ('{}', '{}', '{}', {})
                            ",
            email, hash, salt, isdoctor
        );
        match sqlx::query(&query).execute(&self.connection).await {
            Ok(_) => return true,
            Err(_) => return false,
        }
    }

    pub async fn add_new_patient(&self, name: &String, email: &String, phone: &String) -> bool {
        let query = format!(
            "
                    insert into patients(name, email, phone) values ('{}','{}','{}');
                            ",
            name, email, phone
        );
        match sqlx::query(&query).execute(&self.connection).await {
            Ok(_) => return true,
            Err(_) => return false,
        }
    }

    pub async fn update_patient(&self, patient_id: i64, gender: &String, weight: i32, age: i32, blood_group: &String) -> bool {
        let query = format!(
            "
                    update patients set weight = {}, age = {}, blood_group = '{}', gender = '{}' where id = {};
                            ",
            weight, age, blood_group, gender, patient_id
        );
        match sqlx::query(&query).execute(&self.connection).await {
            Ok(_) => return true,
            Err(_) => return false,
        }
    }

    pub async fn add_new_doctor(
        &self,
        name: &String,
        speciality: i64,
        city: &String,
        address: &String,
        email: &String,
        phone: &String,
    ) -> bool {
        let query = format!("
                    insert into doctors(name, speciality_id, city, address, email, phone) values ('{}',{},'{}', '{}', '{}', '{}');
                            ", name, speciality, city, address, email, phone);
        match sqlx::query(&query).execute(&self.connection).await {
            Ok(_) => return true,
            Err(_) => return false,
        }
    }

    pub async fn add_new_appointment(
        &self,
        docid: i64,
        patid: i64,
        apptype: i64,
        date: &String,
        slot_id: i64,
        phyorvirt: &String,
        symptom: &String,
    ) -> bool {
        let Ok(naivedate) = NaiveDate::parse_from_str(date, "%Y-%m-%d") else {
            tracing::error!("Couldn't parse date into NaiveDateTime");
            return false;
        };
        //check if no appointment has been booked at same time
        let doctorapps = self.view_doctor_appointments(docid).await;
        for app in doctorapps.iter() {
            let Ok(appdate) = NaiveDate::parse_from_str(&app.date, "%Y-%m-%d") else {
                tracing::error!("Couldn't parse date into NaiveDateTime");
                return false;
            };
            if app.slot_id as i64 == slot_id && appdate == naivedate && app.status != "cancelled" {
                tracing::error!("Appointment has already been booked");
                return false;
            }
        }
        let query = format!("
                    INSERT INTO Appointments (doctor_id, patient_id, appointment_type, appointment_date, slot_id, type, status, symptom) VALUES ({}, {}, {}, '{}', {}, '{}', 'scheduled', '{}')
                            ", docid, patid, apptype, naivedate, slot_id, phyorvirt, symptom);
        match sqlx::query(&query).execute(&self.connection).await {
            Ok(_) => return true,
            Err(_) => return false,
        }
    }

    pub async fn add_new_token(
        &self,
        docid: i64,
        patid: i64,
        apptype: i64,
        date: &String,
        symptom: &String,
    ) -> bool {
        let Ok(naivedate) = NaiveDate::parse_from_str(date, "%Y-%m-%d") else {
            tracing::error!("Couldn't parse date into NaiveDateTime");
            return false;
        };
        let checkquery = format!("select doctor_id, patient_id, appointment_date, appointment_type from tokens where doctor_id = {} and patient_id = {} and TO_CHAR(appointment_date, 'YYYY-MM-DD') = '{}' and appointment_type = {}", docid, patid, date, apptype);
        match sqlx::query(&checkquery).fetch_one(&self.connection).await {
            Ok(_) => {
                tracing::error!("This token already exists! Cancelling");
                return false;
            }
            Err(_) => {
                tracing::debug!("An error occurred, attempting to proceed");
            }
        }
        let token_number = self.view_new_token(docid, date).await.num;
        let query = format!("
                    INSERT INTO Tokens (doctor_id, patient_id, appointment_type, appointment_date, token_number, status, symptom) VALUES ({}, {}, {}, '{}', {}, 'scheduled', '{}')
                            ", docid, patid, apptype, naivedate, token_number, symptom);
        match sqlx::query(&query).execute(&self.connection).await {
            Ok(_) => return true,
            Err(_) => return false,
        }
    }

    pub async fn view_new_emergency_no(&self, doctor_id: i64, date: &String) -> TokenNumberPrimary {
        let query = format!("select count(*) as num from emergency_appointments where doctor_id = {} and TO_CHAR(appointment_date, 'YYYY-MM-DD') = '{}'", doctor_id, date);
        match sqlx::query_as::<_, TokenNumberPrimary>(&query)
            .fetch_one(&self.connection)
            .await {
                Ok(mut tn) => {
                    tn.num += 1;
                    tn
                }
                Err(e) => {
                    tracing::error!("Error, can't find emergencies, setting to 1: {}",e);
                    TokenNumberPrimary {
                        num: 1
                    }
                }
            }
    }

    pub async fn add_new_emergency_app(
        &self,
        docid: i64,
        patid: i64,
        apptype: i64,
        date: &String,
        symptom: &String,
    ) -> bool {
        let Ok(naivedate) = NaiveDate::parse_from_str(date, "%Y-%m-%d") else {
            tracing::error!("Couldn't parse date into NaiveDateTime");
            return false;
        };
        let checkquery = format!("select doctor_id, patient_id, appointment_date, appointment_type from emergency_appointments where doctor_id = {} and patient_id = {} and TO_CHAR(appointment_date, 'YYYY-MM-DD') = '{}' and appointment_type = {}", docid, patid, date, apptype);
        match sqlx::query(&checkquery).fetch_one(&self.connection).await {
            Ok(_) => {
                tracing::error!("This emergency already exists! Cancelling");
                return false;
            }
            Err(_) => {
                tracing::debug!("An error occurred, attempting to proceed");
            }
        }
        let emergency_no = self.view_new_emergency_no(docid, date).await.num;
        let query = format!("
                    INSERT INTO emergency_appointments (doctor_id, patient_id, appointment_type, appointment_date, emergency_no, symptom) VALUES ({}, {}, {}, '{}', {}, '{}')
                            ", docid, patid, apptype, naivedate, emergency_no, symptom);
        match sqlx::query(&query).execute(&self.connection).await {
            Ok(_) => return true,
            Err(_) => return false,
        }
    }

    pub async fn cancel_appointment(&self, docid: i64, patid: i64, date: &String) -> bool {
        let query = format!("
                    update appointments set status = 'cancelled' where doctor_id = {} and patient_id = {} and TO_CHAR(appointment_date, 'YYYY-MM-DD') = '{}';
                            ", docid, patid, date);
        match sqlx::query(&query).execute(&self.connection).await {
            Ok(_) => true,
            Err(_) => false,
        }
    }

}
