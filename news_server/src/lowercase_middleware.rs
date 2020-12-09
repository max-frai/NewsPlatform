use std::task::{Context, Poll};

use actix_service::{Service, Transform};
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::{http, Error, HttpResponse};
use futures::future::{ok, Either, Ready};

pub struct LowercaseRequest;

impl<S, B> Transform<S> for LowercaseRequest
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = LowercaseRequestMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(LowercaseRequestMiddleware { service })
    }
}
pub struct LowercaseRequestMiddleware<S> {
    service: S,
}

impl<S, B> Service for LowercaseRequestMiddleware<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Either<S::Future, Ready<Result<Self::Response, Self::Error>>>;

    fn poll_ready(&mut self, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, req: ServiceRequest) -> Self::Future {
        for letter in req.path().chars() {
            if letter.is_alphabetic() && !letter.is_ascii_lowercase() {
                let new_path = req.path().to_ascii_lowercase();
                return Either::Right(ok(req.into_response(
                    HttpResponse::MovedPermanently()
                        .header(actix_web::http::header::LOCATION, new_path)
                        .finish()
                        .into_body(),
                )));
            }
        }

        Either::Left(self.service.call(req))
    }
}
