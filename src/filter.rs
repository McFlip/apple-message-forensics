use crate::models::MsgContactTuple;
use crate::processing::marshal_msg_contact_tuple_to_json;
use chrono::{DateTime, Utc};

pub fn filter_date_start(joined: Vec<MsgContactTuple>, start: DateTime<Utc>) -> String {
    filter_to_json(joined.into_iter().filter(|(message, _)| {
        message
            .timestamp
            .as_ref()
            .is_some_and(|timestamp| timestamp >= &start)
    }))
}

pub fn filter_date_end(joined: Vec<MsgContactTuple>, end: DateTime<Utc>) -> String {
    filter_to_json(joined.into_iter().filter(|(message, _)| {
        message
            .timestamp
            .as_ref()
            .is_some_and(|timestamp| timestamp <= &end)
    }))
}

pub fn filter_date_between(
    joined: Vec<MsgContactTuple>,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
) -> String {
    filter_to_json(joined.into_iter().filter(|(message, _)| {
        message
            .timestamp
            .as_ref()
            .is_some_and(|timestamp| timestamp >= &start && timestamp <= &end)
    }))
}

fn filter_to_json(messages: impl Iterator<Item = MsgContactTuple>) -> String {
    let filtered = messages.collect::<Vec<_>>();
    marshal_msg_contact_tuple_to_json(&filtered).to_string()
}

#[cfg(test)]
mod tests {
    use super::{filter_date_between, filter_date_end, filter_date_start};
    use crate::models::{Message, MsgContactTuple};
    use chrono::{DateTime, Utc};

    fn date(value: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(value)
            .expect("valid test date")
            .with_timezone(&Utc)
    }

    fn messages() -> Vec<MsgContactTuple> {
        [
            Some(date("2024-01-01T00:00:00Z")),
            Some(date("2024-01-02T00:00:00Z")),
            Some(date("2024-01-03T00:00:00Z")),
            None,
        ]
        .into_iter()
        .enumerate()
        .map(|(index, timestamp)| {
            (
                Message {
                    timestamp,
                    chat: None,
                    sender: None,
                    message: Some(format!("message {index}")),
                    is_from_me: Some(index % 2 == 0),
                    attachment: None,
                },
                None,
            )
        })
        .collect()
    }

    fn filtered_messages(json: &str) -> Vec<String> {
        serde_json::from_str::<serde_json::Value>(json)
            .expect("filter returns valid JSON")
            .as_array()
            .expect("filter returns a JSON array")
            .iter()
            .map(|message| {
                message["message"]
                    .as_str()
                    .expect("message has message text")
                    .to_owned()
            })
            .collect()
    }

    #[test]
    fn date_start_keeps_messages_at_or_after_start() {
        let actual = filter_date_start(messages(), date("2024-01-02T00:00:00Z"));
        assert_eq!(filtered_messages(&actual), ["message 1", "message 2"]);
    }

    #[test]
    fn date_end_keeps_messages_at_or_before_end() {
        let actual = filter_date_end(messages(), date("2024-01-02T00:00:00Z"));
        assert_eq!(filtered_messages(&actual), ["message 0", "message 1"]);
    }

    #[test]
    fn date_between_includes_both_boundaries() {
        let actual = filter_date_between(
            messages(),
            date("2024-01-01T00:00:00Z"),
            date("2024-01-02T00:00:00Z"),
        );
        assert_eq!(filtered_messages(&actual), ["message 0", "message 1"]);
    }
}
