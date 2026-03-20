#[derive(petl::PostgreSQL)]
struct Example {
    timestamp: chrono::DateTime<chrono::Utc>,
}

#[tokio::test]
async fn serialize_datetime() {}
