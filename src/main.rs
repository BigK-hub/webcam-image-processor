use olc_pge as olc;
use fastrand;
use camera_capture;
use lerp::Lerp;
use std::ops::Div;
use std::string;
use png;

#[derive(Clone)]
struct Image
{
    width:usize,
    height:usize,
    pixels: Vec<olc::Pixel>
}

trait Illuminator
{
    type Output;
    fn brightness(&self) -> Self::Output;
}

impl Illuminator for olc::Pixel
{
    type Output = u8; 
    fn brightness(&self) -> Self::Output
    {
        let mut value = 0;
        value += self.r as u32 * 299 / 1000;
        value += self.g as u32 * 587 / 1000;
        value += self.b as u32 * 114 / 1000;
        value as u8
        /*
            let mut value = self.r as f32 * 0.299;
            value += self.g as f32 * 0.587;
            value += self.b as f32 * 0.114;
            value as u8
        */
    }
}

impl Image
{
    fn at(&self, x: usize, y: usize) -> &olc::Pixel
    {
        if x >= self.width || y >= self.height
        {
            panic!("in function at() of Image, pixel coordinates exceed image dimensions.");
        }
        &self.pixels[y*self.width+x]
    }

    fn at_mut(&mut self, x: usize, y: usize) -> &mut olc::Pixel
    {
        if x >= self.width || y >= self.height
        {
            panic!("in function at() of Image, pixel coordinates exceed image dimensions.");
        }
        &mut self.pixels[y*self.width+x]
    }

    fn convolve<F>(&mut self, target: &mut Image, kernel_size: usize, mut kernel_generator: F) where F: FnMut(usize, (usize, usize)) -> f32
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

    fn painting(&self, target: &mut Image)
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

    fn sobel_edge_detection_3x3(&mut self, target: &mut Image)
    {   
        let S_Y =  [   -1, 1,-1,
                                 1, 0, 1,
                                -1, 1, -1];
        let S_X = [   1, -1, 1,
                               -1, 1, -1,
                                1, -1,1];
        
        for y in 1..self.height - 1
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

                let grad_x =
                (S_X[0] * at_top_left as i32) +
                (S_X[1] * at_top_middle as i32) +
                (S_X[2] * at_top_right as i32) +
                (S_X[3] * at_left as i32) +
                (S_X[4] * at_middle as i32) +
                (S_X[5] * at_right as i32) +
                (S_X[6] * at_bottom_left as i32) +
                (S_X[7] * at_bottom_middle as i32) +
                (S_X[8] * at_bottom_right as i32);

                let grad_y =
                (S_Y[0] * at_top_left as i32) +
                (S_Y[1] * at_top_middle as i32) +
                (S_Y[2] * at_top_right as i32) +
                (S_Y[3] * at_left as i32) +
                (S_Y[4] * at_middle as i32) +
                (S_Y[5] * at_right as i32) +
                (S_Y[6] * at_bottom_left as i32) +
                (S_Y[7] * at_bottom_middle as i32) +
                (S_Y[8] * at_bottom_right as i32);    

                let gradient: u8 = ((grad_x * grad_x + grad_y * grad_y)as f32).sqrt() as u8 ; 
                *target.at_mut(x, y) = olc::Pixel::rgb(gradient, gradient, gradient);
            }                   
        }
    }

    fn threshold(&mut self, target: &mut Image, threshold: u8)
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

    fn threshold_colour(&mut self, target: &mut Image, threshold: u8)
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

    fn gaussian_blur_3x3(&mut self, target: &mut Image)
    {
        self.convolve(target, 3, |s, (x,y)| [1./16., 1./8., 1./16., 1./8., 1./4., 1./8., 1./16., 1./8., 1./16.][y*s+x]);
    }

    fn box_blur(&mut self, target: &mut Image, kernel_size: usize)
    {
        self.convolve(target, kernel_size, |s, (_x, _y)| 1.0/ (s * s ) as f32);
    }

    fn sobel_edge_detection_3x3_colour(&mut self, target: &mut Image)
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

    fn cross_blur(&mut self, target: &mut Image)
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

#[derive(PartialEq)]
enum Mode
{
    Normal,
    TimeBlend,
    Sobel,
    SobelColour,
    Threshold,
    GaussianBlur,
    BoxBlur,
    Painting,
    CrossBlur,
}

struct Window
{
    cam_iter: camera_capture::ImageIterator,
    counter: u32,
    mode: Mode,
    frame: Image,
    target: Image,
    temp: Image,
}

impl olc::PGEApplication for Window
{
    const APP_NAME: &'static str = "mhm aha";
    fn on_user_create(&mut self, pge: &mut olc::PixelGameEngine) -> bool
    {
        true
    }
    fn on_user_update(&mut self, pge: &mut olc::PixelGameEngine, delta: f32) -> bool
    {
        let img = self.cam_iter.next().unwrap();
        let fraction = (5,10);
        for (i, pixel) in img.pixels().enumerate()
        {
            let p = olc::Pixel::rgb
            (
                pixel.data[0],
                pixel.data[1],
                pixel.data[2],
            );
            self.frame.pixels[i] = p;
        }
    
        //process frame
        match self.mode
        {
            Mode::Normal => (),
            Mode::Sobel => self.frame.sobel_edge_detection_3x3(&mut self.target),
            Mode::SobelColour => self.frame.sobel_edge_detection_3x3_colour(&mut self.target),
            Mode::TimeBlend => unimplemented!(),
            Mode::Threshold => self.frame.threshold(&mut self.target, pge.get_mouse_x() as u8 / 3),
            Mode::GaussianBlur => self.frame.gaussian_blur_3x3(&mut self.target),
            Mode::BoxBlur => self.frame.box_blur(&mut self.target, 5),
            Mode::Painting => self.frame.painting(&mut self.target),
            Mode::CrossBlur => self.frame.cross_blur(&mut self.target),
        };

        if pge.get_key(olc::Key::S).pressed
        {
            let path = String::from("image") + &self.counter.to_string() + ".png";
            let file = std::fs::File::create(std::path::Path::new(&path)).unwrap();
            self.counter += 1;
            let ref mut w = std::io::BufWriter::new(file);
            let mut encoder = png::Encoder::new(w, self.frame.width as u32, self.frame.height as u32); 
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            encoder.set_source_gamma(png::ScaledFloat::from_scaled(45455)); // 1.0 / 2.2, scaled by 100000
            encoder.set_source_gamma(png::ScaledFloat::new(1.0 / 2.2));     // 1.0 / 2.2, unscaled, but rounded
            let source_chromaticities = png::SourceChromaticities::new
            (     // Using unscaled instantiation here
                    (0.31270, 0.32900),
                    (0.64000, 0.33000),
                    (0.30000, 0.60000),
                    (0.15000, 0.06000)
            );
            encoder.set_source_chromaticities(source_chromaticities);
            let mut writer = encoder.write_header().unwrap();
            writer.write_image_data(&self.target.pixels.iter().map(|p| [p.r,p.g,p.b,p.a]).flatten().collect::<Vec<u8>>()).unwrap();
        }

        for y in 0..pge.screen_height()
        {
            for x in 0..pge.screen_width()
            {
                pge.draw(x as i32,y as i32, *self.target.at(x,y));
            }
        }
        true
    }
}

fn main()
{
    let cam = camera_capture::create(0).unwrap();
    let mut cam_iter = cam.fps(30.0).unwrap().start().unwrap();
    let h = cam_iter.next().unwrap();
    let width = h.width() as usize / 2;
    let height = h.height() as usize / 2;
    
    let cam = camera_capture::create(0).unwrap();
    let cam_iter = cam.fps(30.0).unwrap().resolution(width as u32, height as u32).unwrap().start().unwrap();

    let mut pixels = Vec::with_capacity(width* height);
    pixels.resize(width*height, olc::MAGENTA);
    let frame = Image{width,height,pixels: pixels.clone()};
    let mode = Mode::Sobel;
    
    let window = Window{cam_iter, counter: 0, mode, target: frame.clone(), temp: frame.clone(), frame};
    olc::PixelGameEngine::construct(window, width, height, 4, 4).start();
}