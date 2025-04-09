use std::{pin::Pin, sync::Arc};

use axum::{
    Router,
    extract::{FromRequest, State},
    http::HeaderMap,
    routing::{MethodRouter, get},
};
use bytes::Bytes;
use rand::distr::{Alphanumeric, SampleString};
use wist::WistTunnelState;
pub fn wist<F, S: Clone + 'static + Send + Sync>(
    g: impl Fn(&S) -> &WistTunnelState<F> + Send + Sync + 'static,
) -> MethodRouter<S> {
    wist2::<F, S, Bytes>(g, |_, _| {
        Box::pin(async move { Alphanumeric.sample_string(&mut rand::rng(), 24) })
    })
}
pub fn wist2<F, S: Clone + 'static + Send + Sync, X: FromRequest<S> + Send + 'static>(
    g: impl Fn(&S) -> &WistTunnelState<F> + Send + Sync + 'static,
    s: impl Fn(X, S) -> Pin<Box<dyn Future<Output = String> + Send>> + Send + Sync + 'static,
) -> MethodRouter<S> {
    let g = Arc::new(g);
    let s = Arc::new(s);
    return get(|s2: State<S>, x: X| async move { s(x, s2.0).await }).post(
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
pub fn add_wist2<F, S: Clone + 'static + Send + Sync, X: FromRequest<S> + Send + 'static>(
    g: impl Fn(&S) -> &WistTunnelState<F> + Send + Sync + 'static,
    s: impl Fn(X, S) -> Pin<Box<dyn Future<Output = String> + Send>> + Send + Sync + 'static,
    path: &str,
    r: Router<S>,
) -> Router<S> {
    r.route(&format!("{path}.wist"), wist2(g, s))
}
