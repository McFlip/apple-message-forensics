SELECT
    strftime('%Y-%m-%dT%H:%M:%SZ', m.date / 1000000000 + 978307200, 'unixepoch') AS timestamp,
    COALESCE(c.display_name, c.chat_identifier) AS chat,
    h.id AS sender,
    m.text AS message,
    a.filename AS attachment
FROM message AS m
JOIN chat_message_join AS cmj
    ON cmj.message_id = m.ROWID
JOIN chat AS c
    ON c.ROWID = cmj.chat_id
LEFT JOIN handle AS h
    ON h.ROWID = m.handle_id
LEFT JOIN message_attachment_join AS maj
    ON maj.message_id = m.ROWID
LEFT JOIN attachment AS a
    ON a.ROWID = maj.attachment_id
ORDER BY
    c.ROWID,
    m.date;