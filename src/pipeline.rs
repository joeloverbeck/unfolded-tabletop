//! Readiness of Bevy's render pipelines at the caller's sampling point.
//!
//! The caller chooses when to sample and how to use the answer, including any
//! wait, frame count, GPU completion, or admission policy.

use bevy::render::render_resource::{CachedPipelineState, PipelineCache};

/// Whether Bevy reports an empty waiting list and every cached pipeline has
/// compiled successfully. An empty cache with nothing waiting is ready. A
/// terminal error still prevents readiness after Bevy stops retrying it.
///
/// This reads the waiting list and the entries Bevy has moved into the cache.
/// Bevy stages newly queued pipelines separately until it processes the queue.
pub fn is_ready(cache: &PipelineCache) -> bool {
    cache.waiting_pipelines().next().is_none()
        && cache
            .pipelines()
            .all(|pipeline| matches!(pipeline.state, CachedPipelineState::Ok(_)))
}
