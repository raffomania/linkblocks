pub fn time_to_chrono(time_date: time::OffsetDateTime) -> chrono::DateTime<chrono::Utc> {
    #[expect(clippy::cast_possible_truncation)]
    chrono::TimeZone::timestamp_nanos(&chrono::Utc, time_date.unix_timestamp_nanos() as i64)
}
