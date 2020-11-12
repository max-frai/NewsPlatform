use actix_web::{
    dev::{self, Body, ResponseBody},
    http,
    middleware::errhandlers::{ErrorHandlerResponse, ErrorHandlers},
    Error,
};
use actix_web::{get, middleware, post, web, App, HttpResponse, HttpServer, Responder};

pub fn render_500<B>(
    mut res: dev::ServiceResponse<B>,
) -> actix_web::Result<ErrorHandlerResponse<B>> {
    let req = res.request();
    let res = res.map_body(|_, _| ResponseBody::Body(Body::from("test")).into_body());
    Ok(ErrorHandlerResponse::Response(res))
}
