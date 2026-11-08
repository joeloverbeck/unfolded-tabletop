# Local Vulkan staging workaround

This directory vendors the published `wgpu-hal` **30.0.1** crate. It retains the upstream MIT and Apache-2.0 licenses and source notices.

- Crates.io package: <https://crates.io/crates/wgpu-hal/30.0.1>
- Pristine `.crate` SHA256: `b6b7fb58561a792bc237628ba0792e332de418fefe145f13b5ed8201e6d52f58`
- Upstream repository revision from `.cargo_vcs_info.json`: `40f4a34ebaf56f9a046231f54125ad046239d3f3`, path `wgpu-hal`.
- `Cargo.toml` is the published normalized manifest; `Cargo.toml.orig` and the upstream package metadata remain available. Upstream checksum metadata describes the pristine package, not the locally modified source.
- Cargo's generated `.cargo-ok` unpack marker is omitted. Repository attributes preserve the upstream source's line endings.
- Local Rust changes are confined to `src/vulkan/adapter.rs`, `device.rs`, `mod.rs`, and the new `upload_memory.rs`. No other upstream executable source is modified.
- The root Cargo patch selects this copy; versions and dependency lists stay unchanged. The directory is excluded from root workspace membership.

## Scope and reason

The workaround applies only on **Linux Vulkan**, for NVIDIA vendor `0x10de`, device `0x1e07` (the observed RTX 2080 Ti), classified as a discrete GPU. Its evidence comes from NVIDIA driver **615.78.08**. It is not version-gated: later drivers retain the conservative placement until a repeated native comparison justifies removal.

Successful device-local, host-visible/coherent mappings produced approximately 23–25 ms CPU copies of 301,600 bytes and approximately 22 ms zeroing of 262,144 bytes. Foreground rendering fell to approximately 11 main updates and 10 completed GPU submissions per second. Selecting non-device-local coherent host memory reduced the corresponding operations to approximately 0.03 ms and restored approximately 60/59 progress. Returning to the original allocation policy reproduced the slowdown. This establishes a measured staging-write bottleneck on the observed host; it does not establish the driver's underlying fault or performance on other hardware.

See the source [investigation](https://github.com/joeloverbeck/arkham-unfolded/blob/83bd1264b7962d844cfa2417d7f50be642575c9a/docs/research/nvidia-mapping-and-startup-health.md) and [issue #55](https://github.com/joeloverbeck/arkham-unfolded/issues/55). The shared extraction preserves the patch; both application roots configure it explicitly.

## Behavior

At device creation, cache the compatible HOST_VISIBLE|HOST_COHERENT memory types that exclude DEVICE_LOCAL, intersected with wgpu's existing valid-type mask. The affected buffer must have `MemoryFlags::TRANSIENT` and exactly `MAP_WRITE | COPY_SRC`, which is the pinned wgpu-core internal `StagingBuffer` descriptor.

Intersect that buffer's Vulkan requirements with the cached host mask. When the intersection is nonempty, use it for both the OOM budget prediction and allocator request. Allocation failure propagates through the existing error handling; the allocator cannot retry a BAR type excluded from its requirements. If no compatible host type exists, preserve upstream selection, with the application rendering-health guard still controlling gameplay admission.

Other adapters, operating systems, backends, explicit user buffers, readback buffers, and GPU-only resources keep their upstream path. One initialization log identifies that the policy is enabled; there are no runtime environment switches, retries, sleeps, cache changes, or diagnostic timing systems.

## Verification and removal

The selector's tests use the captured six-entry Vulkan memory table (flags `0,1,1,6,e,7`) and independently specified expected host mask `0x18`. They cover adapter scope, exact staging usage, requirement intersections, and no-match preservation. The initial test failed against the original-selection baseline before the policy was implemented. These tests establish selection rules, not GPU performance or gameplay acceptance. Native checks must exercise the resulting application binary and ordinary launch path.

From the repository root, run the dependency's focused tests with:

```sh
cargo test --locked --manifest-path vendor/wgpu-hal/Cargo.toml --lib --no-default-features --features vulkan vulkan::upload_memory::tests
```

This standalone upstream package test uses its included lockfile and may need registry access for test dependencies not used by the application.

Keep this patch only while the observed issue requires it. On a dependency or driver update, rebase against pristine upstream source and repeat the default-versus-host-staging native comparison on the affected hardware. Remove the Cargo patch and vendored copy when an upstream solution, or repeated measurements of the original path, removes the need. Do not expand the adapter match without corresponding native evidence.
