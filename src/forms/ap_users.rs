use garde::Validate;
use time::OffsetDateTime;
use url::Url;

#[derive(Validate)]
pub struct CreateApUser {
    #[garde(length(max = 255))]
    pub ap_id: Url,
    #[garde(length(max = 50))]
    pub username: String,
    #[garde(length(max = 255))]
    pub inbox_url: Url,
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
