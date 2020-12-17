use actix_web::HttpResponse;

pub fn redirect(new_location: &str) -> HttpResponse {
    HttpResponse::MovedPermanently()
        .header(actix_web::http::header::LOCATION, new_location)
        .finish()
}
