SELECT
    h.ROWID AS handle_id,
    h.id AS handle_identifier,
    h.service,
    h.country,
    h.uncanonicalized_id,
    h.person_centric_id
FROM handle AS h
ORDER BY h.id;