use super::Raycaster;

const RED: [u8; 4] = [200, 10, 10, 255];

pub fn fill(rc: &Raycaster, data: &mut [u8]) {
    let index = (rc.height / 2 * rc.width * 4) as usize;
    data[0..index]
        .chunks_exact_mut(4)
        .for_each(|rgba| rgba.copy_from_slice(&RED));
}
