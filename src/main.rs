pub mod pixel_traits;
pub mod image;
use image::Image;
use olc_pge as olc;
use camera_capture;

fn main()
{
    let width = 500;
    let height = width * 9 /16;
    
    let cam = camera_capture::create(0).unwrap();
    let cam_iter = cam.fps(30.0).unwrap().resolution(width as u32, height as u32).unwrap().start().unwrap();

    let pixels = (0..width*height).map(|_x| olc::MAGENTA).collect::<Vec<olc::Pixel>>();
    let frame = Image{width,height, pixels};
    
    let mode = Mode::FloydSteinbergDithering;
    
    let window = Window
    {
        cam_iter,
        counter: 0,
        mode, target: frame.clone(),
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
    FloydSteinbergDithering,
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
    _temp: Image, //remove underscore when you actually need this
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
        // fraction.0 is proportional to the influence of the next frame
        let fraction = if self.mode == Mode::TimeBlend {(5, 10)} else {(10, 10)};
        for (i, pixel) in self.cam_iter.next().unwrap().pixels().enumerate()
        {
            let p = self.frame.pixels[i];
            let mut r = p.r as u32;
            let mut g = p.g as u32;
            let mut b = p.b as u32;

            r *= fraction.1 - fraction.0;
            g *= fraction.1 - fraction.0;
            b *= fraction.1 - fraction.0;

            r += pixel.data[0] as u32 * (fraction.0);
            g += pixel.data[1] as u32 * (fraction.0);
            b += pixel.data[2] as u32 * (fraction.0);

            r /= fraction.1;
            g /= fraction.1;
            b /= fraction.1;

            self.frame.pixels[i] = olc::Pixel::rgb(r as u8,g as u8,b as u8);
        }
    
        //process frame
        match self.mode
        {
            Mode::Normal => std::mem::swap(&mut self.target, &mut self.frame),
            Mode::TimeBlend => std::mem::swap(&mut self.target, &mut self.frame),
            Mode::Sobel => self.frame.sobel_edge_detection_3x3(&mut self.target),
            Mode::SobelColour => self.frame.sobel_edge_detection_3x3_colour(&mut self.target),
            Mode::Threshold => self.frame.threshold(&mut self.target, pge.get_mouse_x() as u8 / 5),
            Mode::ThresholdColour => self.frame.threshold_colour(&mut self.target, pge.get_mouse_x() as u8 / 5),
            Mode::TimeBlend => unimplemented!(),
            Mode::Threshold => self.frame.threshold(&mut self.target, pge.get_mouse_x() as u8 / 3),
            Mode::FloydSteinbergDithering =>  self.frame.floyd_steinberg_dithering(&mut self.target, 1),
            Mode::GaussianBlur => self.frame.gaussian_blur_3x3(&mut self.target),
            Mode::BoxBlur => self.frame.box_blur(&mut self.target, 3),
            Mode::Painting => self.frame.painting(&mut self.target),
            Mode::CrossBlur => self.frame.cross_blur(&mut self.target),
        };

        if pge.get_key(olc::Key::M).pressed
        {
            // It is very important that Mode::CrossBlur remains the last mode in the Mode enum.
            // If that's not the case, pressing the M key will not go through all modes.
            self.mode = unsafe{std::mem::transmute::<u8, Mode>((self.mode.clone() as u8 + 1)% (Mode::CrossBlur as u8 + 1))};
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
            (   // Using unscaled instantiation here
                (0.31270, 0.32900),
                (0.64000, 0.33000),
                (0.30000, 0.60000),
                (0.15000, 0.06000)
            );
            encoder.set_source_chromaticities(source_chromaticities);
            let mut writer = encoder.write_header().unwrap();
            writer.write_image_data(&self.target.pixels.iter().map(|p| [p.r, p.g, p.b, p.a]).flatten().collect::<Vec<u8>>()).unwrap();
        }

        for y in 0..pge.screen_height()
        {
            for x in 0..pge.screen_width()
            {
                pge.draw(x as i32, y as i32, *self.target.at(x,y));
            }
        }
        true
    }
}
