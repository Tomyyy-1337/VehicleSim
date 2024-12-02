use nannou::glam::Vec2;

pub struct PositionMap<T> {
    chunks_size_px: f32,
    num_chunks_x: usize,
    num_chunks_y: usize,
    chunks: Vec<Vec<(Vec2, T)>>,
}

impl<T> PositionMap<T> {
    pub fn new(chunks_size_px: f32, width: f32, height: f32) -> Self {
        let num_chunks_x = (width / chunks_size_px).ceil() as usize;
        let num_chunks_y = (height / chunks_size_px).ceil() as usize;
        let chunks = (0..num_chunks_x * num_chunks_y).map(|_| Vec::new()).collect();

        PositionMap {
            chunks_size_px,
            num_chunks_x,
            num_chunks_y,
            chunks,
        }
    }

    pub fn add_value(&mut self, x: f32, y: f32, value: T) {
        let chunk_x = (x / self.chunks_size_px).floor() as usize;
        let chunk_y = (y / self.chunks_size_px).floor() as usize;

        if let Some(chunk) = self.chunks.get_mut(chunk_y * self.num_chunks_x + chunk_x) {
            chunk.push((Vec2::new(x, y), value));
        }
    }

    pub fn get_chunk(&self, x: f32, y: f32) -> Option<&Vec<(Vec2, T)>> {
        let chunk_x = (x / self.chunks_size_px).floor() as usize;
        let chunk_y = (y / self.chunks_size_px).floor() as usize;

        self.chunks.get(chunk_y * self.num_chunks_x + chunk_x)
    }

    pub fn get_in_radius(&self, x: f32, y: f32, radius: f32) -> Vec<&(Vec2, T)> {
        let chunk_x = (x / self.chunks_size_px).floor() as usize;
        let chunk_y = (y / self.chunks_size_px).floor() as usize;

        let radius_chunks = (radius / self.chunks_size_px).ceil() as usize;

        let mut chunks = Vec::new();
        for y in (chunk_y.saturating_sub(radius_chunks))..=(chunk_y + radius_chunks) {
            for x in (chunk_x.saturating_sub(radius_chunks))..=(chunk_x + radius_chunks) {
                if let Some(chunk) = self.chunks.get(y * self.num_chunks_x + x) {
                    chunks.push(chunk);
                }
            }
        }

        chunks.into_iter().flatten().collect()
    }

    pub fn iter(&self) -> impl Iterator<Item = &(Vec2, T)> {
        self.chunks.iter().flatten()
    }
}