-- e2e fixture: populated AddressBook-v22.abcddb data (schema lives in 0.sql)
-- 12 contacts (10 phone, 2 email) mirroring the chat fixture handles (ROWID 1-12)
-- Contact 1 carries a second phone number and contact 11 a second email to exercise pipe-separated multi-value fields
INSERT INTO ZABCDRECORD (Z_PK, Z_ENT, Z_OPT, ZTYPE, ZNAME, ZFIRSTNAME, ZMIDDLENAME, ZLASTNAME, ZNICKNAME, ZORGANIZATION, ZJOBTITLE, ZUNIQUEID)
VALUES
  (1, 1, 1, 0, 'Aaron Adams', 'Aaron', NULL, 'Adams', NULL, NULL, NULL, 'FIXTURE-CONTACT-001'),
  (2, 2, 1, 0, 'Beth Baker', 'Beth', NULL, 'Baker', NULL, NULL, NULL, 'FIXTURE-CONTACT-002'),
  (3, 3, 1, 0, 'Carl Carter', 'Carl', NULL, 'Carter', NULL, NULL, NULL, 'FIXTURE-CONTACT-003'),
  (4, 4, 1, 0, 'Dana Dane', 'Dana', NULL, 'Dane', NULL, NULL, NULL, 'FIXTURE-CONTACT-004'),
  (5, 5, 1, 0, 'Evan Ford', 'Evan', NULL, 'Ford', NULL, NULL, NULL, 'FIXTURE-CONTACT-005'),
  (6, 6, 1, 0, 'Fay Fisher', 'Fay', NULL, 'Fisher', NULL, NULL, NULL, 'FIXTURE-CONTACT-006'),
  (7, 7, 1, 0, 'Gus Grant', 'Gus', NULL, 'Grant', NULL, NULL, NULL, 'FIXTURE-CONTACT-007'),
  (8, 8, 1, 0, 'Hope Hughes', 'Hope', NULL, 'Hughes', NULL, NULL, NULL, 'FIXTURE-CONTACT-008'),
  (9, 9, 1, 0, 'Ivy Irving', 'Ivy', NULL, 'Irving', NULL, NULL, NULL, 'FIXTURE-CONTACT-009'),
  (10, 10, 1, 0, 'Joan Jones', 'Joan', NULL, 'Jones', NULL, NULL, NULL, 'FIXTURE-CONTACT-010'),
  (11, 11, 1, 0, 'Alice Example', 'Alice', NULL, 'Example', NULL, NULL, NULL, 'FIXTURE-CONTACT-011'),
  (12, 12, 1, 0, 'Bob Baker', 'Bob', NULL, 'Baker', NULL, NULL, NULL, 'FIXTURE-CONTACT-012');
INSERT INTO ZABCDPHONENUMBER (Z_PK, Z_ENT, Z_OPT, ZISPRIMARY, ZORDERINGINDEX, ZOWNER, ZAREACODE, ZCOUNTRYCODE, ZEXTENSION, ZFULLNUMBER, ZLABEL, ZLASTFOURDIGITS, ZLOCALNUMBER, ZUNIQUEID)
VALUES
  (1, 1, 1, 1, 0, 1, '555', '1', NULL, '+15550000001', NULL, '0001', '0000001', 'FIXTURE-PHONE-001'),
  (2, 2, 1, 1, 0, 2, '555', '1', NULL, '+15550000002', NULL, '0002', '0000002', 'FIXTURE-PHONE-002'),
  (3, 3, 1, 1, 0, 3, '555', '1', NULL, '+15550000003', NULL, '0003', '0000003', 'FIXTURE-PHONE-003'),
  (4, 4, 1, 1, 0, 4, '555', '1', NULL, '+15550000004', NULL, '0004', '0000004', 'FIXTURE-PHONE-004'),
  (5, 5, 1, 1, 0, 5, '555', '1', NULL, '+15550000005', NULL, '0005', '0000005', 'FIXTURE-PHONE-005'),
  (6, 6, 1, 1, 0, 6, '555', '1', NULL, '+15550000006', NULL, '0006', '0000006', 'FIXTURE-PHONE-006'),
  (7, 7, 1, 1, 0, 7, '555', '1', NULL, '+15550000007', NULL, '0007', '0000007', 'FIXTURE-PHONE-007'),
  (8, 8, 1, 1, 0, 8, '555', '1', NULL, '+15550000008', NULL, '0008', '0000008', 'FIXTURE-PHONE-008'),
  (9, 9, 1, 1, 0, 9, '555', '1', NULL, '+15550000009', NULL, '0009', '0000009', 'FIXTURE-PHONE-009'),
  (10, 10, 1, 1, 0, 10, '555', '1', NULL, '+15550000010', NULL, '0010', '0000010', 'FIXTURE-PHONE-010'),
  (11, 11, 1, 0, 1, 1, '555', '1', NULL, '+15550000011', NULL, '0011', '0000011', 'FIXTURE-PHONE-011');
INSERT INTO ZABCDEMAILADDRESS (Z_PK, Z_ENT, Z_OPT, ZISPRIMARY, ZORDERINGINDEX, ZOWNER, ZADDRESS, ZADDRESSNORMALIZED, ZLABEL, ZUNIQUEID)
VALUES
  (1, 1, 1, 1, 0, 11, 'alice@example.com', 'alice@example.com', NULL, 'FIXTURE-EMAIL-001'),
  (2, 2, 1, 0, 1, 11, 'alice@alt.example.com', 'alice@alt.example.com', NULL, 'FIXTURE-EMAIL-002'),
  (3, 3, 1, 1, 0, 12, 'bob@example.com', 'bob@example.com', NULL, 'FIXTURE-EMAIL-003');
