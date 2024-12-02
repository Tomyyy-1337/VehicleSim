use nannou::glam::Vec2;

#[derive(Debug, Clone)]
pub struct Vehicle {
    pub position: Vec2,
    pub velocity: Vec2,
}

impl Vehicle {
    pub fn new() -> Self {
        Vehicle { 
            position: Vec2::ZERO,
            velocity: Vec2::ZERO,
        }
    }

    pub fn update(&mut self, delta: f32) {
        self.position += self.velocity * delta;
    }

    pub fn adjust(&mut self, target: &Vec2, factor: f32) {
        let distance = *target - self.position;
        let light_intensity = factor / distance.length().powi(2);
                    
        let acceleration = distance.normalize() * light_intensity;
        self.velocity -= acceleration * 1000.0;
    }

}