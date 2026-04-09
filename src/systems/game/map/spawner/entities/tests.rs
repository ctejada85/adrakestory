use super::*;

#[test]
fn test_parse_light_intensity_default() {
    let props = HashMap::new();
    assert_eq!(parse_light_intensity(&props), 10000.0);
}

#[test]
fn test_parse_light_intensity_custom() {
    let mut props = HashMap::new();
    props.insert("intensity".to_string(), "5000.0".to_string());
    assert_eq!(parse_light_intensity(&props), 5000.0);
}

#[test]
fn test_parse_light_intensity_clamped_high() {
    let mut props = HashMap::new();
    props.insert("intensity".to_string(), "2000000.0".to_string());
    assert_eq!(parse_light_intensity(&props), 1000000.0);
}

#[test]
fn test_parse_light_intensity_clamped_low() {
    let mut props = HashMap::new();
    props.insert("intensity".to_string(), "-100.0".to_string());
    assert_eq!(parse_light_intensity(&props), 0.0);
}

#[test]
fn test_parse_light_intensity_invalid() {
    let mut props = HashMap::new();
    props.insert("intensity".to_string(), "not_a_number".to_string());
    assert_eq!(parse_light_intensity(&props), 10000.0); // Falls back to default
}

#[test]
fn test_parse_light_range_default() {
    let props = HashMap::new();
    assert_eq!(parse_light_range(&props), 10.0);
}

#[test]
fn test_parse_light_range_custom() {
    let mut props = HashMap::new();
    props.insert("range".to_string(), "25.0".to_string());
    assert_eq!(parse_light_range(&props), 25.0);
}

#[test]
fn test_parse_light_range_clamped_high() {
    let mut props = HashMap::new();
    props.insert("range".to_string(), "500.0".to_string());
    assert_eq!(parse_light_range(&props), 100.0);
}

#[test]
fn test_parse_light_range_clamped_low() {
    let mut props = HashMap::new();
    props.insert("range".to_string(), "0.01".to_string());
    assert_eq!(parse_light_range(&props), 0.1);
}

#[test]
fn test_parse_shadows_enabled_default() {
    let props = HashMap::new();
    assert!(!parse_shadows_enabled(&props));
}

#[test]
fn test_parse_shadows_enabled_true() {
    let mut props = HashMap::new();
    props.insert("shadows".to_string(), "true".to_string());
    assert!(parse_shadows_enabled(&props));
}

#[test]
fn test_parse_shadows_enabled_one() {
    let mut props = HashMap::new();
    props.insert("shadows".to_string(), "1".to_string());
    assert!(parse_shadows_enabled(&props));
}

#[test]
fn test_parse_shadows_enabled_false() {
    let mut props = HashMap::new();
    props.insert("shadows".to_string(), "false".to_string());
    assert!(!parse_shadows_enabled(&props));
}

#[test]
fn test_parse_color_valid() {
    let mut props = HashMap::new();
    props.insert("color".to_string(), "1.0,0.5,0.0".to_string());
    let color = parse_color(&props);
    assert!(color.is_some());
}

#[test]
fn test_parse_color_with_spaces() {
    let mut props = HashMap::new();
    props.insert("color".to_string(), "1.0, 0.5, 0.0".to_string());
    let color = parse_color(&props);
    assert!(color.is_some());
}

#[test]
fn test_parse_color_missing() {
    let props = HashMap::new();
    assert!(parse_color(&props).is_none());
}

#[test]
fn test_parse_color_invalid_format() {
    let mut props = HashMap::new();
    props.insert("color".to_string(), "red".to_string());
    assert!(parse_color(&props).is_none());
}

#[test]
fn test_parse_color_incomplete() {
    let mut props = HashMap::new();
    props.insert("color".to_string(), "1.0,0.5".to_string()); // Only 2 values
    assert!(parse_color(&props).is_none());
}

#[test]
fn test_parse_npc_radius_default() {
    let props = HashMap::new();
    assert_eq!(parse_npc_radius(&props), 0.3);
}

#[test]
fn test_parse_npc_radius_custom() {
    let mut props = HashMap::new();
    props.insert("radius".to_string(), "0.5".to_string());
    assert_eq!(parse_npc_radius(&props), 0.5);
}

#[test]
fn test_parse_npc_name_default() {
    let props = HashMap::new();
    assert_eq!(parse_npc_name(&props), "NPC");
}

#[test]
fn test_parse_npc_name_custom() {
    let mut props = HashMap::new();
    props.insert("name".to_string(), "Bob".to_string());
    assert_eq!(parse_npc_name(&props), "Bob");
}

// --- FlickerLight helpers ---

#[test]
fn test_parse_flicker_enabled_default() {
    let props = HashMap::new();
    assert!(!parse_flicker_enabled(&props));
}

#[test]
fn test_parse_flicker_enabled_true() {
    let mut props = HashMap::new();
    props.insert("flicker".to_string(), "true".to_string());
    assert!(parse_flicker_enabled(&props));
}

#[test]
fn test_parse_flicker_enabled_one() {
    let mut props = HashMap::new();
    props.insert("flicker".to_string(), "1".to_string());
    assert!(parse_flicker_enabled(&props));
}

#[test]
fn test_parse_flicker_enabled_false() {
    let mut props = HashMap::new();
    props.insert("flicker".to_string(), "false".to_string());
    assert!(!parse_flicker_enabled(&props));
}

#[test]
fn test_parse_flicker_amplitude_default() {
    let props = HashMap::new();
    assert_eq!(parse_flicker_amplitude(&props), 3000.0);
}

#[test]
fn test_parse_flicker_amplitude_custom() {
    let mut props = HashMap::new();
    props.insert("flicker_amplitude".to_string(), "5000.0".to_string());
    assert_eq!(parse_flicker_amplitude(&props), 5000.0);
}

#[test]
fn test_parse_flicker_amplitude_clamped_high() {
    let mut props = HashMap::new();
    props.insert("flicker_amplitude".to_string(), "999999.0".to_string());
    assert_eq!(parse_flicker_amplitude(&props), 100_000.0);
}

#[test]
fn test_parse_flicker_amplitude_clamped_low() {
    let mut props = HashMap::new();
    props.insert("flicker_amplitude".to_string(), "-50.0".to_string());
    assert_eq!(parse_flicker_amplitude(&props), 0.0);
}

#[test]
fn test_parse_flicker_amplitude_invalid() {
    let mut props = HashMap::new();
    props.insert("flicker_amplitude".to_string(), "loud".to_string());
    assert_eq!(parse_flicker_amplitude(&props), 3000.0);
}

#[test]
fn test_parse_flicker_speed_default() {
    let props = HashMap::new();
    assert_eq!(parse_flicker_speed(&props), 4.0);
}

#[test]
fn test_parse_flicker_speed_custom() {
    let mut props = HashMap::new();
    props.insert("flicker_speed".to_string(), "8.0".to_string());
    assert_eq!(parse_flicker_speed(&props), 8.0);
}

#[test]
fn test_parse_flicker_speed_clamped_high() {
    let mut props = HashMap::new();
    props.insert("flicker_speed".to_string(), "100.0".to_string());
    assert_eq!(parse_flicker_speed(&props), 20.0);
}

#[test]
fn test_parse_flicker_speed_clamped_low() {
    let mut props = HashMap::new();
    props.insert("flicker_speed".to_string(), "0.0".to_string());
    assert_eq!(parse_flicker_speed(&props), 0.1);
}

#[test]
fn test_parse_flicker_speed_invalid() {
    let mut props = HashMap::new();
    props.insert("flicker_speed".to_string(), "fast".to_string());
    assert_eq!(parse_flicker_speed(&props), 4.0);
}
