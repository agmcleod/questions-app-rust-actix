use actix::Actor;
use actix_http::Request;
use actix_service::Service;
use actix_test;
use actix_web::{body::BoxBody, dev::ServiceResponse, error::Error, test, web, App};
use actix_web_actors::ws;
use serde::{de::DeserializeOwned, Serialize};

use crate::routes::routes;
use crate::websocket::{MessageToClient, Server};

pub async fn get_service(
) -> impl Service<Request, Response = ServiceResponse<BoxBody>, Error = Error> {
    test::init_service(
        App::new()
            .app_data(web::Data::new(db::new_pool()))
            .app_data(web::Data::new(Server::new().start()))
            .configure(routes),
    )
    .await
}

pub fn get_test_server() -> actix_test::TestServer {
    actix_test::start(|| {
        App::new()
            .app_data(web::Data::new(db::new_pool()))
            .app_data(web::Data::new(Server::new().start()))
            .configure(routes)
    })
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

pub async fn test_post<T: Serialize, R>(route: &str, params: T) -> (u16, R)
where
    R: DeserializeOwned,
{
    let mut app = get_service().await;

    let req = test::TestRequest::post().set_json(&params).uri(route);

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

pub fn get_websocket_frame_data(frame: ws::Frame) -> Option<MessageToClient> {
    match frame {
        ws::Frame::Text(t) => {
            let bytes = t.as_ref();
            let data = String::from_utf8(bytes.to_vec()).unwrap();
            let value: MessageToClient = serde_json::from_str(&data).unwrap();
            return Some(value);
        }
        _ => {}
    }

    None
}
