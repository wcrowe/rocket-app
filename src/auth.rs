
use rocket::{
    http::Status,
    request::{FromRequest, Outcome, Request},
};


pub struct BasicAuth {
    pub username: String,
    pub password: String,
}

impl BasicAuth {
    fn from_basic_authentication(header: &str) -> Option<BasicAuth> {
        let split = header.split_whitespace().collect::<Vec<_>>();
        if split.len() != 2 {
            return None;
        }

        if split[0] != "Basic" {
            return None;
        }

        Self::from_base64_encode(split[1])
    }

    fn from_base64_encode(base64_string: &str) -> Option<BasicAuth> {
        let decode = base64::decode(base64_string).ok()?;
        let decode_str = String::from_utf8(decode).ok()?;
        let split = decode_str.split(":").collect::<Vec<_>>();

        if split.len() != 2 {
            return None;
        }

        let (username, password) = (split[0].to_string(), split[1].to_string());

        Some(BasicAuth { username, password })
    }
}

#[rocket::async_trait]
impl<'a, 'r> FromRequest<'a, 'r> for BasicAuth {
    type Error = ();

    async fn from_request(request: &'a Request<'r>) -> Outcome<Self, Self::Error> {
        let auth_header = request.headers().get_one("Authorization");
        if let Some(auth_header) = auth_header {
            if let Some(auth) = Self::from_basic_authentication(auth_header) {
                return Outcome::Success(auth);
            }
        }

        Outcome::Failure((Status::Unauthorized, ()))
    }
}
