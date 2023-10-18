SELECT chat_message_id,
    user_id,
    message,
    date_created
FROM chat_messages
WHERE chat_id = "c336acbb-5833-4a89-bba2-1baa0de125d7"
ORDER BY datetime(date_created) DESC
LIMIT 10 OFFSET 10