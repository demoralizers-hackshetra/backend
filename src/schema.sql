-- - set timezone
SET TIMEZONE TO 'Asia/Kolkata';

-- - info about various specialities
CREATE TABLE IF NOT EXISTS Specialities (
    id BIGSERIAL PRIMARY KEY ,
    name VARCHAR(255) NOT NULL,
    description TEXT
);

-- - info about doctors registered in system
CREATE TABLE IF NOT EXISTS Doctors (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    speciality_id INT NOT NULL,
    city VARCHAR(255) NOT NULL,
    address VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL UNIQUE,
    phone VARCHAR(255) NOT NULL,
    FOREIGN KEY (speciality_id) REFERENCES Specialities(id)
);

-- - doctor and emergency stuff, this is mostly beta rn
CREATE TABLE IF NOT EXISTS Doctors_Emergency (
    id BIGSERIAL PRIMARY KEY,
    doctor_id INT NOT NULL,
    available BOOLEAN NOT NULL
);

-- - generic appointment types stored here with some info about them and
-- - restricted to specialities; doesn't make sense for pediatrician to provide
-- - dental services for example
CREATE TABLE IF NOT EXISTS Appointment_Types (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    speciality_id INT NOT NULL,
    description TEXT,
    FOREIGN KEY (speciality_id) REFERENCES Specialities(id)
);

-- - store prices for each appointment type as set by a doctor
CREATE TABLE IF NOT EXISTS Appointment_Prices (
    doctor_id INT NOT NULL,
    appointment_type INT NOT NULL,
    price INT NOT NULL,
    FOREIGN KEY (doctor_id) REFERENCES Doctors(id),
    FOREIGN KEY (appointment_type) REFERENCES Appointment_Types(id),
    PRIMARY KEY (doctor_id, appointment_type)
);

-- - info about patients
CREATE TABLE IF NOT EXISTS Patients (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL UNIQUE,
    phone VARCHAR(255) NOT NULL,
    gender CHAR(1),
    weight INT,
    age INT,
    blood_group VARCHAR(255)
);

CREATE TABLE IF NOT EXISTS Prescriptions (
    id BIGSERIAL PRIMARY KEY,
    patient_id INT NOT NULL,
    doctor_id INT NOT NULL,
    prescription TEXT,
    appointment_date TIMESTAMP NOT NULL,
    FOREIGN KEY (patient_id) REFERENCES Patients(id),
    FOREIGN KEY (doctor_id) REFERENCES Doctors(id)
);

-- - stores slots the doctor sets
CREATE TABLE IF NOT EXISTS Doctor_Slots (
    id BIGSERIAL PRIMARY KEY,
    doctor_id INT NOT NULL,
    time_start TIMESTAMPTZ NOT NULL
);

-- - help doctors keep track of their appointments with patients
CREATE TABLE IF NOT EXISTS Appointments (
    id BIGSERIAL PRIMARY KEY,
    doctor_id INT NOT NULL,
    patient_id INT NOT NULL,
    appointment_type INT NOT NULL,
    appointment_date TIMESTAMP NOT NULL,
    slot_id INT NOT NULL,
    status VARCHAR(255) NOT NULL,
    symptom VARCHAR(255) NOT NULL,
    prescription_id INT,
    type VARCHAR(255) NOT NULL,
    FOREIGN KEY (doctor_id) REFERENCES Doctors(id),
    FOREIGN KEY (patient_id) REFERENCES Patients(id),
    FOREIGN KEY (appointment_type) REFERENCES Appointment_Types(id),
    FOREIGN KEY (slot_id) REFERENCES Doctor_Slots(id),
    FOREIGN KEY (prescription_id) REFERENCES Prescriptions(id),
    CONSTRAINT chk_status CHECK (status IN ('scheduled', 'fulfilled', 'cancelled', 'ongoing')),
    CONSTRAINT chk_type CHECK (type IN ('physical', 'virtual'))
);

-- - help keep track of walk in token based patients
CREATE TABLE IF NOT EXISTS Tokens (
    id BIGSERIAL PRIMARY KEY ,
    doctor_id INT NOT NULL,
    patient_id INT NOT NULL,
    appointment_type INT NOT NULL,
    appointment_date TIMESTAMP NOT NULL,
    token_number INT NOT NULL,
    status VARCHAR(255) NOT NULL,
    prescription_id INT,
    symptom VARCHAR(255) NOT NULL,
    FOREIGN KEY (doctor_id) REFERENCES Doctors(id),
    FOREIGN KEY (patient_id) REFERENCES Patients(id),
    FOREIGN KEY (appointment_type) REFERENCES Appointment_Types(id),
    FOREIGN KEY (prescription_id) REFERENCES Prescriptions(id),
    CONSTRAINT chk_status CHECK (status IN ('scheduled', 'fulfilled', 'cancelled', 'ongoing'))
);

ALTER TABLE Tokens ADD CONSTRAINT unique_token_per_day_doctor UNIQUE (doctor_id, token_number, appointment_date);

-- - help keep track of emergency appointments
CREATE TABLE IF NOT EXISTS Emergency_Appointments (
    id BIGSERIAL PRIMARY KEY,
    doctor_id INT NOT NULL,
    patient_id INT NOT NULL,
    appointment_type INT NOT NULL,
    appointment_date TIMESTAMP NOT NULL,
    emergency_no INT NOT NULL,
    prescription_id INT,
    symptom VARCHAR(255) NOT NULL,
    FOREIGN KEY (doctor_id) REFERENCES Doctors(id),
    FOREIGN KEY (patient_id) REFERENCES Patients(id),
    FOREIGN KEY (appointment_type) REFERENCES Appointment_Types(id),
    FOREIGN KEY (prescription_id) REFERENCES Prescriptions(id)
);

ALTER TABLE Emergency_Appointments ADD CONSTRAINT unique_emergency_per_day_doctor UNIQUE (doctor_id, emergency_no, appointment_date);

-- - keep track of notifications to deliver
CREATE TABLE IF NOT EXISTS Notifications (
    id BIGSERIAL PRIMARY KEY ,
    patient_id INT NOT NULL,
    message TEXT NOT NULL,
    date_time TIMESTAMP NOT NULL,
    FOREIGN KEY (patient_id) REFERENCES Patients(id)
);

-- - keep login info here
CREATE TABLE IF NOT EXISTS Login (
    id BIGSERIAL PRIMARY KEY,
    email VARCHAR(255) NOT NULL UNIQUE,
    password VARCHAR(255) NOT NULL,
    isdoctor BOOLEAN,
    SALT VARCHAR(255) NOT NULL UNIQUE
);
