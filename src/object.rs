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

    pub fn hole(models: &ModelManager) -> Self {
        Self::new(models.get_model(ModelType::CubeHole))
    }

    pub fn voxel(models: &ModelManager) -> Self {
        Self::new(models.get_model(ModelType::Voxel))
    }

    pub fn get_voxel(&self, x: i32, y: i32, z: i32) -> Option<&u8> {
        let dimension = self.dimension() as i32;
        if z < 0
            || z >= dimension
            || x < 0
            || x >= dimension
            || y < 0
            || y >= dimension
        {
            return None;
        }
        self.model.get_voxel(x as usize, y as usize, z as usize)
    }

    pub fn dimension(&self) -> usize {
        self.model.dimension
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectType {
    Cube,
    Hole,
    Voxel,
}

impl ObjectType {
    #[inline]
    pub fn get_object(self, models: &ModelManager) -> Object {
        match self {
            ObjectType::Cube => Object::cube(models),
            ObjectType::Hole => Object::hole(models),
            ObjectType::Voxel => Object::voxel(models),
        }
    }
}

pub struct ModelManager {
    models: HashMap<ModelType, Model>,
}

impl ModelManager {
    pub fn init() -> Self {
        // Cube model:
        let dimension = 4;
        let cube_data = vec![vec![vec![1; dimension]; dimension]; dimension]
            .into_iter()
            .flatten()
            .flatten()
            .collect();
        let cube = Model::new(cube_data, dimension);

        // Cube with a hole model:
        let dimension = 6;
        let mut cube_hole_data =
            vec![vec![vec![1; dimension]; dimension]; dimension];
        for x in 1..dimension - 1 {
            for z in 0..dimension {
                for y in 1..dimension - 1 {
                    cube_hole_data[x][z][y] = 0u8;
                }
            }
        }
        let cube_hole_data =
            cube_hole_data.into_iter().flatten().flatten().collect();
        let cube_hole = Model::new(cube_hole_data, dimension);

        // Single voxel model:
        let dimension = 10;
        let mut voxel_data =
            vec![vec![vec![0; dimension]; dimension]; dimension];
        voxel_data[2][2][2] = 1;
        let voxel_data = voxel_data.into_iter().flatten().flatten().collect();
        let voxel = Model::new(voxel_data, dimension);

        let models = [
            (ModelType::Cube, cube),
            (ModelType::CubeHole, cube_hole),
            (ModelType::Voxel, voxel),
        ]
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
    dimension: usize,
    data: Arc<Vec<u8>>,
}

impl Model {
    pub fn new(data: Vec<u8>, dimension: usize) -> Self {
        Self {
            dimension,
            data: Arc::new(data),
        }
    }

    #[inline]
    fn get_voxel(&self, x: usize, y: usize, z: usize) -> Option<&u8> {
        let index =
            x + z * self.dimension + y * self.dimension * self.dimension;
        self.data.get(index)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModelType {
    Cube,
    CubeHole,
    Voxel,
}
