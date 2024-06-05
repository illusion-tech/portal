use tracing::Span;
use uuid::Uuid;

use crate::get_config;
// use tracing_honeycomb::{register_dist_tracing_root, TraceId};
// use warp::trace::Info;

pub fn remote_trace(source: &str) -> Span {
    let current = tracing::Span::current();

    // let trace_id = TraceId::new();
    let trace_id = Uuid::new_v4();
    let id = get_config().instance_id.clone();

    // Create a span using tracing macros
    let span = tracing::info_span!(target: "event", parent: &current, "begin span", id = %id, source = %source, req = %trace_id);
    span.in_scope(|| {
        // let _ = register_dist_tracing_root(trace_id, None).map_err(|e| {
        //     eprintln!("register trace root error: {:?}", e);
        // });
    });
    span
}
