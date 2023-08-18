use hashbrown::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Object {
    model: Model,
}

impl Object {
    pub fn new(model: Model) -> Self {
        Self { model }
    }

    pub fn cube(models: &ModelManager) -> Self {
        Self::new(models.get_model(ModelType::Cube))
    }

    pub fn width(&self) -> usize {
        self.model.width
    }

    pub fn depth(&self) -> usize {
        self.model.depth
    }

    pub fn height(&self) -> usize {
        self.model.height
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectType {
    Cube,
}

impl ObjectType {
    #[inline]
    pub fn get_object(self, models: &ModelManager) -> Object {
        match self {
            ObjectType::Cube => Object::cube(models),
        }
    }
}

pub struct ModelManager {
    models: HashMap<ModelType, Model>,
}

impl ModelManager {
    pub fn init() -> Self {
        // Cube model:
        let width = 4;
        let depth = 4;
        let height = 4;
        let cube_data = vec![vec![vec![1; height]; depth]; width]
            .into_iter()
            .flatten()
            .flatten()
            .collect();
        let cube = Model::new(cube_data, width, depth, height);

        // Cube with a hole model:
        let width = 6;
        let depth = 6;
        let height = 6;
        let mut cube_hole_data = vec![vec![vec![1; height]; depth]; width];
        for x in 1..width - 1 {
            for y in 0..depth {
                for z in 1..height - 1 {
                    cube_hole_data[x][y][z] = 0u8;
                }
            }
        }
        let cube_hole_data =
            cube_hole_data.into_iter().flatten().flatten().collect();
        let cube_hole = Model::new(cube_hole_data, width, depth, height);

        let models =
            [(ModelType::Cube, cube), (ModelType::CubeHole, cube_hole)]
                .iter()
                .cloned()
                .collect();

        Self { models }
    }

    pub fn get_model(&self, model_type: ModelType) -> Model {
        self.models.get(&model_type).unwrap().clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Model {
    width: usize,
    depth: usize,
    height: usize,
    data: Arc<Vec<u8>>,
}

impl Model {
    pub fn new(
        data: Vec<u8>,
        width: usize,
        depth: usize,
        height: usize,
    ) -> Self {
        Self {
            width,
            depth,
            height,
            data: Arc::new(data),
        }
    }

    #[inline]
    fn index_3d(&self, x: usize, y: usize, z: usize) -> usize {
        x + y * self.width + z * self.width * self.depth
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModelType {
    Cube,
    CubeHole,
}
