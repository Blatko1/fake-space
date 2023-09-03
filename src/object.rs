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

    pub fn pillars(models: &ModelManager) -> Self {
        Self::new(models.get_model(ModelType::Pillars))
    }

    pub fn damaged(models: &ModelManager) -> Self {
        Self::new(models.get_model(ModelType::Damaged))
    }

    #[inline]
    pub fn get_voxel(&self, x: usize, y: usize, z: usize) -> Option<&u8> {
        self.model.get_voxel(x, y, z)
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
    Pillars,
    Damaged,
}

impl ObjectType {
    #[inline]
    pub fn get_object(self, models: &ModelManager) -> Object {
        match self {
            Self::Cube => Object::cube(models),
            Self::Hole => Object::hole(models),
            Self::Voxel => Object::voxel(models),
            Self::Pillars => Object::pillars(models),
            Self::Damaged => Object::damaged(models),
        }
    }
}

pub struct ModelManager {
    models: HashMap<ModelType, Model>,
}

impl ModelManager {
    pub fn init() -> Self {
        // Cube model:
        let dimension = 2;
        let mut cube_data =
            vec![vec![vec![9; dimension]; dimension]; dimension];
        cube_data[0][0][0] = 0u8;
        let cube_data = cube_data.into_iter().flatten().flatten().collect();
        let cube = Model::new(cube_data, dimension);

        // Cube with a hole model:
        let dimension = 6;
        let mut cube_hole_data =
            vec![vec![vec![30; dimension]; dimension]; dimension];
        for x in 1..dimension - 1 {
            for z in 0..dimension {
                for y in 1..dimension - 1 {
                    cube_hole_data[y][z][x] = 0u8;
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
        voxel_data[2][2][2] = 100;
        let voxel_data = voxel_data.into_iter().flatten().flatten().collect();
        let voxel = Model::new(voxel_data, dimension);

        // pillars model:
        let dimension = 8;
        let mut pillars_data =
            vec![vec![vec![0; dimension]; dimension]; dimension];
        for x in 0..dimension / 2 {
            for z in 0..dimension / 2 {
                for y in 0..dimension {
                    pillars_data[y][z * 2][x * 2] = (x + z + y) as u8;
                }
            }
        }
        let pillars_data =
            pillars_data.into_iter().flatten().flatten().collect();
        let pillars = Model::new(pillars_data, dimension);

        // letter B model:
        let dimension = 8;
        let mut damaged_data =
            vec![vec![vec![1; dimension]; dimension]; dimension];
        for x in 0..4 {
            for z in 0..3 {
                for y in 5..8 {
                    damaged_data[y][z][x] = 0u8;
                }
            }
        }
        for y in 0..dimension {
            damaged_data[y][0][0] = 0u8;
            damaged_data[y][5][3] = y as u8;
        }
        for z in 0..dimension {
            damaged_data[4][z][3] = 0u8;
            damaged_data[4][z][4] = 0u8;
            damaged_data[4][z][5] = 0u8;
        }
        damaged_data[5][1][3] = 10u8;
        damaged_data[5][2][3] = 20u8;
        let damaged_data =
            damaged_data.into_iter().flatten().flatten().collect();
        let damaged = Model::new(damaged_data, dimension);

        let models = [
            (ModelType::Cube, cube),
            (ModelType::CubeHole, cube_hole),
            (ModelType::Voxel, voxel),
            (ModelType::Pillars, pillars),
            (ModelType::Damaged, damaged),
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
    Pillars,
    Damaged,
}
