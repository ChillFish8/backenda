use poem::Request;
use poem_openapi::payload::Json;
use poem_openapi::types::ToJSON;
use poem_openapi::{ApiResponse, SecurityScheme};
use poem_openapi::auth::Bearer;


lazy_static!{
    static ref SUPERUSER_KEY: Option<String> = {
      std::env::var("SUPERUSER_KEY").ok()
    };
}

#[derive(SecurityScheme)]
#[oai(type = "bearer")]
pub struct TokenBearer(pub Bearer);

#[derive(SecurityScheme)]
#[oai(type = "bearer", checker = "token_checker")]
pub struct SuperUserBearer(());

async fn token_checker(_: &Request, bearer: Bearer) -> Option<()> {
    if let Some(key) = SUPERUSER_KEY.as_ref() {
        if &bearer.token == key {
            return Some(())
        }
    }

    None
}

#[derive(ApiResponse)]
pub enum JsonResponse<T: Send + Sync + ToJSON> {
    #[oai(status = 200)]
    Ok(Json<T>),

    #[oai(status = 401)]
    Unauthorized,
}
