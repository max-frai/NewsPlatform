use std::{
    borrow::BorrowMut,
    task::{Context, Poll},
};

use actix_service::{Service, Transform};
use actix_web::{
    dev::{ServiceRequest, ServiceResponse},
    FromRequest, HttpMessage,
};
use actix_web::{http, Error, HttpResponse};
use futures::future::{ok, Either, Ready};

pub struct CanonicalRequest;

impl<S, B> Transform<S> for CanonicalRequest
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = CanonicalRequestMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(CanonicalRequestMiddleware { service })
    }
}
pub struct CanonicalRequestMiddleware<S> {
    service: S,
}

impl<S, B> Service for CanonicalRequestMiddleware<S>
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

    fn call(&mut self, mut req: ServiceRequest) -> Self::Future {
        let mut context = tera::Context::new();
        if !req.query_string().is_empty() {
            let canonical = {
                let info = req.connection_info();
                format!("{}://{}{}", info.scheme(), info.host(), req.path())
            };

            dbg!(&canonical);
            context.insert(
                "query_string_meta",
                &format!(
                    r#"
                    <meta name="robots" content="noindex" />
                    <link rel="canonical" href="{}" />
                "#,
                    canonical
                ),
            );
        }

        let (httpreq, payload) = req.into_parts();
        {
            let exts = &mut *httpreq.extensions_mut();
            exts.insert(context);
        }

        match ServiceRequest::from_parts(httpreq, payload) {
            Ok(req) => Either::Left(self.service.call(req)),
            Err(_) => panic!("Something went wrong with ServiceRequest construction"),
        }
    }
}
