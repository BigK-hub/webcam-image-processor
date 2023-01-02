use olc_pge as olc;
use fastrand;
use camera_capture;
use lerp::Lerp;
use std::borrow::Borrow;
use std::ops::Div;
use std::string;
use png;
use std::path::Path;
use std::fs::File;
use std::io::BufWriter;



#[derive(Clone)]
struct Image
{
    width:usize,
    height:usize,
    pixels: Vec<olc::Pixel>
}

impl Image
{
    fn at(&mut self, x: usize, y: usize) -> &mut olc::Pixel
    {
        if x >= self.width || y >= self.height
        {
            panic!("in function at() of Image, pixel coordinates exceed image dimensions.");
        }
        &mut self.pixels[y*self.width+x]
    }
}

#[derive(PartialEq)]
enum Mode
{
    Normal,
    TimeBlend,
    Sobel,
    FonkySobel,
    Threshold,
}

struct Window
{
    cam_iter: camera_capture::ImageIterator,
    mode: Mode,
    frame: Image,
    target: Image,
    temp: Image,
    counter: u32,
}
impl Window
{
    /// writes to target
    fn painting(&mut self)
    {
        let S_Y = 
        [2, 1,-2,
        1,-3, 1,
       -2, 1, 2];
   
        let S_X = 
        [-2, 1, 2,
        1,-3, 1,
        2, 1,-2];
        
        for y in 1..self.frame.height - 1
        {
            for x in 1..self.frame.width - 1
            {
                let at_top_left =      *self.frame.at(x - 1,   y - 1   );
                let at_top_middle =    *self.frame.at(x,       y - 1   );
                let at_top_right =     *self.frame.at(x + 1,   y - 1   );
                let at_left =          *self.frame.at(x - 1,   y       );
                let at_middle =        *self.frame.at(x,       y       );
                let at_right =         *self.frame.at(x + 1,   y       );
                let at_bottom_left =   *self.frame.at(x - 1,   y + 1   );
                let at_bottom_middle=  *self.frame.at(x,       y + 1   );
                let at_bottom_right =  *self.frame.at(x + 1,   y + 1   );

                let grad_x: f32 = (S_X[0] as f32 * at_top_left.r as f32) + (S_X[1] as f32 * at_top_middle.r as f32) + (S_X[2] as f32 * at_top_right.r as f32)+
                                (S_X[3] as f32 * at_left.r as f32) + (S_X[4] as f32 * at_middle.r as f32) + (S_X[5] as f32 * at_right.r as f32)+
                                (S_X[6] as f32 * at_bottom_left.r as f32) + (S_X[7] as f32 * at_bottom_middle.r as f32) + (S_X[8] as f32 * at_bottom_right.r as f32);

                let grad_y: f32 = (S_Y[0] as f32 * at_top_left.r as f32) + (S_Y[1] as f32 * at_top_middle.r as f32) + (S_Y[2] as f32 * at_top_right.r as f32)+
                                (S_Y[3] as f32 * at_left.r as f32) + (S_Y[4] as f32 * at_middle.r as f32) + (S_Y[5] as f32 * at_right.r as f32)+
                                (S_Y[6] as f32 * at_bottom_left.r as f32) + (S_Y[7] as f32 * at_bottom_middle.r as f32) + (S_Y[8] as f32 * at_bottom_right.r as f32);    

                let gradient: u8 = (grad_x * grad_x + grad_y * grad_y).sqrt() as u8 ; 
                *self.target.at(x, y) = olc::Pixel::rgb(gradient, gradient, gradient);
            }
        }
    }

    fn cross_blur(&mut self)
    {
        let S_Y =  [   -1, 1,-1,
                                 1, 0, 1,
                                -1, 1, -1];
        let S_X = [   1, -1, 1,
                               -1, 1, -1,
                                1, -1,1];
        
        for y in 1..self.frame.height - 1
        {
            for x in 1..self.frame.width - 1
            {
                let at_top_left =      *self.frame.at(x - 1,   y - 1   );
                let at_top_middle =    *self.frame.at(x,       y - 1   );
                let at_top_right =     *self.frame.at(x + 1,   y - 1   );
                let at_left =          *self.frame.at(x - 1,   y       );
                let at_middle =        *self.frame.at(x,       y       );
                let at_right =         *self.frame.at(x + 1,   y       );
                let at_bottom_left =   *self.frame.at(x - 1,   y + 1   );
                let at_bottom_middle=  *self.frame.at(x,       y + 1   );
                let at_bottom_right =  *self.frame.at(x + 1,   y + 1   );

                let grad_x: f32 = (S_X[0] as f32 * at_top_left.r as f32) + (S_X[1] as f32 * at_top_middle.r as f32) + (S_X[2] as f32 * at_top_right.r as f32)+
                                (S_X[3] as f32 * at_left.r as f32) + (S_X[4] as f32 * at_middle.r as f32) + (S_X[5] as f32 * at_right.r as f32)+
                                (S_X[6] as f32 * at_bottom_left.r as f32) + (S_X[7] as f32 * at_bottom_middle.r as f32) + (S_X[8] as f32 * at_bottom_right.r as f32);

                let grad_y: f32 = (S_Y[0] as f32 * at_top_left.r as f32) + (S_Y[1] as f32 * at_top_middle.r as f32) + (S_Y[2] as f32 * at_top_right.r as f32)+
                                (S_Y[3] as f32 * at_left.r as f32) + (S_Y[4] as f32 * at_middle.r as f32) + (S_Y[5] as f32 * at_right.r as f32)+
                                (S_Y[6] as f32 * at_bottom_left.r as f32) + (S_Y[7] as f32 * at_bottom_middle.r as f32) + (S_Y[8] as f32 * at_bottom_right.r as f32);    

                let gradient: u8 = (grad_x * grad_x + grad_y * grad_y).sqrt() as u8 ; 
                *self.temp.at(x, y) = olc::Pixel::rgb(gradient, gradient, gradient);
            }
        }
    }
    
    fn sobel_edge_detection_fonky(&mut self)
    {
        /*
        other kernel ideas:

            [    2,  1,  1,
                 1,  0, -1,
                -1, -1, -2];
            [    1, 1, 2,
                -1, 0, 1,
                 -2,-1,-1];

            ____________
            paimting
            [    2, 1,-2,
                 1,-3, 1,
                -2, 1, 2];
            [   -2, 1, 2,
                 1,-3, 1,
                 2, 1,-2];
        */
        
        let S_Y =  [   -1, 1,-1,
                                 1, 0, 1,
                                -1, 1, -1];
        let S_X = [   1, -1, 1,
                               -1, 1, -1,
                                1, -1,1];
        
        for y in 1..self.frame.height - 1
        {
            for x in 1..self.frame.width - 1
            {
                let at_top_left =      *self.frame.at(x - 1,   y - 1   );
                let at_top_middle =    *self.frame.at(x,       y - 1   );
                let at_top_right =     *self.frame.at(x + 1,   y - 1   );
                let at_left =          *self.frame.at(x - 1,   y       );
                let at_middle =        *self.frame.at(x,       y       );
                let at_right =         *self.frame.at(x + 1,   y       );
                let at_bottom_left =   *self.frame.at(x - 1,   y + 1   );
                let at_bottom_middle=  *self.frame.at(x,       y + 1   );
                let at_bottom_right =  *self.frame.at(x + 1,   y + 1   );

                let grad_x: f32 = (S_X[0] as f32 * at_top_left.r as f32) + (S_X[1] as f32 * at_top_middle.r as f32) + (S_X[2] as f32 * at_top_right.r as f32)+
                                (S_X[3] as f32 * at_left.r as f32) + (S_X[4] as f32 * at_middle.r as f32) + (S_X[5] as f32 * at_right.r as f32)+
                                (S_X[6] as f32 * at_bottom_left.r as f32) + (S_X[7] as f32 * at_bottom_middle.r as f32) + (S_X[8] as f32 * at_bottom_right.r as f32);

                let grad_y: f32 = (S_Y[0] as f32 * at_top_left.r as f32) + (S_Y[1] as f32 * at_top_middle.r as f32) + (S_Y[2] as f32 * at_top_right.r as f32)+
                                (S_Y[3] as f32 * at_left.r as f32) + (S_Y[4] as f32 * at_middle.r as f32) + (S_Y[5] as f32 * at_right.r as f32)+
                                (S_Y[6] as f32 * at_bottom_left.r as f32) + (S_Y[7] as f32 * at_bottom_middle.r as f32) + (S_Y[8] as f32 * at_bottom_right.r as f32);    

                let gradient: u8 = (grad_x * grad_x + grad_y * grad_y).sqrt() as u8 ; 
                *self.target.at(x, y) = olc::Pixel::rgb(gradient, gradient, gradient);
                // let num = fastrand::u8(0..3);
                // if num == 0
                // {
                //     *self.target.at(x, y) = olc::Pixel::rgb(gradient, gradient/n.max(1), gradient/n.max(1));
                // }
                // else if num == 1
                // {
                //     *self.target.at(x, y) = olc::Pixel::rgb(gradient/n.max(1), gradient, gradient/n.max(1));
                // }
                // else if num == 2
                // {
                //     *self.target.at(x, y) = olc::Pixel::rgb(gradient/n.max(1), gradient/n.max(1), gradient);
                // }
            }                   
        }
    }

    /// temp -= target
    fn subtract(&mut self)
    {
        for y in 0 .. self.frame.height
        {
            for x in 0 .. self.frame.width
            {
                self.temp.at(x, y).r = (self.temp.at(x, y).r as i16 - self.target.at(x, y).r as i16).max(0) as u8;
                self.temp.at(x, y).g = (self.temp.at(x, y).g as i16 - self.target.at(x, y).g as i16).max(0) as u8;
                self.temp.at(x, y).b = (self.temp.at(x, y).b as i16 - self.target.at(x, y).b as i16).max(0) as u8;
            }
        }
        //std::mem::swap(&mut self.frame, &mut self.target);
    }
    /// target += frame * frac
    fn add_frame(&mut self,fraction:(i16,i16))
    {
        for y in 0 .. self.frame.height
        {
            for x in 0 .. self.frame.width
            {
                self.target.at(x, y).r = (self.frame.at(x, y).r as i16 * fraction.0 /fraction.1  + self.target.at(x, y).r as i16).max(0) as u8;
                self.target.at(x, y).g = (self.frame.at(x, y).g as i16 * fraction.0 /fraction.1  + self.target.at(x, y).g as i16).max(0) as u8;
                self.target.at(x, y).b = (self.frame.at(x, y).b as i16 * fraction.0 /fraction.1  + self.target.at(x, y).b as i16).max(0) as u8;
            }
        }
        //std::mem::swap(&mut self.frame, &mut self.target);
    }
    /// target += temp * frac
    fn add_temp(&mut self,fraction:(i16,i16))
    {
        for y in 0 .. self.frame.height
        {
            for x in 0 .. self.frame.width
            {
                self.target.at(x, y).r = (self.temp.at(x, y).r as i16 * fraction.0 /fraction.1  + self.target.at(x, y).r as i16).max(0) as u8;
                self.target.at(x, y).g = (self.temp.at(x, y).g as i16 * fraction.0 /fraction.1  + self.target.at(x, y).g as i16).max(0) as u8;
                self.target.at(x, y).b = (self.temp.at(x, y).b as i16 * fraction.0 /fraction.1  + self.target.at(x, y).b as i16).max(0) as u8;
            }
        }
        //std::mem::swap(&mut self.frame, &mut self.target);
    }
    /// target *= frame
    fn mul(&mut self)
    {
        for y in 0 .. self.frame.height
        {
            for x in 0 .. self.frame.width
            {
                self.target.at(x, y).r = (self.frame.at(x, y).r as u16 * self.target.at(x, y).r as u16).div(255) as u8;
                self.target.at(x, y).g = (self.frame.at(x, y).g as u16 * self.target.at(x, y).g as u16).div(255) as u8;
                self.target.at(x, y).b = (self.frame.at(x, y).b as u16 * self.target.at(x, y).b as u16).div(255) as u8;
            }
        }
        //std::mem::swap(&mut self.frame, &mut self.target);
    }
    fn sobel_edge_detection_3x3(&mut self)
    {
        const S_X: [i32;9] = [1,0,-1,2,0,-2,1,0,-1];
        const S_Y: [i32;9]  = [1,2,1,0,0,0,-1,-2,-1];
        for y in 1..self.frame.height - 1
        {
            for x in 1..self.frame.width - 1
            {
                let at_top_left =      *self.frame.at(x - 1,   y - 1   );
                let at_top_middle =    *self.frame.at(x,       y - 1   );
                let at_top_right =     *self.frame.at(x + 1,   y - 1   );
                let at_left =          *self.frame.at(x - 1,   y       );
                let at_middle =        *self.frame.at(x,       y       );
                let at_right =         *self.frame.at(x + 1,   y       );
                let at_bottom_left =   *self.frame.at(x - 1,   y + 1   );
                let at_bottom_middle=  *self.frame.at(x,       y + 1   );
                let at_bottom_right =  *self.frame.at(x + 1,   y + 1   );

                let grad_x: f32 = (S_X[0] as f32 * at_top_left.r as f32) + (S_X[1] as f32 * at_top_middle.r as f32) + (S_X[2] as f32 * at_top_right.r as f32)+
                                (S_X[3] as f32 * at_left.r as f32) + (S_X[4] as f32 * at_middle.r as f32) + (S_X[5] as f32 * at_right.r as f32)+
                                (S_X[6] as f32 * at_bottom_left.r as f32) + (S_X[7] as f32 * at_bottom_middle.r as f32) + (S_X[8] as f32 * at_bottom_right.r as f32);

                let grad_y: f32 = (S_Y[0] as f32 * at_top_left.r as f32) + (S_Y[1] as f32 * at_top_middle.r as f32) + (S_Y[2] as f32 * at_top_right.r as f32)+
                                (S_Y[3] as f32 * at_left.r as f32) + (S_Y[4] as f32 * at_middle.r as f32) + (S_Y[5] as f32 * at_right.r as f32)+
                                (S_Y[6] as f32 * at_bottom_left.r as f32) + (S_Y[7] as f32 * at_bottom_middle.r as f32) + (S_Y[8] as f32 * at_bottom_right.r as f32);    

                let gradient: u8 = (grad_x * grad_x + grad_y * grad_y).sqrt() as u8 ; 
                *self.target.at(x, y) = olc::Pixel::rgb(gradient, gradient, gradient);
                
            }                   
        }
        
    }

    fn threshold(&mut self, threshold: u8,color_brt: olc::Pixel, color_drk: olc::Pixel)
    {
        for y in 0..self.frame.height
        {
            for x in 0..self.frame.width
            {
                let brightness = 
                    // (self.frame.at(x,y).r as f32 * 0.299) +
                    // (self.frame.at(x,y).g as f32 * 0.587) +
                    // (self.frame.at(x,y).b as f32 * 0.114);
                    self.frame.at(x,y).g as f32;
                let r = (color_drk.r as f32).lerp(color_brt.r as f32, brightness/255. );
                let g = (color_drk.g as f32).lerp(color_brt.g as f32, brightness/255.);
                let b =(color_drk.b as f32).lerp(color_brt.b as f32, brightness/255.); 

                *self.target.at(x,y) = olc::Pixel::rgb(r as u8, g as u8, b as u8);
            }
        }
    }

    fn to_array(image: &mut Image) -> Vec<u8> 
    {
        let mut data = Vec::new();
        for  pixel in &image.pixels
        {
            data.push(pixel.r);
            data.push(pixel.g);
            data.push(pixel.b);
            data.push(pixel.a);
        }
        return data;
    }
    // fn threshold(&mut self, threshold: u8)
    // {
    //     for y in 0..self.frame.height
    //     {
    //         for x in 0..self.frame.width
    //         {
    //             let brightness = 
    //                 (self.frame.at(x,y).r as f32 * 0.299) +
    //                 (self.frame.at(x,y).g as f32 * 0.587) +
    //                 (self.frame.at(x,y).b as f32 * 0.114);

    //             *self.target.at(x,y) = 
    //                 if brightness as u8 >= threshold
    //                 {olc::WHITE}
    //                 else
    //                 {olc::BLACK};
    //         }
    //     }
    // }

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
        let fraction = (7,10);
        if self.mode == Mode::TimeBlend
        {
            for (i, pixel) in img.pixels().enumerate()
            {
                let p = olc::Pixel::rgb
                (
                    ((pixel.data[0] as u32 * (fraction.1 - fraction.0) + self.frame.pixels[i].r as u32 * fraction.0) / fraction.1) as u8,
                    ((pixel.data[1] as u32 * (fraction.1 - fraction.0) + self.frame.pixels[i].g as u32 * fraction.0) / fraction.1) as u8,
                    ((pixel.data[2] as u32 * (fraction.1 - fraction.0) + self.frame.pixels[i].b as u32 * fraction.0) / fraction.1) as u8,
                );
                self.frame.pixels[i] = p;
            }
        }
        else
        {
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
        }
        //process frame
        match self.mode
        {
            Mode::Normal => (),
            Mode::Sobel => self.sobel_edge_detection_3x3(),
            Mode::FonkySobel => 
            {
                self.sobel_edge_detection_fonky();
                std::mem::swap(&mut self.target,&mut self.temp);
                self.painting();
                self.subtract();
                self.mul();
            },
            Mode::TimeBlend => (),
            Mode::Threshold => self.threshold(pge.get_mouse_x() as u8 / 3,olc::Pixel::rgb(255,100, 00), olc::Pixel::rgb(100,0, 200)),
        };
        
        if pge.get_key(olc::Key::S).pressed
        {
            let path = String::from("image") + &self.counter.to_string() + ".png";
            let file = File::create(Path::new(&path)).unwrap();
            self.counter += 1;
            let ref mut w = BufWriter::new(file);
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
            writer.write_image_data(&Window::to_array(&mut self.target)).unwrap();
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
    
    let counter = 0;
    let window = Window{cam_iter, mode, target: frame.clone(), temp: frame.clone(), frame, counter};
    olc::PixelGameEngine::construct(window, width, height, 4, 4).start();
}