use irdest_sdk::users::UserAuth;
use tide::Next;
use tide::Response;
use crate::Identity;
use crate::State;
use serde::{Deserialize, Serialize};
use tide::{Body, Request, Result, StatusCode, Middleware};

#[derive(Debug, Deserialize)]
struct ReqisterRequest {
    password: String,
}

#[derive(Debug, Serialize)]
struct RegisterResponse {
    id: String,
}

pub(crate) async fn register(mut req: Request<State>) -> Result<Body> {
    let ReqisterRequest { password } = req.body_json().await?;

    let auth = req.state().sdk.users().create(&password).await.unwrap();
    Body::from_json(&RegisterResponse { id: auth.0.to_string() })
}

#[derive(Debug, Deserialize)]
struct LoginRequest {
    id: String,
    password: String,
}

#[derive(Debug, Serialize)]
struct LoginResponse {
    token: String,
}

pub(crate) async fn login(mut req: Request<State>) -> Result<Body> {
    let LoginRequest { id, password } = req.body_json().await?;

    let auth = req.state().sdk.users().login(Identity::from_string(&id), &password).await.unwrap();
    let token = auth.1;
    Body::from_json(&LoginResponse { token })
}

pub(crate) async fn verify_token(req: Request<State>) -> Result<Response> {
    match req.ext::<UserAuth>() {
        None => Ok(Response::new(StatusCode::Unauthorized)),
        Some(_) => Ok(Response::new(StatusCode::Ok)),
    }
}

pub struct LoadUserMiddleware;

#[tide::utils::async_trait]
impl<State: Clone + Send + Sync + 'static> Middleware<State> for LoadUserMiddleware {
    async fn handle(&self, mut req: Request<State>, next: Next<'_, State>) -> Result {
        if let Some(authorization_header) = req.header("Authorization") {
            match authorization_header.to_string().split_whitespace().skip(1).next() {
                None => return Ok(Response::new(400)),
                Some(id_and_token) => {
                    let segments: Vec<&str> = id_and_token.split(":").collect();
                    if segments.len() != 2 {
                        return Ok(Response::new(400));
                    }

                    let user_auth: UserAuth = UserAuth(Identity::from_string(&segments[0].to_string()), segments[1].to_string());
                    req.set_ext(user_auth);
                },
            }
        }
        Ok(next.run(req).await)
    }
}