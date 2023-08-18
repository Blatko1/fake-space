pub struct Cube {

}

const CUBE_WIDTH: usize = 4;
const CUBE_HEIGHT: usize = 4;
const CUBE_DEPTH: usize = 4;

const CUBE_DATA: &[&[&[u8]]] = &[&[&[1; CUBE_DEPTH]; CUBE_HEIGHT]; CUBE_WIDTH];