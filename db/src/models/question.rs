use chrono::{DateTime, Utc};
use diesel::{PgConnection, Insertable, QueryDsl, Queryable, RunQueryDsl};
use serde::{Deserialize, Serialize};

use errors::Error;

use crate::schema::questions;

#[derive(Debug, Identifiable, Serialize, Deserialize, Queryable)]
pub struct Question {
    pub id: i32,
    pub body: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Queryable, Serialize, PartialEq)]
pub struct QuestionDetails {
    pub id: i32,
    pub body: String,
}

#[derive(Debug, Insertable)]
#[table_name = "questions"]
pub struct NewQuestion {
    pub body: String,
}

impl Question {
    pub fn get_all(conn: &PgConnection) -> Result<Vec<Question>, Error> {
        use crate::schema::questions::dsl::{body, questions};

        let all_questions = questions.order(body).load::<Question>(conn)?;

        Ok(all_questions)
    }

    pub fn create(conn: &PgConnection, body: &String) {}
}
