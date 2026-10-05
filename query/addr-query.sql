SELECT
    r.Z_PK AS contact_id,
    r.ZUNIQUEID AS contact_unique_id,
    r.ZTYPE AS record_type,
    r.ZFIRSTNAME AS first_name,
    r.ZMIDDLENAME AS middle_name,
    r.ZLASTNAME AS last_name,
    r.ZNICKNAME AS nickname,
    r.ZORGANIZATION AS organization,
    r.ZJOBTITLE AS job_title,

    (
        SELECT group_concat(p.ZFULLNUMBER, ' | ')
        FROM ZABCDPHONENUMBER AS p
        WHERE p.ZOWNER = r.Z_PK
    ) AS phone_numbers,

    (
        SELECT group_concat(e.ZADDRESS, ' | ')
        FROM ZABCDEMAILADDRESS AS e
        WHERE e.ZOWNER = r.Z_PK
    ) AS email_addresses

FROM ZABCDRECORD AS r
ORDER BY
    r.ZLASTNAME,
    r.ZFIRSTNAME;