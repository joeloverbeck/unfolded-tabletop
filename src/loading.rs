//! Required-asset load/dependency admission extracted from Arkham's native gate.
use bevy::{
    asset::{LoadState, RecursiveDependencyLoadState, UntypedAssetId},
    prelude::*,
};

pub struct RequiredAssets {
    pub loaded: bool,
    pub failures: Vec<String>,
    pub pending: Vec<String>,
}

/// Whether Bevy's text renderer has registered a font: [`font_registration`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FontRegistration {
    /// The text system has not reached the font yet: its alias is still empty.
    Pending,
    /// A family with at least one face is registered under the font's alias.
    Registered,
    /// The text system reached the font and registered no face for it: the
    /// renderer could not decode its bytes.
    Refused,
}

/// FontLoader reads bytes; registration proves the text renderer decoded them.
/// Bevy's text system sets a font's alias when it registers the font, so an empty
/// alias is [`FontRegistration::Pending`] and an alias with no face under it is
/// [`FontRegistration::Refused`].
///
/// No test registers a real font: the tests hold the pending and refused cases only.
pub fn font_registration(font: &Font, context: &mut bevy::text::FontCx) -> FontRegistration {
    if font.alias.is_empty() {
        FontRegistration::Pending
    } else if context
        .collection
        .family_by_name(&font.alias)
        .is_some_and(|family| !family.fonts().is_empty())
    {
        FontRegistration::Registered
    } else {
        FontRegistration::Refused
    }
}

/// Whether the font is [`FontRegistration::Registered`]. A game that must tell a
/// font not registered yet from one refused asks [`font_registration`].
pub fn font_registered(font: &Font, context: &mut bevy::text::FontCx) -> bool {
    font_registration(font, context) == FontRegistration::Registered
}

/// `present` checks the real decoded resource, including consumer font rules.
pub fn required_assets(
    server: &AssetServer,
    ids: &[UntypedAssetId],
    mut present: impl FnMut(UntypedAssetId) -> bool,
) -> RequiredAssets {
    let mut report = RequiredAssets {
        loaded: !ids.is_empty(),
        failures: vec![],
        pending: vec![],
    };
    if ids.is_empty() {
        report
            .failures
            .push("The required asset set is empty.".into());
    }
    for id in ids {
        let path = server.get_path(*id).map_or_else(
            || "unregistered required asset".into(),
            |path| path.to_string(),
        );
        let state = server.get_load_state(*id);
        let dependencies = server.get_recursive_dependency_load_state(*id);
        match (&state, &dependencies) {
            (Some(LoadState::Failed(error)), _)
            | (_, Some(RecursiveDependencyLoadState::Failed(error))) => {
                report.failures.push(format!("{path}: {error}"))
            }
            _ => {}
        }
        let ready = present(*id)
            && matches!(state, Some(LoadState::Loaded))
            && matches!(dependencies, Some(RecursiveDependencyLoadState::Loaded));
        if !ready {
            report
                .pending
                .push(format!("{path}: asset or dependencies have not loaded"));
        }
        report.loaded &= ready;
    }
    report
}
