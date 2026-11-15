#![cfg(feature = "render")]
//! Required-asset load reports at the public interface, through a real
//! `AssetServer` reading test-made files. Wording and admission rules are the
//! module's; the asset type and loader here belong to no game.
use bevy::{
    asset::{AssetLoader, LoadContext, UntypedAssetId, io::Reader},
    prelude::*,
};
use std::time::{Duration, Instant};
use unfolded_tabletop::loading::{RequiredAssets, font_registered, required_assets};

#[derive(Asset, TypePath)]
struct Note {
    #[dependency]
    child: Option<Handle<Note>>,
}

#[derive(TypePath)]
struct NoteLoader;

impl AssetLoader for NoteLoader {
    type Asset = Note;
    type Settings = ();
    type Error = std::io::Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _: &(),
        context: &mut LoadContext<'_>,
    ) -> Result<Note, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let text = String::from_utf8(bytes).map_err(std::io::Error::other)?;
        if text == "fail" {
            return Err(std::io::Error::other("this note cannot be decoded"));
        }
        let child = text
            .strip_prefix("dep:")
            .map(|path| context.load(path.to_owned()));
        Ok(Note { child })
    }

    fn extensions(&self) -> &[&str] {
        &["note"]
    }
}

struct Fixture {
    app: App,
    _root: tempfile::TempDir,
}

fn fixture(files: &[(&str, &str)]) -> Fixture {
    let root = tempfile::tempdir().unwrap();
    for (path, contents) in files {
        std::fs::write(root.path().join(path), contents).unwrap();
    }
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        AssetPlugin {
            file_path: root.path().to_string_lossy().into_owned(),
            ..default()
        },
    ))
    .init_asset::<Note>()
    .register_asset_loader(NoteLoader);
    Fixture { app, _root: root }
}

impl Fixture {
    fn load(&self, path: &str) -> Handle<Note> {
        self.app
            .world()
            .resource::<AssetServer>()
            .load(path.to_owned())
    }

    /// Update until every handle's own and recursive dependency state has settled.
    fn settle(&mut self, handles: &[&Handle<Note>]) {
        use bevy::asset::{LoadState, RecursiveDependencyLoadState as Deps};
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            self.app.update();
            let server = self.app.world().resource::<AssetServer>();
            if handles.iter().all(|handle| {
                matches!(
                    server.get_load_state(handle.id()),
                    Some(LoadState::Loaded | LoadState::Failed(_))
                ) && matches!(
                    server.get_recursive_dependency_load_state(handle.id()),
                    Some(Deps::Loaded | Deps::Failed(_))
                )
            }) {
                return;
            }
            assert!(Instant::now() < deadline, "test assets did not settle");
            std::thread::sleep(Duration::from_millis(2));
        }
    }

    fn report(
        &self,
        ids: &[UntypedAssetId],
        present: impl FnMut(UntypedAssetId) -> bool,
    ) -> RequiredAssets {
        required_assets(self.app.world().resource::<AssetServer>(), ids, present)
    }

    fn decoded(&self) -> impl FnMut(UntypedAssetId) -> bool + '_ {
        let notes = self.app.world().resource::<Assets<Note>>();
        move |id| notes.contains(id.typed::<Note>())
    }
}

const PENDING: &str = ": asset or dependencies have not loaded";

#[test]
fn an_empty_required_set_never_loads() {
    let fixture = fixture(&[]);
    let report = fixture.report(&[], |_| true);
    assert!(!report.loaded);
    assert_eq!(
        report.failures,
        ["The required artwork set is empty. Play cannot start."]
    );
    assert!(report.pending.is_empty());
}

#[test]
fn loaded_assets_with_loaded_dependencies_are_ready_only_when_present() {
    let mut fixture = fixture(&[
        ("plain.note", ""),
        ("parent.note", "dep:child.note"),
        ("child.note", ""),
    ]);
    let plain = fixture.load("plain.note");
    let parent = fixture.load("parent.note");
    fixture.settle(&[&plain, &parent]);
    let ids = [plain.id().untyped(), parent.id().untyped()];

    let report = fixture.report(&ids, fixture.decoded());
    assert!(report.loaded);
    assert!(report.failures.is_empty() && report.pending.is_empty());

    // The consumer's decoded-resource check withholds readiness by itself.
    let refused = plain.id().untyped();
    let report = fixture.report(&ids, |id| id != refused);
    assert!(!report.loaded);
    assert!(report.failures.is_empty());
    assert_eq!(report.pending, [format!("plain.note{PENDING}")]);
}

#[test]
fn own_and_dependency_failures_are_reported_once_per_asset_and_stay_pending() {
    let mut fixture = fixture(&[
        ("plain.note", ""),
        ("broken.note", "fail"),
        ("orphan.note", "dep:absent-child.note"),
    ]);
    let plain = fixture.load("plain.note");
    let missing = fixture.load("absent.note");
    let broken = fixture.load("broken.note");
    let orphan = fixture.load("orphan.note");
    fixture.settle(&[&plain, &missing, &broken, &orphan]);
    let ids = [&plain, &missing, &broken, &orphan].map(|handle| handle.id().untyped());

    let report = fixture.report(&ids, |_| true);
    assert!(!report.loaded, "one ready asset does not admit the set");
    assert_eq!(report.failures.len(), 3, "{:?}", report.failures);
    for (failure, path) in report
        .failures
        .iter()
        .zip(["absent.note", "broken.note", "orphan.note"])
    {
        assert!(failure.starts_with(&format!("{path}: ")), "{failure}");
    }
    assert!(report.failures[1].contains("this note cannot be decoded"));
    assert_eq!(
        report.pending,
        ["absent.note", "broken.note", "orphan.note"].map(|path| format!("{path}{PENDING}"))
    );
}

#[test]
fn an_asset_unknown_to_the_server_is_pending_without_a_failure() {
    let mut fixture = fixture(&[]);
    let id = fixture
        .app
        .world_mut()
        .resource_mut::<Assets<Note>>()
        .add(Note { child: None })
        .id()
        .untyped();
    let report = fixture.report(&[id], |_| true);
    assert!(!report.loaded);
    assert!(report.failures.is_empty());
    assert_eq!(
        report.pending,
        [format!("unregistered required asset{PENDING}")]
    );
}

#[test]
fn a_font_without_a_registered_family_under_its_alias_does_not_count() {
    let mut context = bevy::text::FontCx::default();
    // FontLoader reads bytes and leaves the alias empty until registration. A
    // positive case needs decodable font bytes, which this repository does not hold.
    let unaliased = Font::from_bytes(b"not a font".to_vec());
    assert!(!font_registered(&unaliased, &mut context));
    let mut unregistered = Font::from_bytes(b"not a font".to_vec());
    unregistered.alias = "asset_id:unregistered".into();
    assert!(!font_registered(&unregistered, &mut context));
}
