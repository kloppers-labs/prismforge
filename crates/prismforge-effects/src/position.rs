/// Normalised positions along an LED order: 0.0 at the first LED, 1.0 at the last; a single LED is at 0.0.
pub fn linear_positions(count: usize) -> Vec<f64> {
    let last = count.saturating_sub(1).max(1) as f64;
    (0..count).map(|index| index as f64 / last).collect()
}
