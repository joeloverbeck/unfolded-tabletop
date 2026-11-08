use ash::vk;

pub(super) fn host_upload_memory_types(
    vendor_id: u32,
    device_id: u32,
    device_type: vk::PhysicalDeviceType,
    memory_types: &[vk::MemoryType],
) -> u32 {
    // Measured on this Linux RTX 2080 Ti: CPU writes into successful BAR mappings
    // can take tens of milliseconds per small staging buffer. Keep the workaround
    // local to the observed adapter; see ../../ARKHAM_PATCH.md for evidence/removal.
    if !cfg!(target_os = "linux")
        || vendor_id != 0x10de
        || device_id != 0x1e07
        || device_type != vk::PhysicalDeviceType::DISCRETE_GPU
    {
        return 0;
    }

    let required = vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT;
    memory_types
        .iter()
        .enumerate()
        .filter(|(_, memory_type)| {
            memory_type.property_flags.contains(required)
                && !memory_type
                    .property_flags
                    .contains(vk::MemoryPropertyFlags::DEVICE_LOCAL)
        })
        .fold(0, |mask, (index, _)| mask | (1 << index))
}

pub(super) fn buffer_memory_type_bits(
    allowed_types: u32,
    host_upload_types: u32,
    usage: wgt::BufferUses,
    memory_flags: crate::MemoryFlags,
) -> u32 {
    // This exact descriptor is wgpu-core's internal StagingBuffer. Preserve
    // explicit user buffers, readback, GPU-only buffers, and no-match behavior.
    if memory_flags.contains(crate::MemoryFlags::TRANSIENT)
        && usage == wgt::BufferUses::MAP_WRITE | wgt::BufferUses::COPY_SRC
    {
        let compatible_host_types = allowed_types & host_upload_types;
        if compatible_host_types != 0 {
            return compatible_host_types;
        }
    }
    allowed_types
}

#[cfg(test)]
mod tests {
    use super::*;

    // Captured RTX 2080 Ti / NVIDIA 615.78.08 Vulkan memory-type table.
    // Types 3 and 4 are coherent host memory on heap 1. Type 5 is also
    // DEVICE_LOCAL on heap 2 and must not serve transient upload staging.
    fn observed_memory_types() -> [vk::MemoryType; 6] {
        [(0x0, 1), (0x1, 0), (0x1, 0), (0x6, 1), (0xe, 1), (0x7, 2)].map(|(flags, heap_index)| {
            vk::MemoryType {
                property_flags: vk::MemoryPropertyFlags::from_raw(flags),
                heap_index,
            }
        })
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn observed_card_uploads_use_the_two_system_memory_types() {
        let host_types = host_upload_memory_types(
            0x10de,
            0x1e07,
            vk::PhysicalDeviceType::DISCRETE_GPU,
            &observed_memory_types(),
        );
        assert_eq!(host_types, 0x18);
        assert_eq!(
            buffer_memory_type_bits(
                0x3f,
                host_types,
                wgt::BufferUses::MAP_WRITE | wgt::BufferUses::COPY_SRC,
                crate::MemoryFlags::TRANSIENT,
            ),
            0x18,
        );
    }

    #[test]
    #[cfg(not(target_os = "linux"))]
    fn observed_identity_does_not_activate_the_workaround_off_linux() {
        assert_eq!(
            host_upload_memory_types(
                0x10de,
                0x1e07,
                vk::PhysicalDeviceType::DISCRETE_GPU,
                &observed_memory_types(),
            ),
            0,
        );
    }

    #[test]
    fn other_vendors_models_and_device_types_keep_the_original_policy() {
        for (vendor, model, device_type) in [
            (0x1002, 0x1e07, vk::PhysicalDeviceType::DISCRETE_GPU),
            (0x10de, 0x2206, vk::PhysicalDeviceType::DISCRETE_GPU),
            (0x10de, 0x1e07, vk::PhysicalDeviceType::INTEGRATED_GPU),
            (0x10de, 0x1e07, vk::PhysicalDeviceType::VIRTUAL_GPU),
            (0x10de, 0x1e07, vk::PhysicalDeviceType::CPU),
        ] {
            assert_eq!(
                host_upload_memory_types(vendor, model, device_type, &observed_memory_types()),
                0,
            );
        }
    }

    #[test]
    fn allocation_requirements_can_select_only_one_of_the_host_types() {
        assert_eq!(
            buffer_memory_type_bits(
                0x28, // Only type 3 and BAR type 5 are allowed by the buffer.
                0x18,
                wgt::BufferUses::MAP_WRITE | wgt::BufferUses::COPY_SRC,
                crate::MemoryFlags::TRANSIENT,
            ),
            0x08,
        );
    }

    #[test]
    fn no_compatible_host_type_keeps_original_requirements_including_empty() {
        for original in [0, 0x20, 0x07] {
            assert_eq!(
                buffer_memory_type_bits(
                    original,
                    0x18,
                    wgt::BufferUses::MAP_WRITE | wgt::BufferUses::COPY_SRC,
                    crate::MemoryFlags::TRANSIENT,
                ),
                original,
            );
        }
    }

    #[test]
    fn an_unaffected_or_invalidated_host_mask_does_not_change_requirements() {
        // Adapter opening intersects the host mask with valid_ash_memory_types.
        // Here none of the host bits are valid. Do not reintroduce them.
        assert_eq!(
            buffer_memory_type_bits(
                0x3f,
                0,
                wgt::BufferUses::MAP_WRITE | wgt::BufferUses::COPY_SRC,
                crate::MemoryFlags::TRANSIENT,
            ),
            0x3f,
        );
    }

    #[test]
    fn explicit_user_uploads_readback_and_gpu_buffers_are_unchanged() {
        for (usage, flags) in [
            (
                wgt::BufferUses::MAP_WRITE | wgt::BufferUses::COPY_SRC,
                crate::MemoryFlags::empty(),
            ),
            (
                wgt::BufferUses::MAP_READ | wgt::BufferUses::COPY_DST,
                crate::MemoryFlags::TRANSIENT,
            ),
            (
                wgt::BufferUses::MAP_READ | wgt::BufferUses::MAP_WRITE | wgt::BufferUses::COPY_SRC,
                crate::MemoryFlags::TRANSIENT,
            ),
            (wgt::BufferUses::MAP_WRITE, crate::MemoryFlags::TRANSIENT),
            (
                wgt::BufferUses::COPY_DST | wgt::BufferUses::VERTEX,
                crate::MemoryFlags::TRANSIENT,
            ),
            (
                wgt::BufferUses::MAP_WRITE | wgt::BufferUses::COPY_SRC | wgt::BufferUses::COPY_DST,
                crate::MemoryFlags::TRANSIENT,
            ),
        ] {
            assert_eq!(buffer_memory_type_bits(0x3f, 0x18, usage, flags), 0x3f);
        }
    }

    #[test]
    fn missing_host_visibility_or_coherence_is_not_an_eligible_alternative() {
        let types = [0x0, 0x1, 0x2, 0x4, 0x7, 0xf].map(|flags| vk::MemoryType {
            property_flags: vk::MemoryPropertyFlags::from_raw(flags),
            heap_index: 0,
        });
        assert_eq!(
            host_upload_memory_types(0x10de, 0x1e07, vk::PhysicalDeviceType::DISCRETE_GPU, &types),
            0,
        );
    }
}
