#![cfg_attr(not(feature = "std"), no_std)]

use serde::{Deserialize, Serialize};

/// Represents a 2D position in screen coordinates
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

impl Position {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Calculate distance to another position
    pub fn distance_to(&self, other: &Position) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        libm::sqrtf(dx * dx + dy * dy)
    }
}

/// Hand landmark data from tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandLandmarks {
    pub thumb_tip: Position,
    pub index_tip: Position,
}

impl HandLandmarks {
    pub fn new(thumb_tip: Position, index_tip: Position) -> Self {
        Self {
            thumb_tip,
            index_tip,
        }
    }

    /// Calculate pinch distance (distance between thumb and index finger tips)
    pub fn pinch_distance(&self) -> f32 {
        self.thumb_tip.distance_to(&self.index_tip)
    }
}

/// Update message sent from client to ESP32 to control boid target position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetPositionUpdate {
    /// Optional target position (None means no target/free flying)
    pub position: Option<Position>,
}

/// Boid simulation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoidSettings {
    pub separation_weight: f32,
    pub alignment_weight: f32,
    pub cohesion_weight: f32,
    pub max_speed: f32,
    pub max_force: f32,
    pub seek_weight: f32,
}

impl Default for BoidSettings {
    fn default() -> Self {
        Self {
            separation_weight: 1.5,
            alignment_weight: 1.0,
            cohesion_weight: 1.0,
            max_speed: 2.0,
            max_force: 0.05,
            seek_weight: 8.0,
        }
    }
}

/// Settings update message sent from client to ESP32
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsUpdate {
    pub settings: BoidSettings,
}

/// Status response from ESP32
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusResponse {
    pub boid_count: usize,
    pub fps: u32,
    pub target_active: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_distance() {
        let p1 = Position::new(0.0, 0.0);
        let p2 = Position::new(3.0, 4.0);
        assert_eq!(p1.distance_to(&p2), 5.0);
    }

    #[test]
    fn test_pinch_distance() {
        let landmarks = HandLandmarks::new(Position::new(0.0, 0.0), Position::new(30.0, 40.0));
        assert_eq!(landmarks.pinch_distance(), 50.0);
    }
}
