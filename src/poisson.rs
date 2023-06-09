use rand::Rng;

pub struct Poisson {}

impl Poisson {
    pub fn new() -> Self {
        Poisson {}
    }
    pub fn compute_new_position(
        &self,
        existing_points: &[(f32, f32)],
        near_point: &(f32, f32),
        radius: f32,
        nb_attempts: u32,
        random: &mut impl Rng,
    ) -> Option<(f32, f32)> {
        const EPSILON: f32 = 0.01f32;
        let seed = random.next_u64() as f32 / std::u64::MAX as f32;
        let radius_plus_epsilon = radius + EPSILON;
        let radius_squared = radius * radius;
        for attempt_amount in 0..nb_attempts {
            let theta =
                std::f32::consts::TAU * (seed + attempt_amount as f32 / (nb_attempts as f32));
            let test_point = (
                near_point.0 + radius_plus_epsilon * theta.cos(),
                near_point.1 + radius_plus_epsilon * theta.sin(),
            );
            let mut is_correct = true;
            for existing_point in existing_points.iter() {
                if distance_squared(existing_point, &test_point) <= radius_squared {
                    is_correct = false;
                    break;
                }
            }
            if is_correct {
                return Some(test_point);
            }
        }
        None
    }
}

pub fn distance_squared(p1: &(f32, f32), p2: &(f32, f32)) -> f32 {
    let dx = p2.0 - p1.0;
    let dy = p2.1 - p1.1;
    dx * dx + dy * dy
}
