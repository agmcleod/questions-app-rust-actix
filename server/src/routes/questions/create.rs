use actix::Addr;
use actix_web::{
    web::{block, Data, Json},
    Result,
};
use serde::{Deserialize, Serialize};
use serde_json::to_value;

use db::{get_conn, models::Question, PgPool};
use errors::Error;

use crate::websocket::{MessageToClient, Server};

#[derive(Clone, Deserialize, Serialize)]
pub struct CreateRequest {
    body: String,
}

pub async fn create(
    pool: Data<PgPool>,
    websocket_srv: Data<Addr<Server>>,
    params: Json<CreateRequest>,
) -> Result<Json<Question>, Error> {
    if params.body == "" {
        return Err(Error::BadRequest("Body is required".to_string()));
    }

    let connection = get_conn(&pool)?;

    let question = block(move || Question::create(&connection, &params.body)).await?;

    if let Ok(question) = to_value(question.clone()) {
        let msg = MessageToClient::new("newquestion", question);
        websocket_srv.do_send(msg);
    }

    Ok(Json(question))
}

#[cfg(test)]
mod tests {
    use actix_web::client::Client;
    use diesel::{self, RunQueryDsl};
    use futures::StreamExt;
    use serde_json;

    use db::{
        get_conn,
        models::{NewQuestion, Question},
        new_pool,
        schema::questions,
    };
    use errors::ErrorResponse;

    use crate::tests;

    #[actix_rt::test]
    pub async fn test_create_question() {
        let pool = new_pool();
        let conn = get_conn(&pool).unwrap();

        let srv = tests::get_test_server();

        let client = Client::default();
        let ws_conn = client.ws(srv.url("/ws/")).connect().await.unwrap();

        let mut res = srv
            .post("/api/questions")
            .send_json(&NewQuestion {
                body: "A new question".to_string(),
            })
            .await
            .unwrap();

        assert_eq!(res.status().as_u16(), 200);

        let question: Question = res.json().await.unwrap();
        assert_eq!(question.body, "A new question");

        let mut stream = ws_conn.1.take(1);
        let msg = stream.next().await;

        let data = tests::get_websocket_frame_data(msg.unwrap().unwrap());
        if data.is_some() {
            let msg = data.unwrap();
            assert_eq!(msg.msg_type, "newquestion");
            let question: Question = serde_json::from_value(msg.data).unwrap();
            assert_eq!(question.body, "A new question");
        } else {
            assert!(false, "Message was not a string");
        }

        drop(stream);

        let result_questions = questions::dsl::questions.load::<Question>(&conn).unwrap();
        assert_eq!(result_questions.len(), 1);
        assert_eq!(result_questions[0].body, "A new question");

        srv.stop().await;

        diesel::delete(questions::dsl::questions)
            .execute(&conn)
            .unwrap();
    }

    #[actix_rt::test]
    pub async fn test_create_body_required() {
        let pool = new_pool();
        let conn = get_conn(&pool).unwrap();

        let res: (u16, ErrorResponse) = tests::test_post(
            "/api/questions",
            NewQuestion {
                body: "".to_string(),
            },
        )
        .await;

        assert_eq!(res.0, 400);
        assert_eq!(res.1.errors, vec!["Body is required"]);

        let result_questions = questions::dsl::questions.load::<Question>(&conn).unwrap();
        assert_eq!(result_questions.len(), 0);
    }
}
