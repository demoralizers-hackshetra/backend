use axum::{
    extract::Query,
    http::{
        header::{HeaderMap, AUTHORIZATION},
        Method, StatusCode,
    },
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use db_structs::*;
use std::net::SocketAddr;
use tokio;
use tower_http::cors::{Any, CorsLayer};
use tracing;
use tracing_subscriber;
use sqlx::postgres;

mod database;
mod db_structs;

async fn authenticate(
    conn: &database::Database,
    headers: HeaderMap,
    given_id: &i64,
    isdoctor: bool,
) -> bool {
    let Some(entry) = headers.get(AUTHORIZATION) else {
        tracing::error!("No JWT given in request, denying access..");
        return false;
    };
    let Ok(rawjwt) = entry.to_str() else {
        tracing::error!("JWT can't be parsed, denying access..");
        return false;
    };
    match conn.verify_jwt(rawjwt) {
        Some(jwt) => {
            tracing::debug!("Verified and parsed JWT");
            if *given_id == jwt.id && isdoctor == jwt.isdoctor {
                tracing::debug!("Correct JWT is given!");
                return true;
            } else {
                tracing::error!("Incorrect JWT!");
                return false;
            }
        }
        None => {
            tracing::debug!("Could not verify JWT!");
            false
        }
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_headers(Any)
        .expose_headers(Any)
        .allow_methods([Method::GET, Method::POST]);
    let app = Router::new()
        .route("/", get(root))
        .route("/prevapp", post(prevapp))
        .route("/doctorappointments", post(doctorappointments))
        .route("/emergency/appointments", post(emergency_appointments))
        .route("/doctors", post(doctors))
        .route("/doctor/timeslots", post(doctor_timeslots))
        .route("/doctor/newtoken", post(doctor_newtoken))
        .route("/doctor/curtoken", post(doctor_curtoken))
        .route("/patient", post(patient))
        .route("/patient/update", post(patient_update))
        .route("/emergency/find", get(emergency_find))
        .route("/find", get(find))
        .route("/login", post(login))
        .route("/newpatient", post(newpatient))
        .route("/newdoctor", post(newdoctor))
        .route("/newappointment", post(newappointment))
        .route("/newtoken", post(newtoken))
        .route("/newemergency", post(newemergency))
        .route("/patient/token", post(patient_token))
        .route("/cancelappointment", post(cancelappointment))
        .route("/specialities", get(specialities))
        .route("/cities", get(cities))
        .route("/apptypes", get(apptypes))
        .route("/newprescription", post(newprescription))
        .route("/prescriptions", post(prescriptions))
        .layer(cors);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn root() -> &'static str {
    "Hello world"
}

async fn newprescription(headers: HeaderMap, Json(payload): Json<PrescriptionInfoInput>) -> Response {
    tracing::debug!(
        "Got request to add new prescription for patient ID {} and doctor ID {}",
        payload.patient_id, payload.doctor_id
    );
    let mut code = StatusCode::OK;
    let res = match database::init().await {
        Some(conn) => {
            if authenticate(&conn, headers, &(payload.patient_id as i64), true).await {
                match conn.add_new_prescription(payload.doctor_id, payload.patient_id, &payload.prescription, &payload.date).await {
                    true => "Inserted",
                    false => {
                        code = StatusCode::BAD_REQUEST;
                        "Error while inserting"
                    },
                }
            } else {
                code = StatusCode::UNAUTHORIZED;
                "Error while inserting"
            }
        }
        None => {
            code = StatusCode::INTERNAL_SERVER_ERROR;
            "Error while inserting"
        }
    };
    (code, Json(res)).into_response()
}

async fn prescriptions(headers: HeaderMap, Json(payload): Json<PatientID>) -> Response {
    tracing::debug!(
        "Got request to view previous appointments for patient ID {}",
        payload.patient_id
    );
    let mut code = StatusCode::OK;
    let res = match database::init().await {
        Some(conn) => {
            if authenticate(&conn, headers, &payload.patient_id, false).await {
                let res = conn.view_prescriptions(payload.patient_id).await;
                res
            } else {
                code = StatusCode::UNAUTHORIZED;
                let res: Vec<Prescriptions> = Vec::new();
                res
            }
        }
        None => {
            code = StatusCode::INTERNAL_SERVER_ERROR;
            let res: Vec<Prescriptions> = Vec::new();
            res
        }
    };
    if res.is_empty() && code == StatusCode::OK {
        code = StatusCode::BAD_REQUEST;
    }
    (code, Json(res)).into_response()
}

async fn emergency_appointments(headers: HeaderMap, Json(payload): Json<DoctorDate>) -> Response {
    tracing::debug!(
        "Got request to view emergency appointments for doctor ID {}",
        payload.doctor_id
    );
    let mut code = StatusCode::OK;
    let res = match database::init().await {
        Some(conn) => {
            if authenticate(&conn, headers, &payload.doctor_id, true).await {
                let res = conn.view_doctor_emergencies(payload.doctor_id, &payload.date).await;
                res
            } else {
                code = StatusCode::UNAUTHORIZED;
                let res: Vec<EmergencyAppointments> = Vec::new();
                res
            }
        }
        None => {
            code = StatusCode::INTERNAL_SERVER_ERROR;
            let res: Vec<EmergencyAppointments> = Vec::new();
            res
        }
    };
    if res.is_empty() && code == StatusCode::OK {
        code = StatusCode::BAD_REQUEST;
    }
    (code, Json(res)).into_response()
}



async fn doctorappointments(headers: HeaderMap, Json(payload): Json<PatientID>) -> Response {
    tracing::debug!(
        "Got request to view appointments for doctor ID {}",
        payload.patient_id
    );
    let mut code = StatusCode::OK;
    let res = match database::init().await {
        Some(conn) => {
            if authenticate(&conn, headers, &payload.patient_id, true).await {
                let res = conn.view_doctor_appointments(payload.patient_id).await;
                res
            } else {
                code = StatusCode::UNAUTHORIZED;
                let res: Vec<DoctorAppointments> = Vec::new();
                res
            }
        }
        None => {
            code = StatusCode::INTERNAL_SERVER_ERROR;
            let res: Vec<DoctorAppointments> = Vec::new();
            res
        }
    };
    if res.is_empty() && code == StatusCode::OK {
        code = StatusCode::BAD_REQUEST;
    }
    (code, Json(res)).into_response()
}

async fn prevapp(headers: HeaderMap, Json(payload): Json<PatientID>) -> Response {
    tracing::debug!(
        "Got request to view previous appointments for patient ID {}",
        payload.patient_id
    );
    let mut code = StatusCode::OK;
    let res = match database::init().await {
        Some(conn) => {
            if authenticate(&conn, headers, &payload.patient_id, false).await {
                let res = conn.view_prev_appointments(payload.patient_id).await;
                res
            } else {
                code = StatusCode::UNAUTHORIZED;
                let res: Vec<PrevAppointments> = Vec::new();
                res
            }
        }
        None => {
            code = StatusCode::INTERNAL_SERVER_ERROR;
            let res: Vec<PrevAppointments> = Vec::new();
            res
        }
    };
    if res.is_empty() && code == StatusCode::OK {
        code = StatusCode::BAD_REQUEST;
    }
    (code, Json(res)).into_response()
}

async fn doctors(Json(payload): Json<City>) -> Response {
    tracing::debug!("Got request to view doctors in city {}", payload.city);
    let mut code = StatusCode::OK;
    let res = match database::init().await {
        Some(conn) => conn.view_same_city_doctors(payload.city).await,
        None => {
            code = StatusCode::INTERNAL_SERVER_ERROR;
            let res: Vec<DoctorInfo> = Vec::new();
            res
        }
    };
    if res.is_empty() && code == StatusCode::OK {
        code = StatusCode::BAD_REQUEST;
    }
    (code, Json(res)).into_response()
}


async fn doctor_curtoken(Json(payload): Json<DoctorDate>) -> Response {
    tracing::debug!("Got request to get current ongoing token for doctor ID {}", payload.doctor_id);
    let mut code = StatusCode::OK;
    let res = match database::init().await {
        Some(conn) => conn.view_current_token(payload.doctor_id, &payload.date).await,
        None => {
            code = StatusCode::BAD_REQUEST;
            TokenNumberPrimary {
                num: 0,
            }
        }
    };
    (code, Json(res)).into_response()
}

async fn patient_token(Json(payload): Json<DoctorPatientDate>) -> Response {
    tracing::debug!("Got request to get token booked for patient ID {}", payload.patient_id);
    let mut code = StatusCode::OK;
    let res = match database::init().await {
        Some(conn) => conn.get_patient_token(payload.doctor_id, payload.patient_id, &payload.date).await,
        None => {
            code = StatusCode::BAD_REQUEST;
            TokenNumber {
                num: 0,
            }
        }
    };
    (code, Json(res)).into_response()
}

async fn doctor_newtoken(Json(payload): Json<DoctorDate>) -> Response {
    tracing::debug!("Got request to predict new token for doctor ID {}", payload.doctor_id);
    let mut code = StatusCode::OK;
    let res = match database::init().await {
        Some(conn) => conn.view_new_token(payload.doctor_id, &payload.date).await,
        None => {
            code = StatusCode::BAD_REQUEST;
            TokenNumberPrimary {
                num: 0,
            }
        }
    };
    (code, Json(res)).into_response()
}

async fn doctor_timeslots(Json(payload): Json<DoctorDate>) -> Response {
    tracing::debug!("Got request to view timeslots for doctor ID {}", payload.doctor_id);
    let mut code = StatusCode::OK;
    let res = match database::init().await {
        Some(conn) => conn.view_doctor_timeslots(payload.doctor_id, &payload.date).await,
        None => {
            code = StatusCode::INTERNAL_SERVER_ERROR;
            let res: Vec<Timeslots> = Vec::new();
            res
        }
    };
    if res.is_empty() && code == StatusCode::OK {
        code = StatusCode::BAD_REQUEST;
    }
    (code, Json(res)).into_response()
}

async fn patient(headers: HeaderMap, Json(payload): Json<PatientID>) -> Response {
    tracing::debug!(
        "Got request to view patient info corresponding to patient ID {}",
        payload.patient_id
    );
    let mut code = StatusCode::OK;
    let res = match database::init().await {
        Some(conn) => {
            if authenticate(&conn, headers, &payload.patient_id, false).await {
                conn.view_patient_info(payload.patient_id).await
            } else {
                code = StatusCode::UNAUTHORIZED;
                let res: Vec<PatientInfo> = Vec::new();
                res
            }
        }
        None => {
            code = StatusCode::INTERNAL_SERVER_ERROR;
            let res: Vec<PatientInfo> = Vec::new();
            res
        }
    };
    if res.is_empty() && code == StatusCode::OK {
        code = StatusCode::BAD_REQUEST;
    }
    (code, Json(res)).into_response()
}

async fn patient_update(headers: HeaderMap, Json(payload): Json<PatientInfoInput>) -> Response {
    tracing::debug!(
        "Got request to update patient info corresponding to patient ID {}",
        payload.patient_id
    );
    let mut code = StatusCode::OK;
    let res = match database::init().await {
        Some(conn) => {
            if authenticate(&conn, headers, &payload.patient_id, false).await {
                match conn.update_patient(payload.patient_id, &payload.gender, payload.weight, payload.age, &payload.blood_group).await {
                    true => "Updated",
                    false => {
                        code = StatusCode::BAD_REQUEST;
                        "Error while updating"
                    }
                }
            } else {
                code = StatusCode::UNAUTHORIZED;
                "Error while updating"
            }
        }
        None => {
            code = StatusCode::INTERNAL_SERVER_ERROR;
            "Error while updating"
        }
    };
    (code, Json(res)).into_response()
}


async fn emergency_find(payload: Query<CityApptype>) -> Response {
    tracing::debug!(
        "Got request to view all doctors with appointment type {} in city {} in emergency",
        payload.apptype,
        payload.city
    );
    let mut code = StatusCode::OK;
    let res = match database::init().await {
        Some(conn) => {
            conn.view_doctor_prices_emergency(&payload.city, &payload.apptype)
                .await
        }
        None => {
            code = StatusCode::INTERNAL_SERVER_ERROR;
            let res: Vec<DoctorPrices> = Vec::new();
            res
        }
    };
    if res.is_empty() && code == StatusCode::OK {
        code = StatusCode::BAD_REQUEST;
    }
    (code, Json(res)).into_response()
}

async fn find(payload: Query<CityApptype>) -> Response {
    tracing::debug!(
        "Got request to view all doctors with appointment type {} in city {}",
        payload.apptype,
        payload.city
    );
    let mut code = StatusCode::OK;
    let res = match database::init().await {
        Some(conn) => {
            conn.view_doctor_prices(&payload.city, &payload.apptype)
                .await
        }
        None => {
            code = StatusCode::INTERNAL_SERVER_ERROR;
            let res: Vec<DoctorPrices> = Vec::new();
            res
        }
    };
    if res.is_empty() && code == StatusCode::OK {
        code = StatusCode::BAD_REQUEST;
    }
    (code, Json(res)).into_response()
}

async fn newpatient(Json(payload): Json<Patient>) -> Response {
    tracing::debug!("Got request to insert new patient info");
    match database::init().await {
        Some(conn) => {
            let res = conn
                .add_new_patient(&payload.name, &payload.email, &payload.phone)
                .await
                && conn
                    .register(&payload.email, &payload.password, false)
                    .await;
            if res {
                tracing::debug!("Record inserted successfully");
                return (StatusCode::OK, Json("Inserted")).into_response();
            } else {
                return (StatusCode::BAD_REQUEST, Json("Error while inserting")).into_response();
            }
        }
        None => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json("Error while inserting"),
            )
                .into_response();
        }
    }
}

async fn newdoctor(Json(payload): Json<Doctor>) -> Response {
    tracing::debug!("Got request to insert new doctor info");
    match database::init().await {
        Some(conn) => {
            let mut res = conn.register(&payload.email, &payload.password, true).await;
            if !res {
                tracing::error!("Record could not be inserted successfully");
                return (StatusCode::BAD_REQUEST, Json("Error while inserting")).into_response();
            }
            res = res && conn
                .add_new_doctor(
                    &payload.name,
                    payload.speciality,
                    &payload.city,
                    &payload.address,
                    &payload.email,
                    &payload.phone,
                )
                .await;
            if res {
                tracing::debug!("Record inserted successfully");
                return (StatusCode::OK, Json("Inserted")).into_response();
            } else {
                tracing::error!("Record could not be inserted successfully");
                return (StatusCode::BAD_REQUEST, Json("Error while inserting")).into_response();
            }
        }
        None => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json("Error while inserting"),
            )
                .into_response();
        }
    }
}

async fn newemergency(headers: HeaderMap, Json(payload): Json<Token>) -> Response {
    tracing::debug!("Got request to insert new emergency info");
    match database::init().await {
        Some(conn) => {
            if authenticate(&conn, headers, &payload.patient_id, false).await {
                let res = conn
                    .add_new_emergency_app(
                        payload.doctor_id,
                        payload.patient_id,
                        payload.apptype,
                        &payload.date,
                        &payload.symptom
                    )
                    .await;
                if res {
                    tracing::debug!("Record inserted successfully");
                    return (StatusCode::OK, Json("Inserted")).into_response();
                } else {
                    return (StatusCode::BAD_REQUEST, Json("Error while inserting"))
                        .into_response();
                }
            } else {
                return (StatusCode::UNAUTHORIZED, Json("Error while inserting")).into_response();
            }
        }
        None => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json("Error while inserting"),
            )
                .into_response();
        }
    }
}

async fn newtoken(headers: HeaderMap, Json(payload): Json<Token>) -> Response {
    tracing::debug!("Got request to insert new token info");
    match database::init().await {
        Some(conn) => {
            if authenticate(&conn, headers, &payload.patient_id, false).await {
                let res = conn
                    .add_new_token(
                        payload.doctor_id,
                        payload.patient_id,
                        payload.apptype,
                        &payload.date,
                        &payload.symptom
                    )
                    .await;
                if res {
                    tracing::debug!("Record inserted successfully");
                    return (StatusCode::OK, Json("Inserted")).into_response();
                } else {
                    return (StatusCode::BAD_REQUEST, Json("Error while inserting"))
                        .into_response();
                }
            } else {
                return (StatusCode::UNAUTHORIZED, Json("Error while inserting")).into_response();
            }
        }
        None => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json("Error while inserting"),
            )
                .into_response();
        }
    }
}

async fn newappointment(headers: HeaderMap, Json(payload): Json<Appointment>) -> Response {
    tracing::debug!("Got request to insert new appointment info");
    match database::init().await {
        Some(conn) => {
            if authenticate(&conn, headers, &payload.patient_id, false).await {
                let res = conn
                    .add_new_appointment(
                        payload.doctor_id,
                        payload.patient_id,
                        payload.apptype,
                        &payload.date,
                        payload.slot_id,
                        &payload.phyorvirt,
                        &payload.symptom
                    )
                    .await;
                if res {
                    tracing::debug!("Record inserted successfully");
                    return (StatusCode::OK, Json("Inserted")).into_response();
                } else {
                    return (StatusCode::BAD_REQUEST, Json("Error while inserting"))
                        .into_response();
                }
            } else {
                return (StatusCode::UNAUTHORIZED, Json("Error while inserting")).into_response();
            }
        }
        None => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json("Error while inserting"),
            )
                .into_response();
        }
    }
}

async fn cancelappointment(headers: HeaderMap, Json(payload): Json<CancelAppointment>) -> Response {
    tracing::debug!("Got request to cancel appointment");
    match database::init().await {
        Some(conn) => {
            if authenticate(&conn, headers, &payload.patient_id, false).await {
                let res = conn
                    .cancel_appointment(
                        payload.doctor_id,
                        payload.patient_id,
                        &payload.date,
                    )
                    .await;
                if res {
                    tracing::debug!("Record updated successfully");
                    return (StatusCode::OK, Json("Cancelled")).into_response();
                } else {
                    return (StatusCode::BAD_REQUEST, Json("Error while cancelling"))
                        .into_response();
                }
            } else {
                return (StatusCode::BAD_REQUEST, Json("Error while cancelling")).into_response();
            }
        }
        None => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json("Error while cancelling"),
            )
                .into_response();
        }
    }
}

async fn cities() -> Response {
    tracing::debug!("Got request to fetch cities");
    let mut code = StatusCode::OK;
    let res = match database::init().await {
        Some(conn) => conn.view_cities().await,
        None => {
            code = StatusCode::INTERNAL_SERVER_ERROR;
            let res: Vec<Cities> = Vec::new();
            res
        }
    };
    if res.is_empty() && code == StatusCode::OK {
        code = StatusCode::BAD_REQUEST;
    }
    return (code, Json(res)).into_response();
}

async fn apptypes() -> Response {
    tracing::debug!("Got request to fetch appointment types");
    let mut code = StatusCode::OK;
    let res = match database::init().await {
        Some(conn) => conn.view_appointment_types().await,
        None => {
            code = StatusCode::INTERNAL_SERVER_ERROR;
            let res: Vec<Apptypes> = Vec::new();
            res
        }
    };
    if res.is_empty() && code == StatusCode::OK {
        code = StatusCode::BAD_REQUEST;
    }
    return (code, Json(res)).into_response();
}

async fn specialities() -> Response {
    tracing::debug!("Got request to fetch specialities");
    let mut code = StatusCode::OK;
    let res = match database::init().await {
        Some(conn) => conn.view_specialities().await,
        None => {
            code = StatusCode::INTERNAL_SERVER_ERROR;
            let res: Vec<Specialities> = Vec::new();
            res
        }
    };
    if res.is_empty() && code == StatusCode::OK {
        code = StatusCode::BAD_REQUEST;
    }
    return (code, Json(res)).into_response();
}

async fn login(Json(payload): Json<Login>) -> Response {
    tracing::debug!("Got request to login");
    match database::init().await {
        Some(conn) => {
            let res = conn.login(&payload.email, &payload.password).await;
            match res {
                Some(jwt) => {
                    tracing::debug!("Generated JWT successfully! {}", jwt);
                    return (StatusCode::OK, Json(jwt)).into_response();
                }
                None => {
                    return (StatusCode::BAD_REQUEST, Json("Error while logging in"))
                        .into_response();
                }
            }
        }
        None => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json("Error while logging in"),
            )
                .into_response();
        }
    }
}
