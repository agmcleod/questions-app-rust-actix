use actix_http::Request;
use actix_service::Service;
use actix_web::{body::Body, dev::ServiceResponse, error::Error, test, App};
use serde::de::DeserializeOwned;

use crate::routes::routes;

pub async fn get_service(
) -> impl Service<Request = Request, Response = ServiceResponse<Body>, Error = Error> {
    test::init_service(App::new().data(db::new_pool()).configure(routes)).await
}

pub async fn test_get<R>(route: &str) -> (u16, R)
where
    R: DeserializeOwned,
{
    let mut app = get_service().await;
    let req = test::TestRequest::get().uri(route);
    let res = test::call_service(&mut app, req.to_request()).await;

    let status = res.status().as_u16();
    let body = test::read_body(res).await;
    let json_body = serde_json::from_slice(&body).unwrap_or_else(|_| {
        panic!(
            "read_response_json failed during deserialization. response: {} status: {}",
            String::from_utf8(body.to_vec())
                .unwrap_or_else(|_| "Could not convert Bytes -> String".to_string()),
            status
        )
    });

    (status, json_body)
}
