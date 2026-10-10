/* WARNING! ERRORS ENCOUNTERED DURING SQL PARSING! */
SELECT strftime('%Y-%m-%dT%H:%M:%SZ', m.DATE / 1000000000 + 978307200, 'unixepoch') AS TIMESTAMP
	,	CASE 
    WHEN m.date_edited = 0 
      THEN NULL 
    ELSE strftime('%Y-%m-%dT%H:%M:%SZ', m.date_edited / 1000000000 + 978307200, 'unixepoch') 
  END AS date_edited
	,COALESCE(NULLIF(c.display_name, ''), NULLIF(c.chat_identifier, ''), NULLIF(m.ck_chat_id, ''), '') AS chat
	,CASE 
		WHEN m.handle_id = 0
			THEN 'Custodian'
		ELSE COALESCE(h.id, 'UNKOWN')
		END AS sender
	,m.is_from_me
	,m.TEXT AS message
	,a.filename AS attachment
	,m.is_delivered
	,m.is_sent
	,m.is_read
	,m.is_forward
	FROM message AS m
JOIN chat_message_join AS cmj ON cmj.message_id = m.ROWID
JOIN chat AS c ON c.ROWID = cmj.chat_id
LEFT JOIN handle AS h ON h.ROWID = m.handle_id
LEFT JOIN message_attachment_join AS maj ON maj.message_id = m.ROWID
LEFT JOIN attachment AS a ON a.ROWID = maj.attachment_id
ORDER BY c.ROWID
	,m.DATE;