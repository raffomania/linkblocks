use garde::Validate;
use time::OffsetDateTime;

#[derive(Validate)]
pub struct CreateApUser {
    /// TODO use an Url type for this
    #[garde(length(max = 255), url)]
    pub ap_id: String,
    #[garde(length(max = 50))]
    pub username: String,
    /// TODO use an Url type for this
    #[garde(length(max = 255), url)]
    pub inbox_url: String,
    #[garde(length(max = 10_000))]
    pub public_key: String,
    #[garde(length(max = 10_000))]
    pub private_key: Option<String>,
    #[garde(skip)]
    pub last_refreshed_at: OffsetDateTime,
    #[garde(length(max = 100))]
    pub display_name: Option<String>,
    #[garde(length(max = 1_000))]
    pub bio: Option<String>,
}
