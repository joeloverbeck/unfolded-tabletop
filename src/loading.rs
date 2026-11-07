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

/// FontLoader reads bytes; registration proves the text renderer decoded them.
pub fn font_registered(font: &Font, context: &mut bevy::text::FontCx) -> bool {
    !font.alias.is_empty()
        && context
            .collection
            .family_by_name(&font.alias)
            .is_some_and(|family| !family.fonts().is_empty())
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
            .push("The required artwork set is empty. Play cannot start.".into());
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
