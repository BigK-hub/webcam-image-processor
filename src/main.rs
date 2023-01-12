pub mod pixel_traits;
pub mod image;

use image::Image;
use olc_pge as olc;
use camera_capture;
use pixel_traits::*;

const MODE_NAMES: [&str; 11] = ["Normal", "Sobel", "SobelColour", "Threshold", "ThresholdColour", "GaussianBlur", "BoxBlur", "GreyScale", "Sharpen", "SharpenColour", "CrossBlur"];

fn main()
{
    let pixelsize = get_pixel_size_input();
    let width = 640/pixelsize;
    let height = width * 9 / 16;
    
    let cam = camera_capture::create(0).unwrap();
    let cam_iter = cam.fps(30.0).unwrap().resolution(width as u32, height as u32).unwrap().start().unwrap();

    let pixels = (0..width*height).map(|_x| olc::MAGENTA).collect::<Vec<olc::Pixel>>();
    let frame = Image{width,height, pixels};
    
    let slider = Slider
    {
        x: 5,
        y: 5,
        w: 50,
        h: 20,
        start_val: 0,
        end_val: (Processor::CrossBlur as u32),
        step_size: 1,
        current_val: Processor::Normal as u32,
    };

    let window = Window::new
    (
        cam_iter,
        slider,
        frame
    );
    olc::PixelGameEngine::construct(window, width, height, pixelsize*2, pixelsize*2).start();
}

fn get_pixel_size_input() -> usize
{
    println!("Please enter the pixel size you want. Pixel sizes larger than 1 come with decreased resolution (image looks pixelated).");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    let output = 
    loop
    {
        match input.trim().parse::<usize>()
        {
            Err(_e) => println!("Invalid input. Input must be a natural number between 1 and 32. Recommended values are: 1, 2, 4, and 8."),
            Ok(num) => if num <= 32 && num >= 1{break num} else{println!("Value was outside of permissible range. Enter a value between 1 and 32.");},
        };
        std::io::stdin().read_line(&mut input).unwrap();
    };
    return output;
}

#[allow(dead_code)]
#[derive(PartialEq, Clone, Copy)]
enum Processor
{
    Normal,
    Sobel,
    SobelColour,
    Threshold,
    ThresholdColour,
    FloydSteinbergDithering,
    GaussianBlur,
    BoxBlur,
    GreyScale,
    Sharpen,
    SharpenColour,
    CrossBlur,
}

#[allow(dead_code)]
#[derive(PartialEq, Clone, Copy)]
enum InputMode
{
    Normal, 
    TimeBlend,
    Denoising,
}

struct Slider
{
    x: i32, 
    y: i32,
    w: i32,
    h: i32,
    start_val: u32, 
    end_val: u32,
    step_size: u32,
    current_val: u32,
}

impl Slider
{
    fn get_value(&mut self, x: i32, y: i32) -> u32
    {
        if self.is_hovering(x, y)
        {
            //inside slider
            let delta_val = self.end_val as i32 - self.start_val as i32;
            self.current_val = (self.start_val as i32 + (((x - self.x) * delta_val / self.w) / self.step_size as i32 ) * self.step_size as i32) as u32;
        }
        return self.current_val;
    }

    fn is_hovering(&self, x: i32, y: i32) -> bool
    {
        let rightx = self.x + self.w;
        let bottomy = self.y + self.h;
        return x >= self.x && x <= rightx
        && y >= self.y && y <= bottomy;
    }

    fn get_slider_x(&self) -> i32
    {
        let delta_val = self.end_val as i32 - self.start_val as i32;
        self.current_val as i32 * self.w / delta_val + self.x 
    }
}

struct Window
{
    cam_iter: camera_capture::ImageIterator,
    slider: Slider,
    processors: Vec<Processor>,
    input_mode: InputMode,
    hide_ui: bool,
    frame: Image,
    target: Image,
    _temp: Image, //remove underscore when you actually need this
}

impl Window
{
    fn new(cam_iter: camera_capture::ImageIterator, slider: Slider, frame: Image) -> Self
    {
        Self
        {
            cam_iter,
            slider,
            processors: vec![Processor::Normal],
            input_mode: InputMode::Normal,
            hide_ui: false,
            target: frame.clone(),
            _temp: frame.clone(),
            frame
        }
    }

    fn pre_process_input(&mut self)
    {
        match self.input_mode
        {
            InputMode::Normal
            => 
                for (i, pixel) in self.cam_iter.next().unwrap().pixels().enumerate()
                {
                    self.frame.pixels[i] = olc::Pixel::rgb(pixel.data[0], pixel.data[1], pixel.data[2]);
                }
            ,

            InputMode::TimeBlend
            => 
                // fraction.0 is proportional to the influence of the next frame
                for (i, pixel) in self.cam_iter.next().unwrap().pixels().enumerate()
                {
                    let fraction = (2, 10);
                    let pa = self.frame.pixels[i];
                    let pb = olc::Pixel::rgb(pixel.data[0], pixel.data[1], pixel.data[2]);

                    let mut r = pa.r as u32 * fraction.1;
                    let mut g = pa.g as u32 * fraction.1;
                    let mut b = pa.b as u32 * fraction.1;

                    r -= pa.r as u32 * fraction.0;
                    g -= pa.g as u32 * fraction.0;
                    b -= pa.b as u32 * fraction.0;

                    r += pb.a as u32 * fraction.0;
                    g += pb.g as u32 * fraction.0;
                    b += pb.b as u32 * fraction.0;

                    r /= fraction.1;
                    g /= fraction.1;
                    b /= fraction.1;

                    self.frame.pixels[i] = olc::Pixel::rgb(r as u8, g as u8, b as u8);
                }
            ,

            InputMode::Denoising
            => 
                //denoising based on pixel difference between frames
                for (i, pixel) in self.cam_iter.next().unwrap().pixels().enumerate()
                {
                    let p = temporal_denoising(self.frame.pixels[i], olc::Pixel::rgb(pixel.data[0], pixel.data[1], pixel.data[2]));
                    self.frame.pixels[i] = p;
                }
            ,
        }
    }
}

impl olc::PGEApplication for Window
{
    const APP_NAME: &'static str = "[Webcam Image Processor] | Press [S] to save image. |";
    fn on_user_create(&mut self, _pge: &mut olc::PixelGameEngine) -> bool
    {
        true
    }
    fn on_user_update(&mut self, pge: &mut olc::PixelGameEngine, _delta: f32) -> bool
    {
        self.pre_process_input();
        
        for processor in &self.processors
        {
            //process frame
            match processor
            {
                Processor::Normal => self.target.pixels.copy_from_slice(&self.frame.pixels),
                Processor::Sobel => self.frame.sobel_edge_detection_3x3(&mut self.target),
                Processor::SobelColour => self.frame.sobel_edge_detection_3x3_colour(&mut self.target),
                Processor::Threshold => self.frame.threshold(&mut self.target, (pge.get_mouse_x()*255/ pge.screen_width() as i32) as u8),
                Processor::ThresholdColour => self.frame.threshold_colour(&mut self.target, (pge.get_mouse_x()*255/ pge.screen_width() as i32) as u8),
                Processor::FloydSteinbergDithering =>  self.frame.floyd_steinberg_dithering(&mut self.target, 1),
                Processor::GaussianBlur => self.frame.gaussian_blur_3x3(&mut self.target),
                Processor::BoxBlur => self.frame.box_blur(&mut self.target, (((pge.get_mouse_x()*255/ pge.screen_width() as i32 )/2)*2 + 1).min(51) as usize),
                Processor::GreyScale => self.frame.greyscale(&mut self.target),
                Processor::Sharpen => self.frame.sharpen(&mut self.target),
                Processor::SharpenColour => self.frame.sharpen_colour(&mut self.target),
                Processor::CrossBlur => self.frame.cross_blur(&mut self.target),
            };
        }

        if pge.get_mouse(0).held
        {
            let value = self.slider.get_value(pge.get_mouse_x(), pge.get_mouse_y());
            self.processors[0] = unsafe{std::mem::transmute::<u8, Processor>(value as u8)};
        }

        if pge.get_key(olc::Key::S).pressed
        {
            save_image_as_png(&self.target);
        }

        if pge.get_key(olc::Key::H).pressed
        {
            self.hide_ui ^= true;
        }

        if pge.get_key(olc::Key::Left).pressed
        {   
            let lower = self.processors[0] as i32 - 1;
            let val = if lower < 0 {Processor::CrossBlur as u8} else {lower as u8};
            self.processors[0] = unsafe{std::mem::transmute::<u8, Processor>(val)};
            self.slider.current_val = val as u32;
        }
        if pge.get_key(olc::Key::Right).pressed
        {
            let val = ( (self.processors[0] as i32 + 1) % (Processor::CrossBlur as i32 + 1) ) as u8;
            self.processors[0] = unsafe{std::mem::transmute::<u8, Processor>(val)};
            self.slider.current_val = val as u32;
        }
        if pge.get_key(olc::Key::Down).pressed
        {
            let lower = self.input_mode as i32 - 1;
            let val = if lower < 0 {InputMode::Denoising as u8} else {lower as u8};
            self.input_mode = unsafe{std::mem::transmute::<u8, InputMode>(val)};
        }
        if pge.get_key(olc::Key::Up).pressed
        {
            let val = ( (self.input_mode as i32 + 1) % (InputMode::Denoising as i32 + 1) ) as u8;
            self.input_mode = unsafe{std::mem::transmute::<u8, InputMode>(val)};
        }

        for y in 0..pge.screen_height()
        {
            for x in 0..pge.screen_width()
            {
                pge.draw(x as i32, y as i32, *self.target.at(x,y));
            }
        }
        if !self.hide_ui
        {
            pge.fill_rect(self.slider.x + 2, self.slider.y, self.slider.w as u32, self.slider.h as u32, olc::Pixel::rgb(70, 150, 140));
            pge.fill_rect(self.slider.get_slider_x(), self.slider.y, 2, self.slider.h as u32, olc::Pixel::rgb(200, 235, 225));
            pge.draw_string(5, pge.screen_height() as i32 - 10, &MODE_NAMES[self.processors[0] as usize].to_string(), olc::WHITE);
        }
        true
    }
}

fn save_image_as_png(image: &Image)
{
    let pathstring = String::from("image_") + &format!("{:x}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_micros()) + ".png";
    let file = std::fs::File::create(std::path::Path::new(&pathstring)).unwrap();
    let ref mut w = std::io::BufWriter::new(file);
    let mut encoder = png::Encoder::new(w, image.width as u32, image.height as u32); 
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_source_gamma(png::ScaledFloat::from_scaled(45455)); // 1.0 / 2.2, scaled by 100000
    encoder.set_source_gamma(png::ScaledFloat::new(1.0 / 2.2));     // 1.0 / 2.2, unscaled, but rounded
    let source_chromaticities = png::SourceChromaticities::new
    (   // Using unscaled instantiation here
        (0.31270, 0.32900),
        (0.64000, 0.33000),
        (0.30000, 0.60000),
        (0.15000, 0.06000)
    );
    encoder.set_source_chromaticities(source_chromaticities);
    let mut writer = encoder.write_header().unwrap();
    writer.write_image_data(&image.pixels.iter().map(|p| [p.r, p.g, p.b, p.a]).flatten().collect::<Vec<u8>>()).unwrap();
}