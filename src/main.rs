pub mod image;
pub mod pixel_traits;
use image::Image;
use olc_pge as olc;
use camera_capture;

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

#[allow(dead_code)]
#[derive(PartialEq, Clone)]
enum Mode
{
    Normal,
    Sobel,
    SobelColour,
    Threshold,
    ThresholdColour,
    InterpolateColour,
    GaussianBlur,
    BoxBlur,
    Painting,
    CrossBlur,
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
        let img = self.cam_iter.next().unwrap();
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
            Mode::Normal => {std::mem::swap(&mut self.frame, &mut self.target);},
            Mode::Sobel => self.frame.sobel_edge_detection_3x3(&mut self.target),
            Mode::SobelColour => self.frame.sobel_edge_detection_3x3_colour(&mut self.target),
            Mode::Threshold => self.frame.threshold(&mut self.target, pge.get_mouse_x() as u8 / 3),
            Mode::ThresholdColour => self.frame.threshold_colour(&mut self.target, 100),
            Mode::InterpolateColour => self.frame.brightness_interpolation(&mut self.target, olc::Pixel::rgb(255, 200, 200), olc::Pixel::rgb(200, 255, 255)),
            Mode::GaussianBlur => self.frame.gaussian_blur_3x3(&mut self.target),
            Mode::BoxBlur => self.frame.box_blur(&mut self.target, 5),
            Mode::Painting => self.frame.painting(&mut self.target),
            Mode::CrossBlur => self.frame.cross_blur(&mut self.target),
        };

        if pge.get_key(olc::Key::M).pressed
        {
            unsafe
            {
                self.mode = std::mem::transmute::<u8, Mode>((self.mode.clone() as u8 + 1) % 9);
            }
        }

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

struct Window
{
    cam_iter: camera_capture::ImageIterator,
    counter: u32,
    mode: Mode,
    frame: Image,
    target: Image,
    temp: Image,
}

