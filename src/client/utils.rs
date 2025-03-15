
use crate::client::ClientContext;
use reqwest::blocking::Response;
use reqwest::StatusCode;


pub fn json_to_yaml<T>(str: String) -> Option<String>
where
    T: serde::de::DeserializeOwned + serde::Serialize,
{
    let obj = serde_json::from_str::<T>(str.as_str()).ok()?;
    let yaml = serde_yaml::to_string(&obj).ok()?;
    Some(yaml)
}

#[macro_export]
macro_rules! API {
    ($url : literal) => {
        concat!("http://127.0.0.1:8080/api/v1", $url)
    };
}

pub fn handle_errors(response: reqwest::blocking::Response) -> Result<String, String> {

    match response.status() {
        StatusCode::OK => {
            println!("Success");
            Ok(response.text().unwrap())
        }
        _ => Err(response.text().unwrap()),
    }
}

pub fn post_with_params(url: &str, body: String, ctx: &ClientContext) -> Result<Response, String> {
    let client = reqwest::blocking::Client::new();
    // let ctx = ctx_ref.lock().expect("Mutex");
    let mut post_req = client.post(url);
    if let Some(auth) = &ctx.auth_info {
        post_req = post_req.query(&[("token", auth.token.to_string().as_str())]);
    }
    if let Some(bik) = &ctx.bik {
        post_req = post_req.query(&[("bank", bik.to_string().as_str())]);
    }

    post_req = post_req.body(body);
    post_req.send().map_err(|e| e.to_string())
}

pub fn get_with_params(url: &str, ctx: &ClientContext) -> Result<Response, String> {
    let client = reqwest::blocking::Client::new();
    // let ctx = ctx_ref.lock().expect("Mutex");

    let mut get_req = client.get(url);
    if let Some(auth) = &ctx.auth_info {
        get_req = get_req.query(&[("token", auth.token.to_string().as_str())]);
    }
    if let Some(bik) = &ctx.bik {
        get_req = get_req.query(&[("bank", bik.to_string().as_str())]);
    }

    get_req.send().map_err(|e| e.to_string())
}
