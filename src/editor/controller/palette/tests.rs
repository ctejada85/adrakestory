use super::*;

#[test]
fn test_truncate_name_short() {
    assert_eq!(truncate_name("Grass", 8), "Grass");
}

#[test]
fn test_truncate_name_long() {
    assert_eq!(truncate_name("VeryLongName", 8), "VeryLon…");
}

#[test]
fn test_truncate_name_exact() {
    assert_eq!(truncate_name("Exactly8", 8), "Exactly8");
}
