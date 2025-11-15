use boid_core::{BoidConfig, Vector2D};

/// Shared state for boid simulation
pub struct SimulationState {
    pub target_position: Option<Vector2D>,
    pub config: BoidConfig,
}
