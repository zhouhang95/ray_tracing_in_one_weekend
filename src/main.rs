use image::{ImageBuffer, RgbImage, Rgb};

fn main() {
    let nx = 200;
    let ny = 100;
    let mut img: RgbImage = ImageBuffer::new(nx, ny);
    for j in 0..ny {
        for i in 0..nx {
            let r = i as f32 / nx as f32;
            let g = j as f32 / ny as f32;
            let b = 0.2;

            img.put_pixel(i, j, Rgb([
                (r * 255.99) as u8,
                (g * 255.99) as u8,
                (b * 255.99) as u8,
            ]));
        }
    }
    image::imageops::flip_vertical_in_place(&mut img);
    img.save("test.png").unwrap();
}
