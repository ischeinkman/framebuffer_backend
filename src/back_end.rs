use std::ptr;

use texture::*;

use graphics::{self, types};

use primitives::{BufferPoint, Triangle, TextureTriangle};
use std::f32;

pub struct CoordinateTransform {
    matrix_transform : [f32 ; 4],
    translation : [f32; 2],
}

impl CoordinateTransform {

    pub const IDENTITY : CoordinateTransform = CoordinateTransform {
        matrix_transform : [1.0, 0.0, 0.0, 1.0],
        translation : [0.0, 0.0]
    };

    pub fn new(matrix_transform : [f32 ; 4], translation : [f32 ; 2]) -> CoordinateTransform {
        CoordinateTransform {
            matrix_transform, 
            translation
        }
    }

    pub fn apply_to_bufferpoint(&self, vect : &[f32 ; 2]) -> BufferPoint {
        let x = vect[0];
        let y = vect[1];

        let x_out = x * self.matrix_transform[0] + y * self.matrix_transform[1] + self.translation[0]; 
        let y_out = x * self.matrix_transform[2] + y * self.matrix_transform[3] + self.translation[1];

        BufferPoint::new(x_out as usize, y_out as usize) 
    }

    pub fn with_origin(&self, new_origin : &[f32 ; 2]) -> CoordinateTransform {
        CoordinateTransform {
            matrix_transform : self.matrix_transform,
            translation : [self.translation[0] - new_origin[0], self.translation[1] - new_origin[1]]
        }
    }

    pub fn with_scale(&self, x_scale : f32, y_scale : f32) -> CoordinateTransform {
        CoordinateTransform {
            matrix_transform : [
                self.matrix_transform[0] * x_scale, self.matrix_transform[1] * x_scale,
                self.matrix_transform[2] * y_scale, self.matrix_transform[3] * y_scale
            ],
            translation : self.translation,
        }
    }

}

pub struct RgbaBufferGraphics {
    width: usize,
    height: usize,
    buffer: *mut u8,
    transform : CoordinateTransform,
}


impl RgbaBufferGraphics {
    pub fn new(width: usize, height: usize, buffer: *mut u8) -> RgbaBufferGraphics {
        use graphics::Graphics;
        let mut retval = RgbaBufferGraphics {
            width,
            height,
            buffer,
            transform : CoordinateTransform::IDENTITY,
        };
        retval.clear_color([0.0,1.0,0.0,1.0]);
        retval
    }

    pub fn with_transform(width: usize, height: usize, buffer: *mut u8, transform : CoordinateTransform) -> RgbaBufferGraphics {
        use graphics::Graphics;
        let mut retval = RgbaBufferGraphics {
            width,
            height,
            buffer,
            transform,
        };
        retval.clear_color([0.0,1.0,0.0,1.0]);
        retval
    }



    #[inline]
    pub fn coords_to_pixel_index(&self, p: &BufferPoint) -> usize {
        p.x + p.y * self.width
    }

    #[inline]
    pub fn write_color(&mut self, pixel_index : usize, color : &types::Color) {
        if pixel_index > self.width * self.height - 1 {
            return;
        }
        let converted_color = [
            piston_color_channel_to_byte(color[0]),
            piston_color_channel_to_byte(color[1]),
            piston_color_channel_to_byte(color[2]),
            piston_color_channel_to_byte(color[3]),
        ];
        self.write_color_bytes(pixel_index, converted_color);
    }

    #[inline]
    pub fn write_color_bytes(&mut self, pixel_index: usize, color: [u8 ; 4]) {

        let red_new = color[0];
        let green_new = color[1];
        let blue_new = color [2];
        let alpha_new = color[3];

        if alpha_new == 0 {
            return;
        }
            
        let red_idx = pixel_index as isize * 4;
        let green_idx = red_idx + 1;
        let blue_idx = green_idx + 1;
        let alpha_idx = blue_idx + 1;

        let alpha_old : u8 = unsafe { ptr::read(self.buffer.offset(alpha_idx)) };
        let (red, green, blue, alpha) = if alpha_new != 255 && alpha_old != 0{
            let red_old : u8 = unsafe { ptr::read(self.buffer.offset(red_idx)) };
            let green_old : u8 = unsafe { ptr::read(self.buffer.offset(green_idx)) };
            let blue_old : u8 = unsafe { ptr::read(self.buffer.offset(blue_idx)) };

            let red = ((red_new as u16 * alpha_new as u16 + red_old as u16 * alpha_old as u16) / (alpha_new as u16 + alpha_old as u16)) as u8;
            let green = ((green_new as u16 * alpha_new as u16 + green_old as u16 * alpha_old as u16) / (alpha_new as u16 + alpha_old as u16)) as u8;
            let blue = ((blue_new as u16 * alpha_new as u16 + blue_old as u16 * alpha_old as u16) / (alpha_new as u16 + alpha_old as u16)) as u8;
            let alpha = 128;//if alpha_old > 255 - alpha_new { 255 } else { alpha_old + alpha_new};
            (red, green, blue, alpha)

        } else { (red_new, green_new, blue_new, alpha_new) };

        println!("red : {} => {}", red_idx, red);
        println!("green : {} => {}", green_idx, green);
        println!("blue : {} => {}", blue_idx, blue);
        println!("alpha : {} => {}", alpha_idx, alpha);

        unsafe {
            ptr::write(self.buffer.offset(red_idx), red);
            ptr::write(self.buffer.offset(green_idx), green);
            ptr::write(self.buffer.offset(blue_idx), blue);
            ptr::write(self.buffer.offset(alpha_idx), alpha);
        }
    }

    pub fn vertex_to_pixel_coords(&self, v: [f32; 2]) -> BufferPoint {
        self.transform.apply_to_bufferpoint(&v)
    }
}


impl graphics::Graphics for RgbaBufferGraphics {
    type Texture = RgbaTexture;
    fn clear_color(&mut self, color: types::Color) {
        let num_pixels = self.width * self.height;
        for i in 0..num_pixels {
            self.write_color(i, &color);
        }
    }
    fn clear_stencil(&mut self, _value: u8) {
        //TODO:this
    }
    fn tri_list<F>(&mut self, _draw_state: &graphics::DrawState, color: &[f32; 4], mut f: F) where F: FnMut(&mut FnMut(&[[f32; 2]])) {
        f(&mut |verts: &[[f32; 2]]| {
            for t in 0..verts.len() / 3 {
                let v1 = verts[t * 3];
                let v2 = verts[t * 3 + 1];
                let v3 = verts[t * 3 + 2];
                let tri = Triangle::new(self.vertex_to_pixel_coords(v1),
                                        self.vertex_to_pixel_coords(v2),
                                        self.vertex_to_pixel_coords(v3));
                tri.render(self, color);
            }
        })
    }
    fn tri_list_uv<F>(&mut self, _draw_state: &graphics::DrawState, color: &[f32; 4], texture: &<Self as graphics::Graphics>::Texture, mut f: F) where F: FnMut(&mut FnMut(&[[f32; 2]], &[[f32; 2]])) {
        f(&mut |verts: &[[f32; 2]], text_verts : &[[f32 ; 2]]| {
            for idx in 0..verts.len() / 3 {

                let v1 = verts[idx * 3 + 0];
                let v1_pt = self.vertex_to_pixel_coords(v1);

                let v2 = verts[idx * 3 + 1];
                let v2_pt = self.vertex_to_pixel_coords(v2);
                
                let v3 = verts[idx * 3 + 2];
                let v3_pt = self.vertex_to_pixel_coords(v3);
                
                let t1 = text_verts[idx * 3 + 0];
                let t1_pt = texture.vertex_to_pixel_coords(t1);

                let t2 = text_verts[idx * 3 + 1];
                let t2_pt = texture.vertex_to_pixel_coords(t2);
                
                let t3 = text_verts[idx * 3 + 2];
                let t3_pt = texture.vertex_to_pixel_coords(t3);

                let tri = TextureTriangle::new((v1_pt, t1_pt), (v2_pt, t2_pt), (v3_pt, t3_pt), &texture);

                tri.render(self, color);
            }
        })
    }
}