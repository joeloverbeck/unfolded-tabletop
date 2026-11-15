//! Bounded and verified reads, PNG crop bounds and no-replacement publication,
//! held at the public interface with test-made files. Expectations come from
//! Arkham's recorded atlas fixture and from hand-computed sizes and digests.
use std::{fs, path::Path};
use unfolded_tabletop::assets::{
    PngCrop, crop_png, publish_directory, read_bounded, read_verified, regular_file, relative_path,
    resolve_output, verify_bytes,
};

/// SHA-256 of the three bytes `abc` (FIPS 180-2, appendix B.1).
const ABC_SHA256: &str = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";

/// Arkham's independently specified 4x3 atlas, including transparent colors.
fn atlas() -> Vec<u8> {
    use image::ImageEncoder;
    let pixels = [
        [1, 0, 0, 255],
        [2, 0, 0, 255],
        [3, 0, 0, 255],
        [4, 0, 0, 255],
        [5, 0, 0, 255],
        [6, 20, 30, 0],
        [7, 40, 50, 128],
        [8, 0, 0, 255],
        [9, 0, 0, 255],
        [10, 60, 70, 255],
        [11, 80, 90, 255],
        [12, 0, 0, 255],
    ];
    let mut bytes = Vec::new();
    image::codecs::png::PngEncoder::new(&mut bytes)
        .write_image(pixels.as_flattened(), 4, 3, image::ExtendedColorType::Rgba8)
        .unwrap();
    bytes
}

fn crop(source: (u32, u32), x: u32, y: u32, width: u32, height: u32) -> PngCrop {
    PngCrop {
        source_width: source.0,
        source_height: source.1,
        x,
        y,
        width,
        height,
    }
}

#[test]
fn relative_paths_admit_only_ordinary_components_inside_the_collection() {
    for accepted in ["first.png", "pictures/first.png", "a/b/c"] {
        assert_eq!(relative_path(accepted).unwrap(), Path::new(accepted));
    }
    for refused in [
        "",
        "/absolute.png",
        "../outside.png",
        "pictures/../../outside.png",
        "./first.png",
        "pictures\\first.png",
    ] {
        assert!(relative_path(refused).is_err(), "{refused:?}");
    }
}

#[test]
fn regular_files_refuse_links_directories_and_a_linked_root() {
    use std::os::unix::fs::symlink;
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("collection");
    fs::create_dir_all(root.join("pictures")).unwrap();
    fs::write(root.join("pictures/first.png"), b"first").unwrap();
    fs::write(directory.path().join("outside.png"), b"outside").unwrap();
    assert_eq!(
        regular_file(&root, "pictures/first.png").unwrap(),
        root.join("pictures/first.png")
    );

    symlink(
        directory.path().join("outside.png"),
        root.join("linked.png"),
    )
    .unwrap();
    symlink(root.join("pictures"), root.join("linked-directory")).unwrap();
    for refused in [
        "linked.png",
        "linked-directory/first.png",
        "pictures",
        "pictures/first.png/below",
        "absent.png",
        "../outside.png",
    ] {
        assert!(regular_file(&root, refused).is_err(), "{refused}");
    }

    let alias = directory.path().join("collection-link");
    symlink(&root, &alias).unwrap();
    assert!(regular_file(&alias, "pictures/first.png").is_err());
    let file_root = root.join("pictures/first.png");
    assert!(regular_file(&file_root, "first.png").is_err());
}

#[test]
fn bounded_reads_admit_the_limit_and_refuse_one_byte_more() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("record");
    fs::write(&path, b"abcd").unwrap();
    assert_eq!(read_bounded(&path, 4).unwrap(), b"abcd");
    assert_eq!(read_bounded(&path, 5).unwrap(), b"abcd");
    assert!(read_bounded(&path, 3).is_err());
    assert!(read_bounded(&directory.path().join("absent"), 4).is_err());
}

#[test]
fn verification_requires_both_the_length_and_the_digest() {
    verify_bytes(b"abc", 3, ABC_SHA256).unwrap();
    assert!(verify_bytes(b"abc", 4, ABC_SHA256).is_err());
    // Same length, different bytes: a matching length alone admits nothing.
    assert!(verify_bytes(b"abd", 3, ABC_SHA256).is_err());
    assert!(verify_bytes(b"abc", 3, &ABC_SHA256.to_uppercase()).is_err());
}

#[test]
fn verified_reads_check_path_size_and_identity() {
    use std::os::unix::fs::symlink;
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("collection");
    fs::create_dir_all(root.join("pictures")).unwrap();
    fs::write(root.join("pictures/abc.png"), b"abc").unwrap();
    assert_eq!(
        read_verified(&root, "pictures/abc.png", 3, ABC_SHA256).unwrap(),
        b"abc"
    );
    assert!(read_verified(&root, "pictures/abc.png", 4, ABC_SHA256).is_err());
    assert!(read_verified(&root, "pictures/abc.png", 2, ABC_SHA256).is_err());
    fs::write(root.join("pictures/abd.png"), b"abd").unwrap();
    assert!(read_verified(&root, "pictures/abd.png", 3, ABC_SHA256).is_err());
    symlink(root.join("pictures/abc.png"), root.join("linked.png")).unwrap();
    assert!(read_verified(&root, "linked.png", 3, ABC_SHA256).is_err());
    assert!(read_verified(&root, "../collection/pictures/abc.png", 3, ABC_SHA256).is_err());
}

#[test]
fn atlas_crop_preserves_selected_pixels_and_alpha_without_resizing() {
    let bytes = crop_png(&atlas(), crop((4, 3), 1, 1, 2, 2)).unwrap();
    let face = image::load_from_memory_with_format(&bytes, image::ImageFormat::Png)
        .unwrap()
        .into_rgba8();
    assert_eq!(face.dimensions(), (2, 2));
    assert_eq!(
        face.as_raw(),
        &[
            6, 20, 30, 0, 7, 40, 50, 128, 10, 60, 70, 255, 11, 80, 90, 255
        ]
    );
    let whole = crop_png(&atlas(), crop((4, 3), 0, 0, 4, 3)).unwrap();
    let whole = image::load_from_memory(&whole).unwrap().into_rgba8();
    assert_eq!(whole.dimensions(), (4, 3));
    assert_eq!(whole.get_pixel(3, 2).0, [12, 0, 0, 255]);
}

#[test]
fn atlas_crop_refuses_invalid_bounds_dimensions_and_invalid_images() {
    for (source, x, y, width, height) in [
        ((4, 3), 0, 0, 0, 1),
        ((4, 3), 0, 0, 1, 0),
        ((4, 3), 3, 0, 2, 1),
        ((4, 3), 0, 2, 1, 2),
        ((4, 3), u32::MAX, 0, 2, 1),
        ((4, 3), 0, u32::MAX, 1, 2),
        ((3, 3), 0, 0, 2, 2),
        ((5, 3), 0, 0, 2, 2),
        ((2048, 3), 0, 0, 2, 2),
        ((0, 3), 0, 0, 1, 1),
        ((4, 0), 0, 0, 1, 1),
    ] {
        assert!(
            crop_png(&atlas(), crop(source, x, y, width, height)).is_err(),
            "{source:?} {x} {y} {width} {height}"
        );
    }
    assert!(crop_png(b"not a png", crop((4, 3), 1, 1, 2, 2)).is_err());
}

#[test]
fn atlas_crop_refuses_a_declared_source_above_the_decoding_bound() {
    use image::ImageEncoder;
    // 1024 is the largest admitted side; 1025 is refused before decoding.
    let mut wide = Vec::new();
    image::codecs::png::PngEncoder::new(&mut wide)
        .write_image(&[0; 1025 * 4], 1025, 1, image::ExtendedColorType::Rgba8)
        .unwrap();
    assert!(crop_png(&wide, crop((1025, 1), 0, 0, 1, 1)).is_err());
    let mut edge = Vec::new();
    image::codecs::png::PngEncoder::new(&mut edge)
        .write_image(&[0; 1024 * 4], 1024, 1, image::ExtendedColorType::Rgba8)
        .unwrap();
    assert!(crop_png(&edge, crop((1024, 1), 1023, 0, 1, 1)).is_ok());
}

#[test]
fn publication_moves_a_complete_stage_and_never_replaces_a_destination() {
    let directory = tempfile::tempdir().unwrap();
    let stage = directory.path().join("stage");
    fs::create_dir(&stage).unwrap();
    fs::write(stage.join("manifest.json"), b"new").unwrap();
    let output = directory.path().join("assets");
    publish_directory(&stage, &output).unwrap();
    assert!(!stage.exists());
    assert_eq!(fs::read(output.join("manifest.json")).unwrap(), b"new");

    for existing in ["non-empty", "empty", "file"] {
        let destination = directory.path().join(existing);
        match existing {
            "non-empty" => {
                fs::create_dir(&destination).unwrap();
                fs::write(destination.join("owner-notes.txt"), b"keep this").unwrap();
            }
            "empty" => fs::create_dir(&destination).unwrap(),
            _ => fs::write(&destination, b"keep this").unwrap(),
        }
        let stage = directory.path().join(format!("stage-{existing}"));
        fs::create_dir(&stage).unwrap();
        fs::write(stage.join("manifest.json"), b"new").unwrap();
        let error = publish_directory(&stage, &destination).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("without replacing existing data"),
            "{error}"
        );
        assert_eq!(fs::read(stage.join("manifest.json")).unwrap(), b"new");
        match existing {
            "non-empty" => assert_eq!(
                fs::read(destination.join("owner-notes.txt")).unwrap(),
                b"keep this"
            ),
            "empty" => assert!(fs::read_dir(&destination).unwrap().next().is_none()),
            _ => assert_eq!(fs::read(&destination).unwrap(), b"keep this"),
        }
    }
}

#[test]
fn output_resolution_follows_existing_links_and_keeps_new_components() {
    use std::os::unix::fs::symlink;
    let directory = tempfile::tempdir().unwrap();
    let base = directory.path().canonicalize().unwrap();
    let source = base.join("source");
    fs::create_dir(&source).unwrap();
    let alias = base.join("source-link");
    symlink(&source, &alias).unwrap();

    // A consumer comparing the result with its source sees the overlap through the alias.
    assert_eq!(
        resolve_output(&alias.join("new/assets")).unwrap(),
        source.join("new/assets")
    );
    assert!(!source.join("new").exists(), "resolution creates nothing");
    assert_eq!(resolve_output(&source).unwrap(), source);
    assert_eq!(
        resolve_output(&base.join("absent-parent/assets")).unwrap(),
        base.join("absent-parent/assets")
    );
    assert!(!base.join("absent-parent").exists());
}
