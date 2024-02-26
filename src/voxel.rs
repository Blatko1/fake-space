pub struct VoxelModelManager {
    models: Vec<VoxelModelData>,
}

impl VoxelModelManager {
    #[allow(clippy::needless_range_loop)]
    pub fn init() -> Self {
        // Cube model:
        let dimension = 2;
        let mut cube_data = vec![vec![vec![9; dimension]; dimension]; dimension];
        cube_data[0][0][0] = 0u8;
        let cube_data = cube_data.into_iter().flatten().flatten().collect();
        let cube = VoxelModelData::new(cube_data, dimension);

        // Cube with a hole model:
        let dimension = 6;
        let mut cube_hole_data = vec![vec![vec![30; dimension]; dimension]; dimension];
        for x in 1..dimension - 1 {
            for z in 0..dimension {
                for y in 1..dimension - 1 {
                    cube_hole_data[y][z][x] = 0u8;
                }
            }
        }
        let cube_hole_data = cube_hole_data.into_iter().flatten().flatten().collect();
        let cube_hole = VoxelModelData::new(cube_hole_data, dimension);

        // Single voxel model:
        let dimension = 10;
        let mut voxel_data = vec![vec![vec![0; dimension]; dimension]; dimension];
        voxel_data[2][2][2] = 100;
        let voxel_data = voxel_data.into_iter().flatten().flatten().collect();
        let voxel = VoxelModelData::new(voxel_data, dimension);

        // pillars model:
        let dimension = 8;
        let mut pillars_data = vec![vec![vec![0; dimension]; dimension]; dimension];
        for x in 0..dimension / 2 {
            for z in 0..dimension / 2 {
                for y in 0..dimension {
                    pillars_data[y][z * 2][x * 2] = (x + z + y) as u8;
                }
            }
        }
        let pillars_data = pillars_data.into_iter().flatten().flatten().collect();
        let pillars = VoxelModelData::new(pillars_data, dimension);

        // damaged cube model:
        let dimension = 8;
        let mut damaged_data = vec![vec![vec![1; dimension]; dimension]; dimension];
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
        let damaged_data = damaged_data.into_iter().flatten().flatten().collect();
        let damaged = VoxelModelData::new(damaged_data, dimension);

        let models = vec![cube, cube_hole, voxel, pillars, damaged];

        Self { models }
    }

    pub fn get_model(&self, model_type: VoxelModelID) -> VoxelModelDataRef {
        self.models.get(model_type.to_index()).unwrap().as_ref()
    }
}

// TODO switch to 3D array instead of Vec
#[derive(Debug, PartialEq, Eq)]
pub struct VoxelModelData {
    pub dimension: usize,
    pub data: Vec<u8>,
}

impl VoxelModelData {
    pub fn new(data: Vec<u8>, dimension: usize) -> Self {
        Self { dimension, data }
    }

    pub fn as_ref(&self) -> VoxelModelDataRef {
        VoxelModelDataRef {
            dimension: self.dimension,
            data: self.data.as_slice(),
        }
    }
}

/// A [`VoxelModel`] reference for faster `data` access.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VoxelModelDataRef<'a> {
    pub dimension: usize,
    pub data: &'a [u8],
}

impl<'a> VoxelModelDataRef<'a> {
    #[inline]
    pub fn get_voxel(&self, x: usize, y: usize, z: usize) -> Option<&u8> {
        let index = x + z * self.dimension + y * self.dimension * self.dimension;
        self.data.get(index)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VoxelModelID {
    Cube = 0,
    CubeHole,
    Voxel,
    Pillars,
    Damaged,
}

impl VoxelModelID {
    #[inline]
    fn to_index(self) -> usize {
        self as usize
    }
}
