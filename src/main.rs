pub mod pixel_traits;
pub mod image;
use image::Image;
use olc_pge as olc;
use camera_capture;
use pixel_traits::Illuminator;

fn main()
{
    let width = 256;
    let height = width * 9 / 16;
    
    let cam = camera_capture::create(0).unwrap();
    let cam_iter = cam.fps(60.0).unwrap().resolution(width as u32, height as u32).unwrap().start().unwrap();

    let pixels = (0..width*height).map(|_x| olc::MAGENTA).collect::<Vec<olc::Pixel>>();
    let frame = Image{width,height, pixels};
    
    let processors = vec![Mode::Sharpen];
    
    let slider = Slider
    {
        x: 5,
        y: 5,
        w: 50,
        h: 20,
        start_val: 0,
        end_val: (Mode::CrossBlur as u32),
        step_size: 1,
        current_val: 1,
    };

    let window = Window
    {
        cam_iter,
        processors,
        slider,
        target: frame.clone(),
        _temp: frame.clone(),
        frame
    };
    olc::PixelGameEngine::construct(window, width, height, 4, 4).start();
}

#[allow(dead_code)]
#[derive(PartialEq, Clone, Copy)]
enum Mode
{
    Normal,
    TimeBlend,
    Sobel,
    SobelColour,
    Threshold,
    ThresholdColour,
    GaussianBlur,
    BoxBlur,
    GreyScale,
    Sharpen,
    SharpenColour,
    CrossBlur,
}

struct Window
{
    cam_iter: camera_capture::ImageIterator,
    slider: Slider,
    processors: Vec<Mode>,
    frame: Image,
    target: Image,
    _temp: Image, //remove underscore when you actually need this
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

fn save_image_as_png(image: &Image)
{
    let mut image_hash = String::new();
    for i in 0..10
    {
        let c = image.pixels[i*i].brightness();
        if (c as char).is_alphanumeric()
        {
            image_hash.push(c as char);
        }
        image_hash.push(((c) % 26 + 97) as char);
    }
    let pathstring = String::from("image") + &image_hash + ".png";
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

impl olc::PGEApplication for Window
{
    const APP_NAME: &'static str = "mhm aha";
    fn on_user_create(&mut self, _pge: &mut olc::PixelGameEngine) -> bool
    {
        true
    }
    fn on_user_update(&mut self, pge: &mut olc::PixelGameEngine, _delta: f32) -> bool
    {
        for processor in &self.processors
        {
            // fraction.0 is proportional to the influence of the next frame
            let fraction = if *processor == Mode::TimeBlend||true {(1, 10)} else {(10, 10)};
            for (i, pixel) in self.cam_iter.next().unwrap().pixels().enumerate()
            {
                let p = self.frame.pixels[i];
                let mut r = p.r as u32 * fraction.1 - p.r as u32 * fraction.0 + pixel.data[0] as u32 * fraction.0;
                let mut g = p.g as u32 * fraction.1 - p.g as u32 * fraction.0 + pixel.data[1] as u32 * fraction.0;
                let mut b = p.b as u32 * fraction.1 - p.b as u32 * fraction.0 + pixel.data[2] as u32 * fraction.0;

                r /= fraction.1;
                g /= fraction.1;
                b /= fraction.1;

                self.frame.pixels[i] = olc::Pixel::rgb(r as u8, g as u8, b as u8);
            }
        
            //process frame
            match processor
            {
                Mode::Normal => self.target = self.frame.clone(),
                Mode::TimeBlend => self.target = self.frame.clone(),
                Mode::Sobel => self.frame.sobel_edge_detection_3x3(&mut self.target),
                Mode::SobelColour => self.frame.sobel_edge_detection_3x3_colour(&mut self.target),
                Mode::Threshold => self.frame.threshold(&mut self.target, pge.get_mouse_x() as u8),
                Mode::ThresholdColour => self.frame.threshold_colour(&mut self.target, pge.get_mouse_x() as u8),
                Mode::GaussianBlur => self.frame.gaussian_blur_3x3(&mut self.target),
                Mode::BoxBlur => self.frame.box_blur(&mut self.target, 5),
                Mode::GreyScale => self.frame.greyscale(&mut self.target),
                Mode::Sharpen => self.frame.sharpen(&mut self.target),
                Mode::SharpenColour => self.frame.sharpen_colour(&mut self.target),
                Mode::CrossBlur => self.frame.cross_blur(&mut self.target),
            };
        }

        if pge.get_mouse(0).held
        {
            let value = self.slider.get_value(pge.get_mouse_x(), pge.get_mouse_y());
            self.processors[0] = unsafe{std::mem::transmute::<u8, Mode>(value as u8)};
        }

        if pge.get_key(olc::Key::S).pressed
        {
            save_image_as_png(& self.target);
        }

        for y in 0..pge.screen_height()
        {
            for x in 0..pge.screen_width()
            {
                pge.draw(x as i32, y as i32, *self.target.at(x,y));
            }
        }
        pge.fill_rect(self.slider.x + 2, self.slider.y, self.slider.w as u32, self.slider.h as u32, olc::Pixel::rgb(70, 150, 140));
        pge.fill_rect(self.slider.get_slider_x(), self.slider.y, 2, self.slider.h as u32, olc::Pixel::rgb(200, 235, 225));
        true
    }
}
