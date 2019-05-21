//! Crate that provides helpers and extensions for Tide
//! related to query strings.

#![feature(async_await)]
#![warn(
    nonstandard_style,
    rust_2018_idioms,
    future_incompatible,
    missing_debug_implementations
)]

use http::StatusCode;
use serde::Deserialize;
use tide_core::{error::Error, Context};

/// An extension trait for `Context`, providing query string deserialization.
pub trait ContextExt<'de> {
    fn url_query<T: Deserialize<'de>>(&'de self) -> Result<T, Error>;
}

impl<'de, Data> ContextExt<'de> for Context<Data> {
    fn url_query<T: Deserialize<'de>>(&'de self) -> Result<T, Error> {
        let query = self.uri().query();
        if query.is_none() {
            return Err(Error::from(StatusCode::BAD_REQUEST));
        }
        Ok(serde_urlencoded::from_str(query.unwrap())
            .map_err(|_| Error::from(StatusCode::BAD_REQUEST))?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::executor::block_on;
    use http_service::Body;
    use http_service_mock::make_server;
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct Params {
        msg: String,
    }

    async fn handler(cx: tide::Context<()>) -> Result<String, Error> {
        let p = cx.url_query::<Params>()?;
        Ok(p.msg)
    }

    fn app() -> tide::App<()> {
        let mut app = tide::App::new();
        app.at("/").get(handler);
        app
    }

    #[test]
    fn successfully_deserialize_query() {
        let app = app();
        let mut server = make_server(app.into_http_service()).unwrap();
        let req = http::Request::get("/?msg=Hello")
            .body(Body::empty())
            .unwrap();
        let res = server.simulate(req).unwrap();
        assert_eq!(res.status(), 200);
        let body = block_on(res.into_body().into_vec()).unwrap();
        assert_eq!(&*body, &*b"Hello");
    }

    #[test]
    fn unsuccessfully_deserialize_query() {
        let app = app();
        let mut server = make_server(app.into_http_service()).unwrap();
        let req = http::Request::get("/").body(Body::empty()).unwrap();
        let res = server.simulate(req).unwrap();
        assert_eq!(res.status(), 400);
    }

    #[test]
    fn malformatted_query() {
        let app = app();
        let mut server = make_server(app.into_http_service()).unwrap();
        let req = http::Request::get("/?error=should_fail")
            .body(Body::empty())
            .unwrap();
        let res = server.simulate(req).unwrap();
        assert_eq!(res.status(), 400);
    }
}
