INSERT INTO Specialities (name, description) VALUES
('Cardiology', 'Heart and blood vessel diseases'),
('Dermatology', 'Skin diseases'),
('Gastroenterology', 'Digestive system diseases'),
('Pediatrics','Child health and diseases');

INSERT INTO Doctors (name, speciality_id, city, address, email, phone) VALUES
('Dr. Rohan Shah', 1, 'Mumbai', '1234 Main St', 'rohan.shah@example.com', '1234567890'),
('Dr. Priya Patel', 2, 'Delhi', '5678 Park Ave', 'priya.patel@example.com', '0987654321'),
('Dr. Vikram Singh', 3, 'Bangalore', '9101112 Oak St', 'vikram.singh@example.com', '1212121212'),
('Dr. Anjali Gupta', 4, 'Chennai', '131415 Elm St', 'anjali.gupta@example.com', '3434343434');

INSERT INTO Appointment_Types (name, speciality_id, description) VALUES
('Consultation', 1, 'Initial evaluation and diagnosis'),
('Follow-up', 1, 'Monitoring and treatment of ongoing condition'),
('Acne treatment', 2, 'Medications and procedures for acne management'),
('Colonoscopy', 3, 'Examination of the colon with a camera');

INSERT INTO Appointment_Prices (doctor_id, appointment_type, price) VALUES
(1, 1, 1000),
(1, 2, 800),
(2, 3, 500),
(3, 4, 1500);

INSERT INTO Patients (name, email, phone, gender, weight, age, blood_group) VALUES
('Rajesh Gupta', 'rajeshgupta@example.com', '5555555555', 'M', 75, 25, 'O+'),
('Priya Sharma', 'priyasharma@example.com', '6666666666', 'F', 65, 30, 'A-'),
('Suresh Patel', 'sureshpatel@example.com', '7777777777', 'M', 80, 35, 'B+'),
('Kavita Patel', 'kavitapatel@example.com', '8888888888', 'F', 55, 20, 'AB+');

INSERT INTO Prescriptions (patient_id, doctor_id, prescription, appointment_date) VALUES
(1,1,'Take 2 tablets of Paracetamol everyday for 2 days', '2022-02-01 00:00:00'),
(2,1,'Take 1 tablet of Paracetamol everyday for 10 days', '2022-01-01 00:00:00'),
(3,4,'Do Physical Therap', '2021-02-01 00:00:00'),
(4,1,'Take ample rest and avoid stressful situations', '2022-01-21 00:00:00');

INSERT INTO Doctor_Slots (doctor_id, time_start) VALUES
(1, '1111-11-11 09:00:00'),
(1, '1111-11-11 10:00:00'),
(1, '1111-11-11 11:00:00'),
(2, '1111-11-11 09:00:00'),
(2, '1111-11-11 10:00:00'),
(3, '1111-11-11 09:00:00'),
(3, '1111-11-11 10:00:00'),
(4, '1111-11-11 09:00:00'),
(4, '1111-11-11 10:00:00');

INSERT INTO Appointments (doctor_id, patient_id, appointment_type, appointment_date, slot_id, type, status, symptom, prescription_id) VALUES
(1, 1, 1, '2022-02-01 00:00:00', 1, 'physical', 'scheduled', 'Headache', 1),
(2, 2, 2, '2022-02-02 00:00:00', 5, 'virtual', 'scheduled', 'Acne', 2),
(3, 3, 3, '2022-02-03 00:00:00', 7, 'physical', 'scheduled', 'Stomach pain', 3),
(4, 4, 4, '2022-02-04 00:00:00', 9, 'virtual', 'scheduled', 'Chest pain', 4);

INSERT INTO Tokens (doctor_id, patient_id, appointment_type, appointment_date, token_number, symptom, status) VALUES
(1, 1, 1, '2022-02-01 10:00:00', 1, 'Headache way too severe', 'fulfilled'),
(2, 2, 2, '2022-02-02 10:00:00', 2, 'Acne too much', 'ongoing'),
(3, 3, 3, '2022-02-03 10:00:00', 3, 'Stomach pains for few days', 'scheduled'),
(4, 4, 4, '2022-02-04 10:00:00', 4, 'Chest pains for last few hours', 'scheduled');
