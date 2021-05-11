use crate::State;
use tide::{Request, Result};

pub(crate) async fn create_user(req: Request<State>) -> Result<String> {
    let id = req.state().sdk.users().create("blörp").await.unwrap();
    Ok(format!("User Id: {:?}", id))
}
