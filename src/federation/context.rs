use url::Url;

#[derive(Clone)]
pub struct Context {
    pub db_pool: sqlx::PgPool,
    pub base_url: Url,
}

pub type Data = activitypub_federation::config::Data<Context>;
