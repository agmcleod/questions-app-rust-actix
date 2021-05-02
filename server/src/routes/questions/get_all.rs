use actix_web::{
    error::Error,
    web::{block, Data, Json},
    Result,
};

use db::{get_conn, models::Question, PgPool};
// use errors::Error;

pub async fn get_all(pool: Data<PgPool>) -> Result<Json<Vec<Question>>, Error> {
    let connection = pool.get().unwrap();

    let questions = block(move || Question::get_all(&connection)).await?;

    Ok(Json(questions))
}
