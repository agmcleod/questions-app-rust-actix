pub mod questions;

use actix_web::web;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .service(web::scope("/questions")
                .route("", web::get().to(questions::get_all))
                .route("", web::post().to(questions::create))),
    );
}
