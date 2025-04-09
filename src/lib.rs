use std::sync::Arc;

use axum::{
    Router,
    extract::State,
    http::HeaderMap,
    routing::{MethodRouter, get},
};
use bytes::Bytes;
use rand::distr::{Alphanumeric, SampleString};
use wist::WistTunnelState;

pub fn wist<F, S: Clone + 'static + Send + Sync>(
    g: impl Fn(&S) -> &WistTunnelState<F> + Send + Sync + 'static,
) -> MethodRouter<S> {
    let g = Arc::new(g);
    return get(|| async move { Alphanumeric.sample_string(&mut rand::rng(), 24) }).post(
        |s: State<S>, h: HeaderMap, body: Bytes| async move {
            let s = g(&s.0);
            let Some(h) = h.get("X-Instance-Id").and_then(|a| a.to_str().ok()) else {
                return Bytes::default();
            };
            let h = s.handler(h.to_owned());
            return h.process(&body, 1048576).await.into();
        },
    );
}
pub fn add_wist<F, S: Clone + 'static + Send + Sync>(
    g: impl Fn(&S) -> &WistTunnelState<F> + Send + Sync + 'static,
    path: &str,
    r: Router<S>,
) -> Router<S> {
    r.route(&format!("{path}.wist"), wist(g))
}
