use super::Raycaster;

const GRAY: [u8; 4] = [100, 100, 100, 255];

pub fn fill(rc: &Raycaster, data: &mut [u8]) {
    let index = (rc.height / 2 * 4 * rc.width) as usize;
    data[index..]
        .chunks_exact_mut(4)
        .for_each(|rgba| rgba.copy_from_slice(&GRAY));
    //rc.column_buffer[0..index].chunks_exact_mut(4).for_each(|rgba| rgba.copy_from_slice(&RED));
    //for x in 0..rc.width as usize {
    //    let begin_y = 0;
    //    let end_y = rc.height as usize / 2;
    //    canvas.draw_line(x, begin_y, end_y, &rc.column_buffer[0..index]);
    //}
}
