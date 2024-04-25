use dot_vox::{Color, Model};

const BLANK: Color = Color {
    r: 0,
    g: 0,
    b: 0,
    a: 0,
};

pub struct ModelManager {
    models: Vec<ModelData>,
}

impl ModelManager {
    pub fn new(models: Vec<ModelData>) -> Self {
        Self { models }
    }

    pub(super) fn get_model_data(&self, id: ModelID) -> ModelDataRef {
        self.models[id.0].as_ref()
    }
}

#[derive(Debug)]
pub struct ModelData {
    dimension: u32,
    voxels: Vec<Color>,
}

impl ModelData {
    pub fn from_vox_model(model: Model, palette: Vec<Color>) -> Self {
        assert!(
            model.size.x == model.size.y
                && model.size.y == model.size.z
                && model.size.x == model.size.z,
            "Dimensions of a voxel not equal!!!"
        );
        let dimension = model.size.x as u32;
        let mut voxels = vec![BLANK; (dimension * dimension * dimension) as usize];
        model.voxels.iter().for_each(|v| {
            // Replace y and z since vox models have z axis pointing up
            let index = position_to_index(dimension, v.x as u32, v.z as u32, v.y as u32);
            voxels[index] = palette[v.i as usize];
        });

        Self {
            dimension: model.size.x,
            voxels: voxels,
        }
    }

    fn as_ref(&self) -> ModelDataRef {
        ModelDataRef {
            dimension: self.dimension,
            voxels: &self.voxels,
        }
    }
}

#[derive(Debug)]
pub struct ModelDataRef<'a> {
    pub dimension: u32,
    pub voxels: &'a [Color],
}

impl<'a> ModelDataRef<'a> {
    #[inline]
    pub fn get_voxel(&self, x: u32, y: u32, z: u32) -> Option<Color> {
        let index = position_to_index(self.dimension, x, y, z);
        self.voxels.get(index).copied()
    }
}

// TODO Change input to usize or something
fn position_to_index(dimension: u32, x: u32, y: u32, z: u32) -> usize {
    (x + z * dimension + y * dimension * dimension) as usize
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ModelID(pub usize);
