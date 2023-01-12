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

    pub fn convolve<F>(&self, target: &mut Image, kernel_size: usize, mut kernel_generator: F, denominator: i32) where F: FnMut(usize, (usize, usize)) -> i32
    {
        if denominator == 0 {panic!("In function convolve of Image, denominator argument may not equal 0.");}
        for y in kernel_size/2..self.height - kernel_size/2
        {
            for x in kernel_size/2..self.width - kernel_size/2
            {
                let (mut r,mut g,mut b) = (0, 0, 0);
                for kernel_y in 0..kernel_size
                {
                    for kernel_x in 0..kernel_size
                    {
                        let kernel_value = kernel_generator(kernel_size, (kernel_x, kernel_y));
                        let pixel = *self.at(x - kernel_size/2 + kernel_x, y - kernel_size/2 + kernel_y);
                        r += pixel.r as i32 * kernel_value;
                        g += pixel.g as i32 * kernel_value;
                        b += pixel.b as i32 * kernel_value;
                    }
                }
                r /= denominator;
                g /= denominator;
                b /= denominator;
                let r = r.min(255).max(0) as u8;
                let g = g.min(255).max(0) as u8;
                let b = b.min(255).max(0) as u8;
                *target.at_mut(x, y) = olc::Pixel::rgb(r as u8, g as u8, b as u8);
            }
        }
    }

    pub fn handle_edges<F>(&self, target: &mut Image, kernel_size: usize, mut edge_handler: F) where F: FnMut(&Image, usize, (usize, usize)) -> olc::Pixel
    {
        for y in (0..kernel_size/2).chain(self.height - kernel_size/2 .. self.height)
        {
            for x in 0..self.width
            {
                let pixel = edge_handler(self, kernel_size, (x, y));
                *target.at_mut(x, y) = pixel;
            }
        }
        for y in 0..self.height
        {
            for x in (0..kernel_size/2).chain(self.width - kernel_size/2..self.width)
            {
                let pixel = edge_handler(self, kernel_size, (x, y));
                *target.at_mut(x, y) = pixel;
            }
        }
    }

    pub fn greyscale(&self, target: &mut Image)
    {
        for (i, &pixel) in self.pixels.iter().enumerate()
        {
            let brt = pixel.brightness();
            target.pixels[i] = olc::Pixel::rgb(brt, brt, brt);
        }
    }

    pub fn sharpen_alternative(&self, target: &mut Image)
    {
        let s_y = 
        [-1, 0,-1,
          0, 5, 0,
         -1, 0,-1];
   
        let s_x = 
        [0, -1, 0,
         -1 ,5, -1,
         0, -1, 0];

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
                let mut grad_y = 0;
                let mut grad_x = 0;
                for kernel_y in 0..3
                {
                    for kernel_x in 0..3
                    {
                        let kernel_value_x = s_x[kernel_y*3+kernel_x];
                        let kernel_value_y = s_y[kernel_y*3+kernel_x];
                        let value = self.at(x - 3/2 + kernel_x, y - 3/2 + kernel_y).brightness() as i32;
                        grad_x += value * kernel_value_x;
                        grad_y += value * kernel_value_y;
                    }
                }  

                //let gradient = ((grad_x * grad_x + grad_y * grad_y) as f32).sqrt() as u8;
                let gradient = ( (grad_x.abs() + grad_y.abs()) / 2 ).min(255).max(0) as u8;

                *target.at_mut(x, y) = olc::Pixel::rgb(gradient, gradient, gradient);
            }                   
        }
    }

    pub fn sharpen(&self, target: &mut Image)
    {
        let kernel = [0, -1, 0, -1, 5, -1, 0, -1, 0];

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
                let mut output = 0;
                for kernel_y in 0..3
                {
                    for kernel_x in 0..3
                    {
                        let kernel_value = kernel[kernel_y*3+kernel_x];
                        let brightness = self.at(x - 3/2 + kernel_x, y - 3/2 + kernel_y).brightness() as i32;
                        output += brightness * kernel_value;
                    }
                }

                let value = output.min(255).max(0) as u8;
                *target.at_mut(x, y) = olc::Pixel::rgb(value, value, value);
            }                   
        }
    }

    pub fn sharpen_colour(&self, target: &mut Image)
    {
        self.convolve(target, 3, |s,(x,y)| [0, -1, 0, -1, 5, -1, 0, -1, 0][y*s+x], 1);
        self.handle_edges(target, 3,
            |img, _s, (x,y)|
            *img.at(x,y)
        );
    }

    pub fn sobel_edge_detection_3x3(&self, target: &mut Image)
    {
        let s_x: [i32;9] = [1, 0, -1, 2, 0, -2, 1, 0, -1];
        let s_y: [i32;9] = [1, 2, 1, 0, 0, 0, -1, -2, -1];
        for y in 0..self.height
        {
            for x in 0..self.width
            {
                if !(1..self.width-1).contains(&x)
                || !(1..self.height-1).contains(&y)
                {
                    //handling edges here
                    *target.at_mut(x, y) = olc::BLACK;
                    continue;
                }
                let at_top_left =      self.at(x - 1,   y - 1   ).brightness() as i32;
                let at_top_middle =    self.at(x,       y - 1   ).brightness() as i32;
                let at_top_right =     self.at(x + 1,   y - 1   ).brightness() as i32;
                let at_left =          self.at(x - 1,   y       ).brightness() as i32;
                let at_middle =        self.at(x,       y       ).brightness() as i32;
                let at_right =         self.at(x + 1,   y       ).brightness() as i32;
                let at_bottom_left =   self.at(x - 1,   y + 1   ).brightness() as i32;
                let at_bottom_middle=  self.at(x,       y + 1   ).brightness() as i32;
                let at_bottom_right =  self.at(x + 1,   y + 1   ).brightness() as i32;

                let grad_x = 
                s_x[0] * at_top_left +
                s_x[1] * at_top_middle +
                s_x[2] * at_top_right +
                s_x[3] * at_left +
                s_x[4] * at_middle +
                s_x[5] * at_right +
                s_x[6] * at_bottom_left +
                s_x[7] * at_bottom_middle +
                s_x[8] * at_bottom_right;

                let grad_y = 
                s_y[0] * at_top_left +
                s_y[1] * at_top_middle +
                s_y[2] * at_top_right +
                s_y[3] * at_left + 
                s_y[4] * at_middle +
                s_y[5] * at_right +
                s_y[6] * at_bottom_left +
                s_y[7] * at_bottom_middle +
                s_y[8] * at_bottom_right;

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

    pub fn gaussian_blur_3x3(&self, target: &mut Image)
    {
        self.convolve(target, 3, |s, (x,y)|
            [
                1, 2, 1,
                2, 4, 2,
                1, 2, 1,
            ][y*s+x], 16
        );
        self.handle_edges(target, 3,
            |img, _s, (x,y)|
            *img.at(x,y)
        );
    }

    pub fn box_blur(&mut self, target: &mut Image, kernel_size: usize)
    {
        self.convolve(target, kernel_size, |s, (_x, _y)| 1, (kernel_size*kernel_size) as i32);
        self.handle_edges(target, kernel_size, 
            |img, _s, (x,y)|
            *img.at(x,y)
        );
    }

    pub fn sobel_edge_detection_3x3_colour(&mut self, target: &mut Image)
    {
        let s_x = [1,0,-1,2,0,-2,1,0,-1];
        let s_y = [1,2,1,0,0,0,-1,-2,-1];
        
        for y in 0..self.height
        {
            for x in 0..self.width
            {
                if !(1..self.width-1).contains(&x)
                || !(1..self.height-1).contains(&y)
                {
                    //handling edges here
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
                        rx += current_pixel.r as i32 * s_x[ik];
                        gx += current_pixel.g as i32 * s_x[ik];
                        bx += current_pixel.b as i32 * s_x[ik];
                        ry += current_pixel.r as i32 * s_y[ik];
                        gy += current_pixel.g as i32 * s_y[ik];
                        by += current_pixel.b as i32 * s_y[ik];
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
        let s_y =  [   -1, 1,-1,
                                 1, 0, 1,
                                -1, 1, -1];
        let s_x = [   1, -1, 1,
                               -1, 1, -1,
                                1, -1,1];
        
        for y in 1..self.height - 1
        {
            for x in 1..self.width - 1
            {
                let at_top_left =      self.at(x - 1,   y - 1   ).brightness() as i32;
                let at_top_middle =    self.at(x,       y - 1   ).brightness() as i32;
                let at_top_right =     self.at(x + 1,   y - 1   ).brightness() as i32;
                let at_left =          self.at(x - 1,   y       ).brightness() as i32;
                let at_middle =        self.at(x,       y       ).brightness() as i32;
                let at_right =         self.at(x + 1,   y       ).brightness() as i32;
                let at_bottom_left =   self.at(x - 1,   y + 1   ).brightness() as i32;
                let at_bottom_middle=  self.at(x,       y + 1   ).brightness() as i32;
                let at_bottom_right =  self.at(x + 1,   y + 1   ).brightness() as i32;

                let grad_x = 
                s_x[0] * at_top_left +
                s_x[1] * at_top_middle +
                s_x[2] * at_top_right +
                s_x[3] * at_left +
                s_x[4] * at_middle +
                s_x[5] * at_right +
                s_x[6] * at_bottom_left +
                s_x[7] * at_bottom_middle +
                s_x[8] * at_bottom_right;

                let grad_y = 
                s_y[0] * at_top_left +
                s_y[1] * at_top_middle +
                s_y[2] * at_top_right +
                s_y[3] * at_left + 
                s_y[4] * at_middle +
                s_y[5] * at_right +
                s_y[6] * at_bottom_left +
                s_y[7] * at_bottom_middle +
                s_y[8] * at_bottom_right;

                let gradient: u8 = ((grad_x * grad_x + grad_y * grad_y) as f32).sqrt() as u8; 
                *target.at_mut(x, y) = olc::Pixel::rgb(gradient, gradient, gradient);
            }
        }
    }
}