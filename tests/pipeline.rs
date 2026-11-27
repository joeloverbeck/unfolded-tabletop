//! The readiness query against Bevy's cache and queue.
#![cfg(feature = "render")]

use bevy::{
    asset::Assets,
    render::{
        render_resource::{CachedPipelineState, ComputePipelineDescriptor, PipelineCache},
        renderer::initialize_renderer,
        settings::{Backends, WgpuSettings},
    },
    shader::Shader,
    tasks::block_on,
};
use unfolded_tabletop::pipeline;

#[test]
#[ignore = "requires a Vulkan render device"]
fn readiness_tracks_waiting_and_terminal_failure_in_the_pipeline_cache() {
    let settings = WgpuSettings::default();
    let device = block_on(initialize_renderer(Backends::VULKAN, None, &settings)).0;
    let mut cache = PipelineCache::new(device, true);

    assert!(pipeline::is_ready(&cache), "an empty cache is ready");

    let shaders = Assets::<Shader>::default();
    let shader = shaders.reserve_handle();
    cache.set_shader(
        shader.id(),
        Shader::from_wgsl("@compute @workgroup_size(1) fn main() {}", "ready.wgsl"),
    );
    let compiled = cache.queue_compute_pipeline(ComputePipelineDescriptor {
        shader,
        ..Default::default()
    });

    cache.process_queue();
    assert!(cache.get_compute_pipeline(compiled).is_some());
    assert!(cache.waiting_pipelines().next().is_some());
    assert!(!pipeline::is_ready(&cache), "the queue still has work");

    cache.process_queue();
    assert!(
        pipeline::is_ready(&cache),
        "the queue and pipeline are ready"
    );

    let shader = shaders.reserve_handle();
    cache.set_shader(
        shader.id(),
        Shader::from_wgsl(
            "@compute @workgroup_size(1) fn main() { missing(); }",
            "failed.wgsl",
        ),
    );
    let failed = cache.queue_compute_pipeline(ComputePipelineDescriptor {
        shader,
        ..Default::default()
    });

    cache.process_queue();
    cache.process_queue();
    assert!(cache.waiting_pipelines().next().is_none());
    assert!(matches!(
        cache.get_compute_pipeline_state(failed),
        CachedPipelineState::Err(_)
    ));
    assert!(
        !pipeline::is_ready(&cache),
        "a terminal shader failure still prevents readiness"
    );
}
