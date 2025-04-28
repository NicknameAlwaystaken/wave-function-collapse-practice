use eframe::egui::{Color32, ColorImage};
use image::RgbImage;



pub fn blit_tile_onto_rgbimage(
    canvas: &mut RgbImage,
    tile: &RgbImage,
    offset_x: u32,
    offset_y: u32,
) {
    for y in 0..tile.height() {
        for x in 0..tile.width() {
            let pixel = tile.get_pixel(x, y);
            canvas.put_pixel(x + offset_x, y + offset_y, *pixel);
        }
    }
}

pub fn blit_tile_onto_colorimage(
    canvas: &mut ColorImage,
    tile: &RgbImage,
    offset_x: u32,
    offset_y: u32,
) {
    let canvas_width = canvas.size[0];

    for (dy, row) in tile.rows().enumerate() {
        for (dx, pixel) in row.enumerate() {
            let x = offset_x as usize + dx;
            let y = offset_y as usize + dy;
            let index = y * canvas_width + x;

            let [r, g, b] = pixel.0;
            canvas.pixels[index] = Color32::from_rgb(r, g, b);
        }
    }
}
