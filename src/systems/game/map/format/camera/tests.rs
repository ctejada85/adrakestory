use super::*;

#[test]
fn camera_data_defaults_produce_none_optionals() {
    let ron = r#"(
        position: (1.0, 2.0, 3.0),
        look_at: (0.0, 0.0, 0.0),
        rotation_offset: 0.0,
    )"#;
    let cd: CameraData = ron::from_str(ron).expect("parse failed");
    assert!(cd.follow_speed.is_none());
    assert!(cd.rotation_speed.is_none());
    assert!(cd.fov_degrees.is_none());
}

#[test]
fn camera_data_follow_speed_round_trips() {
    let ron = r#"(
        position: (1.0, 2.0, 3.0),
        look_at: (0.0, 0.0, 0.0),
        rotation_offset: 0.0,
        follow_speed: Some(8.0),
    )"#;
    let cd: CameraData = ron::from_str(ron).expect("parse failed");
    assert_eq!(cd.follow_speed, Some(8.0));
}

#[test]
fn camera_data_rotation_speed_round_trips() {
    let ron = r#"(
        position: (1.0, 2.0, 3.0),
        look_at: (0.0, 0.0, 0.0),
        rotation_offset: 0.0,
        rotation_speed: Some(2.5),
    )"#;
    let cd: CameraData = ron::from_str(ron).expect("parse failed");
    assert_eq!(cd.rotation_speed, Some(2.5));
}

#[test]
fn camera_data_fov_degrees_round_trips() {
    let ron = r#"(
        position: (1.0, 2.0, 3.0),
        look_at: (0.0, 0.0, 0.0),
        rotation_offset: 0.0,
        fov_degrees: Some(90.0),
    )"#;
    let cd: CameraData = ron::from_str(ron).expect("parse failed");
    assert_eq!(cd.fov_degrees, Some(90.0));
}

#[test]
fn camera_data_all_optional_fields_present() {
    let ron = r#"(
        position: (5.0, 10.0, 5.0),
        look_at: (0.0, 0.0, 0.0),
        rotation_offset: -1.5707964,
        follow_speed: Some(5.0),
        rotation_speed: Some(3.0),
        fov_degrees: Some(75.0),
    )"#;
    let cd: CameraData = ron::from_str(ron).expect("parse failed");
    assert_eq!(cd.follow_speed, Some(5.0));
    assert_eq!(cd.rotation_speed, Some(3.0));
    assert_eq!(cd.fov_degrees, Some(75.0));
}

#[test]
fn camera_data_default_impl_has_none_optionals() {
    let cd = CameraData::default();
    assert!(cd.follow_speed.is_none());
    assert!(cd.rotation_speed.is_none());
    assert!(cd.fov_degrees.is_none());
}
