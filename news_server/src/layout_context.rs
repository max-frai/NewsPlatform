use actix_web::{dev::Payload, web::Data, Error, FromRequest, HttpRequest};
use futures::future::{ok, Ready};
use std::ops::{Deref, DerefMut};

pub struct LayoutContext(tera::Context);

impl FromRequest for LayoutContext {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;
    type Config = ();

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        ok(LayoutContext(
            req.extensions().get::<tera::Context>().unwrap().clone(),
        ))
    }
}

impl Deref for LayoutContext {
    type Target = tera::Context;
    fn deref(&self) -> &tera::Context {
        &self.0
    }
}

impl DerefMut for LayoutContext {
    fn deref_mut(&mut self) -> &mut tera::Context {
        &mut self.0
    }
}
