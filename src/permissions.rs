use actix_web::error;
use actix_web::FromRequest;
use actix_web::HttpRequest;
use actix_web::Result;

#[derive(Deserialize, Debug, Clone)]
pub struct Scope {
    pub scope: String,
}

impl<S> FromRequest<S> for Scope {
    type Config = ();
    type Result = Result<Self, error::Error>;

    #[inline]
    fn from_request(req: &HttpRequest<S>, _cfg: &Self::Config) -> Self::Result {
        req.headers()
            .get("scope")
            .and_then(|h| h.to_str().ok())
            .map(|h| Scope {
                scope: h.to_owned(),
            })
            .ok_or_else(|| error::ErrorForbidden("no scope"))
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct UserId {
    pub user_id: String,
}

impl<S> FromRequest<S> for UserId {
    type Config = ();
    type Result = Result<Self, error::Error>;

    #[cfg(not(feature = "nouid"))]
    #[inline]
    fn from_request(req: &HttpRequest<S>, _cfg: &Self::Config) -> Self::Result {
        req.headers()
            .get("x-forwarded-user-subject")
            .and_then(|id| id.to_str().ok())
            .map(|id| UserId {
                user_id: id.to_owned(),
            })
            .ok_or_else(|| error::ErrorForbidden("no user_id"))
    }

    #[cfg(feature = "nouid")]
    #[inline]
    fn from_request(_: &HttpRequest<S>, _cfg: &Self::Config) -> Self::Result {
        use std::env::var;
        let user_id = var("DPF_USER_ID").unwrap();
        Ok(UserId { user_id })
    }
}
