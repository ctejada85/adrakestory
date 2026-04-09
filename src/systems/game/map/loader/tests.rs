use super::*;

#[test]
fn test_load_progress_percentage() {
    const EPSILON: f32 = 1e-6;
    let approx_eq = |a: f32, b: f32| (a - b).abs() < EPSILON;

    assert!(approx_eq(LoadProgress::Started.percentage(), 0.0));
    assert!(approx_eq(LoadProgress::LoadingFile(0.5).percentage(), 0.1));
    assert!(approx_eq(LoadProgress::ParsingData(0.5).percentage(), 0.3));
    assert!(approx_eq(
        LoadProgress::ValidatingMap(0.5).percentage(),
        0.5
    ));
    assert!(approx_eq(
        LoadProgress::SpawningVoxels(0.5).percentage(),
        0.75
    ));
    assert!(approx_eq(
        LoadProgress::SpawningEntities(0.5).percentage(),
        0.925
    ));
    assert!(approx_eq(LoadProgress::Finalizing(0.5).percentage(), 0.975));
    assert!(approx_eq(LoadProgress::Complete.percentage(), 1.0));
}

#[test]
fn test_map_load_progress() {
    let mut progress = MapLoadProgress::new();
    assert!(!progress.is_complete());
    assert!(!progress.has_error());

    progress.update(LoadProgress::Started);
    assert_eq!(progress.percentage(), 0.0);

    progress.update(LoadProgress::Complete);
    assert!(progress.is_complete());
    assert_eq!(progress.percentage(), 1.0);
}

#[test]
fn test_load_default_map() {
    let map = MapLoader::load_default();
    assert_eq!(map.metadata.name, "Default Map");
    assert!(validate_map(&map).is_ok());
}
