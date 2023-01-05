use olc_pge as olc;
use crate::pixel_traits::*;

#[derive(Clone)]
pub struct Image
{
    pub width:usize,
    pub height:usize,
    pub pixels: Vec<olc::Pixel>
}

impl Image
{
    pub fn at(&self, x: usize, y: usize) -> &olc::Pixel
    {
        if x >= self.width || y >= self.height
        {
            panic!("in function at() of Image, pixel coordinates exceed image dimensions.");
        }
        &self.pixels[y*self.width+x]
    }

    pub fn at_mut(&mut self, x: usize, y: usize) -> &mut olc::Pixel
    {
        if x >= self.width || y >= self.height
        {
            panic!("in function at() of Image, pixel coordinates exceed image dimensions.");
        }
        &mut self.pixels[y*self.width+x]
    }

    pub fn convolve<F>(&mut self, target: &mut Image, kernel_size: usize, mut kernel_generator: F) where F: FnMut(usize, (usize, usize)) -> f32
    {
        for y in kernel_size/2..self.height - kernel_size/2
        {
            for x in kernel_size/2..self.width - kernel_size/2
            {
                let (mut r,mut g,mut b) = (0.0, 0.0, 0.0);
                for kernel_y in 0..kernel_size
                {
                    for kernel_x in 0..kernel_size
                    {
                        let kernel_value = kernel_generator(kernel_size, (kernel_x, kernel_y));
                        let pixel = *self.at(x - kernel_size/2 + kernel_x, y - kernel_size/2 + kernel_y);
                        r += pixel.r as f32 * kernel_value;
                        g += pixel.g as f32 * kernel_value;
                        b += pixel.b as f32 * kernel_value;
                    }
                }

                *target.at_mut(x, y) = olc::Pixel::rgb(r as u8, g as u8, b as u8);
            }
        }
    }

    pub fn painting(&self, target: &mut Image)
    {
        let S_Y = 
        [2, 1,-2,
        1,-3, 1,
       -2, 1, 2];
   
        let S_X = 
        [-2, 1, 2,
        1,-3, 1,
        2, 1,-2];

        for y in 0..self.height
        {
            for x in 0..self.width
            {
                if !(1..self.width-1).contains(&x)
                || !(1..self.height-1).contains(&y)
                {
                    *target.at_mut(x, y) = olc::BLACK;
                    continue;
                }
                let at_top_left =      self.at(x - 1,   y - 1   ).brightness();
                let at_top_middle =    self.at(x,       y - 1   ).brightness();
                let at_top_right =     self.at(x + 1,   y - 1   ).brightness();
                let at_left =          self.at(x - 1,   y       ).brightness();
                let at_middle =        self.at(x,       y       ).brightness();
                let at_right =         self.at(x + 1,   y       ).brightness();
                let at_bottom_left =   self.at(x - 1,   y + 1   ).brightness();
                let at_bottom_middle=  self.at(x,       y + 1   ).brightness();
                let at_bottom_right =  self.at(x + 1,   y + 1   ).brightness();

                let grad_x = (S_X[0] * at_top_left as i32) + (S_X[1] * at_top_middle as i32) + (S_X[2] * at_top_right as i32)+
                                (S_X[3] * at_left as i32) + (S_X[4] * at_middle as i32) + (S_X[5] * at_right as i32)+
                                (S_X[6] * at_bottom_left as i32) + (S_X[7] * at_bottom_middle as i32) + (S_X[8] * at_bottom_right as i32);

                let grad_y = (S_Y[0] * at_top_left as i32) + (S_Y[1] * at_top_middle as i32) + (S_Y[2] * at_top_right as i32)+
                                (S_Y[3] * at_left as i32) + (S_Y[4] * at_middle as i32) + (S_Y[5] * at_right as i32)+
                                (S_Y[6] * at_bottom_left as i32) + (S_Y[7] * at_bottom_middle as i32) + (S_Y[8] * at_bottom_right as i32);    

                let gradient: u8 = ((grad_x * grad_x + grad_y * grad_y)as f32).sqrt() as u8 ; 
                *target.at_mut(x, y) = olc::Pixel::rgb(gradient, gradient, gradient);
            }                   
        }
    }

    pub fn sobel_edge_detection_3x3(&mut self, target: &mut Image)
    {
        let S_Y = 
        [2, 1,-2,
        1,-3, 1,
       -2, 1, 2];
   
        let S_X = 
        [-2, 1, 2,
        1,-3, 1,
        2, 1,-2];

        let S_X: [i32;9] = [1, 0, -1, 2, 0, -2, 1, 0, -1];
        let S_Y: [i32;9] = [1, 2, 1, 0, 0, 0, -1, -2, -1];
        for y in 0..self.height
        {
            for x in 0..self.width
            {
                if !(1..self.width-1).contains(&x)
                || !(1..self.height-1).contains(&y)
                {
                    *target.at_mut(x, y) = olc::BLACK;
                    continue;
                }
                let at_top_left =      self.at(x - 1,   y - 1   ).brightness();
                let at_top_middle =    self.at(x,       y - 1   ).brightness();
                let at_top_right =     self.at(x + 1,   y - 1   ).brightness();
                let at_left =          self.at(x - 1,   y       ).brightness();
                let at_middle =        self.at(x,       y       ).brightness();
                let at_right =         self.at(x + 1,   y       ).brightness();
                let at_bottom_left =   self.at(x - 1,   y + 1   ).brightness();
                let at_bottom_middle=  self.at(x,       y + 1   ).brightness();
                let at_bottom_right =  self.at(x + 1,   y + 1   ).brightness();

                let grad_x = (S_X[0] * at_top_left as i32) + (S_X[1] * at_top_middle as i32) + (S_X[2] * at_top_right as i32)+
                                (S_X[3] * at_left as i32) + (S_X[4] * at_middle as i32) + (S_X[5] * at_right as i32)+
                                (S_X[6] * at_bottom_left as i32) + (S_X[7] * at_bottom_middle as i32) + (S_X[8] * at_bottom_right as i32);

                let grad_y = (S_Y[0] * at_top_left as i32) + (S_Y[1] * at_top_middle as i32) + (S_Y[2] * at_top_right as i32)+
                                (S_Y[3] * at_left as i32) + (S_Y[4] * at_middle as i32) + (S_Y[5] * at_right as i32)+
                                (S_Y[6] * at_bottom_left as i32) + (S_Y[7] * at_bottom_middle as i32) + (S_Y[8] * at_bottom_right as i32);    

                let gradient: u8 = ((grad_x * grad_x + grad_y * grad_y)as f32).sqrt() as u8 ; 
                *target.at_mut(x, y) = olc::Pixel::rgb(gradient, gradient, gradient);
            }                   
        }
    }

    pub fn threshold(&mut self, target: &mut Image, threshold: u8)
    {
        for y in 0..self.height
        {
            for x in 0..self.width
            {
                let brt = self.at(x,y).brightness();
                *target.at_mut(x,y) = if brt >= threshold {olc::WHITE} else {olc::BLACK};
            }
        }
    }

    pub fn threshold_colour(&mut self, target: &mut Image, threshold: u8)
    {
        for y in 0..self.height
        {
            for x in 0..self.width
            {
                let p = self.at(x,y);
                target.at_mut(x,y).r = (p.r >= threshold) as u8 * 255;
                target.at_mut(x,y).g = (p.g >= threshold) as u8 * 255;
                target.at_mut(x,y).b = (p.b >= threshold) as u8 * 255;
            }
        }
    }
    pub fn floyd_steinberg_dithering(&mut self, target: &mut Image, bits_per_channel:usize)
    {
       let max_values_per_channel = if bits_per_channel > 8 {255} else{1 << bits_per_channel};

        for y in 0..self.height
        {
            for x in 0..self.width
            {
                *target.at_mut(x, y) = *self.at(x, y);
            }
        }
        for y in 1..self.height-1
        {
            for x in 1..self.width-1
            {
                let old_pixel = *target.at(x,y);
                
                let factor = 255/max_values_per_channel;

                let new_r = ((old_pixel.r / factor) * (factor)) as u8;
                let new_g = ((old_pixel.g / factor) * (factor)) as u8;
                let new_b = ((old_pixel.b / factor) * (factor)) as u8;
                *target.at_mut(x, y) = olc::Pixel::rgb(new_r, new_g, new_b);

                let error_r = old_pixel.r as i32 - new_r as i32;
                let error_g = old_pixel.g as i32 - new_g as i32;
                let error_b = old_pixel.b as i32 - new_b as i32;
                
                let add = |factor:i32, error:i32| error as f32 * factor as f32 /16.0;
                let mut pixel = *target.at(x+1, y);
                pixel.r += add(7,error_r) as u8;
                pixel.g += add(7,error_g) as u8;
                pixel.b += add(7,error_b) as u8;
                *target.at_mut(x+1, y) = pixel;

                pixel = *target.at(x-1, y+1);
                pixel.r += add(3,error_r) as u8;
                pixel.g += add(3,error_g) as u8;
                pixel.b += add(3,error_b) as u8;
                *target.at_mut(x-1, y+1) = pixel;

                pixel = *target.at(x, y+1);
                pixel.r += add(5,error_r) as u8;
                pixel.g += add(5,error_g) as u8;
                pixel.b += add(5,error_b) as u8;
                *target.at_mut(x, y+1) = pixel;

                pixel = *target.at(x+1, y+1);
                pixel.r += add(1,error_r) as u8;
                pixel.g += add(1,error_g) as u8;
                pixel.b += add(1,error_b) as u8;
                *target.at_mut(x+1, y+1) = pixel;
                
            }
        }
        
    }
    pub fn gaussian_blur_3x3(&mut self, target: &mut Image)
    {
        self.convolve(target, 3, |s, (x,y)| [1./16., 1./8., 1./16., 1./8., 1./4., 1./8., 1./16., 1./8., 1./16.][y*s+x]);
    }

    pub fn box_blur(&mut self, target: &mut Image, kernel_size: usize)
    {
        self.convolve(target, kernel_size, |s, (_x, _y)| 1.0/ (s * s ) as f32);
    }

    pub fn sobel_edge_detection_3x3_colour(&mut self, target: &mut Image)
    {
        const S_X: [i32;9] = [1,0,-1,2,0,-2,1,0,-1];
        const S_Y: [i32;9]  = [1,2,1,0,0,0,-1,-2,-1];
        
        for y in 0..self.height
        {
            for x in 0..self.width
            {
                if !(1..self.width-1).contains(&x)
                || !(1..self.height-1).contains(&y)
                {
                    *target.at_mut(x, y) = olc::BLACK;
                    continue;
                }
                let mut rx = 0;
                let mut gx = 0;
                let mut bx = 0;
                let mut ry = 0;
                let mut gy = 0;
                let mut by = 0;
                for kernel_y in 0..3
                {
                    for kernel_x in 0..3
                    {
                        let ix = x + kernel_x - 1;
                        let iy = y + kernel_y - 1;
                        let ik = kernel_y * 3 + kernel_x;
                        let current_pixel = *self.at(ix,iy);
                        rx += current_pixel.r as i32 * S_X[ik];
                        gx += current_pixel.g as i32 * S_X[ik];
                        bx += current_pixel.b as i32 * S_X[ik];
                        ry += current_pixel.r as i32 * S_Y[ik];
                        gy += current_pixel.g as i32 * S_Y[ik];
                        by += current_pixel.b as i32 * S_Y[ik];
                    }
                }

                let r = ((rx*rx + ry*ry) as f32).sqrt() as u8;
                let g = ((gx*gx + gy*gy) as f32).sqrt() as u8;
                let b = ((bx*bx + by*by) as f32).sqrt() as u8;
                *target.at_mut(x, y) = olc::Pixel::rgb(r, g, b);
            }                   
        }
    }

    pub fn cross_blur(&mut self, target: &mut Image)
    {
        let S_Y =  [   -1, 1,-1,
                                 1, 0, 1,
                                -1, 1, -1];
        let S_X = [   1, -1, 1,
                               -1, 1, -1,
                                1, -1,1];
        
        for y in 1..self.height - 1
        {
            for x in 1..self.width - 1
            {
                let at_top_left =      self.at(x - 1,   y - 1   ).brightness();
                let at_top_middle =    self.at(x,       y - 1   ).brightness();
                let at_top_right =     self.at(x + 1,   y - 1   ).brightness();
                let at_left =          self.at(x - 1,   y       ).brightness();
                let at_middle =        self.at(x,       y       ).brightness();
                let at_right =         self.at(x + 1,   y       ).brightness();
                let at_bottom_left =   self.at(x - 1,   y + 1   ).brightness();
                let at_bottom_middle=  self.at(x,       y + 1   ).brightness();
                let at_bottom_right =  self.at(x + 1,   y + 1   ).brightness();

                let grad_x= (S_X[0] * at_top_left as i32) + (S_X[1] * at_top_middle as i32) + (S_X[2] * at_top_right as i32)+
                                (S_X[3] * at_left as i32) + (S_X[4] * at_middle as i32) + (S_X[5] * at_right as i32)+
                                (S_X[6] * at_bottom_left as i32) + (S_X[7] * at_bottom_middle as i32) + (S_X[8] * at_bottom_right as i32);

                let grad_y = (S_Y[0] * at_top_left as i32) + (S_Y[1] * at_top_middle as i32) + (S_Y[2] * at_top_right as i32)+
                                (S_Y[3] * at_left as i32) + (S_Y[4] * at_middle as i32) + (S_Y[5] * at_right as i32)+
                                (S_Y[6] * at_bottom_left as i32) + (S_Y[7] * at_bottom_middle as i32) + (S_Y[8] * at_bottom_right as i32);    

                let gradient: u8 = ((grad_x * grad_x + grad_y * grad_y) as f32).sqrt() as u8; 
                *target.at_mut(x, y) = olc::Pixel::rgb(gradient, gradient, gradient);
            }
        }
    }
}