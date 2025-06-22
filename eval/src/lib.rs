#[derive(Debug, Clone, serde_derive::Deserialize, serde_derive::Serialize, PartialEq)]
pub struct OutCsvRow {
    pub t_a: f64,
    pub t_b: f64,
    pub t_max: f64,
    pub t_min: f64,
    pub length: usize,
    // pub frac_ties_x: f64,
    // pub frac_ties_y: f64,
    pub frac_ties: f64,
    pub sum_of_tie_lengths: usize,
    pub tie_count: usize,
    pub longest_tie: usize,
    pub permutation_count: u128,
    pub compute_time: f32,
}
