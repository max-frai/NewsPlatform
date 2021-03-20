use std::task::{Context, Poll};

use actix_service::{Service, Transform};
use actix_web::{
    dev::{MessageBody, ServiceRequest, ServiceResponse},
    web::Data,
};

use actix_web::Error;
use futures::future::{ok, Ready};

use crate::state::State;

pub struct CanonicalRequest;

impl<S, B> Transform<S, ServiceRequest> for CanonicalRequest
where
    B: MessageBody,
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
{
    type Response = ServiceResponse<B>;
    type Transform = CanonicalRequestMiddleware<S>;
    type Error = Error;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(CanonicalRequestMiddleware { service })
    }
}
pub struct CanonicalRequestMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for CanonicalRequestMiddleware<S>
where
    B: MessageBody,
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, req: ServiceRequest) -> Self::Future {
        let mut context = tera::Context::new();

        if let Some(state) = req.app_data::<Data<State>>() {
            context.insert("PROJECT_DOMAIN", &state.constants.full_domain);
            context.insert("BUILD_VERSION", &state.build_random_number);
        }

        if !req.query_string().is_empty() {
            let canonical = {
                let info = req.connection_info();
                format!("{}://{}{}", info.scheme(), info.host(), req.path())
            };

            // dbg!(&canonical);
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
            Ok(req) => self.service.call(req),
            Err(_) => panic!("Something went wrong with ServiceRequest construction"),
        }
    }
}
