use bytes::Bytes;
use hyper::{Body, Request};
use hyper::body::to_bytes;
use route_recognizer::Params;
pub type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

#[derive(Clone, Debug)]
pub struct AppState {
    pub state_thing: String,
}

#[derive(Debug)]
pub struct Context {
    pub state: AppState,
    pub req: Request<Body>,
    pub params: Params,
    body_bytes: Option<Bytes>,
}

impl Context {
    pub fn new(state: AppState, req: Request<Body>, params: Params) -> Context {
        Context {
            state,
            req,
            params,
            body_bytes: None,
        }
    }

    pub async fn body_json(&mut self) -> Result<String, Error> {
        let body_bytes = match self.body_bytes {
            Some(ref v) => v,
            _ => {
                let body = to_bytes(self.req.body_mut()).await?;
                self.body_bytes = Some(body);
                self.body_bytes.as_ref().expect("body_bytes wasn't set")
            }
        };
        println!("Body is: \n{}", String::from_utf8(body_bytes.to_vec()).unwrap());
        Ok(String::from_utf8(body_bytes.to_vec()).unwrap())
    }
}
