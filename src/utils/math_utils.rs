pub fn round_to_two_decimals(val: f32) -> f32 {
    (val * 100.0).round() / 100.0
}
