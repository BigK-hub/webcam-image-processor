pub mod pixel_traits;
pub mod image;
use pixel_traits::*;
use image::Image;
use camera_capture;

use pixel_engine as px;
use px::inputs::Keycodes;
use px::Color;
use px::traits::*;
use px::vector2::*;

fn main()
{
    let pixelsize = get_pixel_size_input();
    let width = 640/pixelsize;
    let height = width * 9 / 16;
    
    let cam = camera_capture::create(0).unwrap();
    let mut cam_iter = cam.fps(17.0).unwrap().resolution(width as u32, height as u32).unwrap().start().unwrap();

    let pixels = (0..width*height).map(|_x| Color::new(255,50,255)).collect::<Vec<Color>>();

    let mut processors: Vec<Processor> = vec![Processor::Normal];
    let mut input_mode: InputMode = InputMode::Normal;
    let mut frame_time: std::time::Duration = std::time::Duration::from_millis(0);
    let mut hide_ui: bool = false;
    let mut frame: Image = Image{width, height, pixels};
    let mut target: Image = frame.clone();
    let mut temp: Image = frame.clone();

    let mut slider = Slider
    {
        pos: Vu2d::from([5,5]),
        size: Vu2d::from([49, 10]),
        start_val: 0,
        end_val: (Processor::CrossBlur as u32),
        step_size: 1,
        current_val: Processor::Normal as u32,
    };

    px::launch(async move
    {
        let game = px::EngineWrapper::new("Lines".to_owned(), (width as u32, height as u32, pixelsize as u32 * 2)).await;
        game.run(move |game: &mut px::Engine|
            {
                let start = std::time::Instant::now();
                if !true
                {
                    std::thread::sleep(std::time::Duration::from_millis(80));
                    return Ok(true);
                }
                
                pre_process_input(&mut cam_iter, input_mode, &mut frame);

                let past_input = std::time::Instant::now();

                for processor in processors.iter()
                {
                    use Processor::*;
                    let rgb = Color::new;
                    //process frame
                    match processor
                    {
                        Normal => target.pixels.copy_from_slice(&frame.pixels),
                        Sobel => frame.sobel_edge_detection_3x3(&mut target),
                        SobelColour => frame.sobel_edge_detection_3x3_colour(&mut target),
                        Threshold => frame.threshold(&mut target, (game.get_mouse_location().x*255/ game.size().x) as u8),
                        ThresholdColour => frame.threshold_colour(&mut target, (game.get_mouse_location().x * 255/ game.size().x) as u8),
                        RandomBiasDithering => frame.random_bias_dithering(&mut target, game.get_mouse_location().x as usize * 8 / game.size().x as usize + 1),
                        PatternedDithering => frame.patterned_dithering(&mut target, game.get_mouse_location().x as usize * 8 / game.size().x as usize + 1),
                        FloydSteinbergDithering => frame.floyd_steinberg_dithering(&mut target, game.get_mouse_location().x as usize * 8 / game.size().x as usize + 1),
                        FloydSteinbergDitheringCustomPalette => frame.floyd_steinberg_with_custom_colour_palette(&mut target, &[rgb(0,60,60),rgb(140,120,50),rgb(255,225,0),rgb(60,60,80),rgb(60,60,140),rgb(80,0,0),rgb(120,60,50),rgb(50,150,120),rgb(120,100,200)]),
                        GaussianBlur => frame.gaussian_blur_3x3(&mut target),
                        BoxBlur => frame.box_blur(&mut target, ((((game.get_mouse_location().x as usize * 255 * 49 / game.size().x.pow(2) as usize)/2)*2 + 1)).min((game.size().x/2) as usize * 2 - 1).max(3)),
                        Emboss => frame.emboss(&mut target),
                        Outline => frame.outline(&mut target),
                        GreyScale => frame.greyscale(&mut target),
                        ChromaticAberration => frame.chromatic_aberration(&mut target, (game.get_mouse_location().x as usize * 255/ game.size().x as usize )/20),
                        Sharpen => frame.sharpen(&mut target),
                        SharpenColour => frame.sharpen_colour(&mut target),
                        CrossBlur => frame.cross_blur(&mut target),
                    };
                }

                
                if game.get_mouse_btn(px::inputs::MouseBtn::Left).held
                {
                    let loc = game.get_mouse_location();
                    let value = slider.get_value(loc.x, loc.y);
                    processors[0] = unsafe{std::mem::transmute::<u8, Processor>(value as u8)};
                }

                if game.get_key(Keycodes::S).pressed
                {
                    save_image_as_png(&target);
                }

                if game.get_key(Keycodes::H).pressed
                {
                    hide_ui ^= true;
                }

                if game.get_key(Keycodes::Left).pressed
                {   
                    let lower = processors[0] as i32 - 1;
                    let val = if lower < 0 {Processor::CrossBlur as u8} else {lower as u8};
                    processors[0] = unsafe{std::mem::transmute::<u8, Processor>(val)};
                    slider.current_val = val as u32;
                }
                if game.get_key(Keycodes::Right).pressed
                {
                    let val = ( (processors[0] as i32 + 1) % (Processor::CrossBlur as i32 + 1) ) as u8;
                    processors[0] = unsafe{std::mem::transmute::<u8, Processor>(val)};
                    slider.current_val = val as u32;
                }
                if game.get_key(Keycodes::Down).pressed
                {
                    let lower = input_mode as i32 - 1;
                    let val = if lower < 0 {InputMode::Denoising as u8} else {lower as u8};
                    input_mode = unsafe{std::mem::transmute::<u8, InputMode>(val)};
                }
                if game.get_key(Keycodes::Up).pressed
                {
                    let val = ( (input_mode as i32 + 1) % (InputMode::Denoising as i32 + 1) ) as u8;
                    input_mode = unsafe{std::mem::transmute::<u8, InputMode>(val)};
                }

                for y in 0..game.size().y
                {
                    for x in 0..game.size().x
                    {
                        game.draw(Vi2d::from((x as i32, y as i32)) ,target[(x, y)]);
                    }
                }

                let end = std::time::Instant::now();
                if !hide_ui
                {
                    let scale = Vf2d::from([1.0, 1.0]);

                    game.draw_rect(slider.pos.cast_i32(), slider.size.cast_i32(), Color::new(70, 150, 140));
                    game.fill_rect(Vu2d::from([slider.get_slider_x(), slider.pos.y]).cast_i32(), Vi2d::from([2, slider.size.y as i32]), Color::new(200, 235, 225));
                    game.draw_text_decal(Vf2d::from([5.0, game.size().y as f32 - 20.0]), "Processor:", scale, Color::WHITE);
                    
                    // game.draw_string(5, game.size().y as i32 - 10, &format!("{:?}", processors[0]), Color::WHITE);
                    // game.draw_string(game.size().x as i32 - 80, game.size().x as i32 - 25, &"InputMode:".to_string(), Color::WHITE);
                    // game.draw_string(game.size().x as i32 - 80, game.size().x as i32 - 10, &format!("{:?}", input_mode), Color::WHITE);
                    
                    // game.draw_string(game.size().x as i32 - 145, 3, &"[<] Processors [>]".to_string(), Color::WHITE);
                    
                    // let inputy = 25;
                    // game.draw_string(game.size().x as i32 - 65  , inputy - 8, &"[^]".to_string()        , Color::WHITE);
                    // game.draw_string(game.size().x as i32 - 100 , inputy    , &"Input Modes".to_string(), Color::WHITE);
                    // game.draw_string(game.size().x as i32 - 65  , inputy + 8, &"[v]".to_string()        , Color::WHITE);
                    
                    // let keysy = 50;
                    // game.draw_string(game.size().x as i32 - 120, keysy, &"[H] hide UI".to_string(), Color::WHITE);
                    // game.draw_string(game.size().x as i32 - 120, keysy+10, &"[S] save image".to_string(), Color::WHITE);

                    // let input_duration = past_input - start;
                    // let rendering_duration = ((end - past_input) + frame_time * 99)/100;
                    // frame_time = rendering_duration;
                    // game.draw_string(0, 0, &("input duration: ".to_string() + &input_duration.as_secs_f32().to_string()), Color::WHITE);
                    // game.draw_string(0, 10, &("rendering duration: ".to_string() + &rendering_duration.as_secs_f32().to_string()), Color::WHITE);
                }

                if game.get_key(px::inputs::Keycodes::Escape).held
                {
                    return Ok(false); // Returning Ok(false) is the only way to do a clean shutdown
                }

                Ok(true) // Continue to next frame
            });
    });
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
#[derive(PartialEq, Clone, Copy, Debug)]
enum Processor
{
    Normal,
    Sobel,
    SobelColour,
    Threshold,
    ThresholdColour,
    RandomBiasDithering,
    PatternedDithering,
    FloydSteinbergDithering,
    FloydSteinbergDitheringCustomPalette,
    GaussianBlur,
    BoxBlur,
    Emboss,
    Outline,
    GreyScale,
    ChromaticAberration,
    Sharpen,
    SharpenColour,
    CrossBlur,
}

#[allow(dead_code)]
#[derive(PartialEq, Clone, Copy, Debug)]
enum InputMode
{
    Normal, 
    TimeBlend,
    Denoising,
}

struct Slider
{
    pos: Vu2d,
    size: Vu2d,
    start_val: u32, 
    end_val: u32,
    step_size: u32,
    current_val: u32,
}

impl Slider
{
    fn get_value(&mut self, x: u32, y: u32) -> u32
    {
        if self.is_hovering(x, y)
        {
            //inside slider
            let delta_val = self.end_val - self.start_val;
            self.current_val = (self.start_val + (((x - self.pos.x) * delta_val / self.size.x) / self.step_size ) * self.step_size) as u32;
        }
        return self.current_val;
    }

    fn is_hovering(&self, x: u32, y: u32) -> bool
    {
        let rightx = self.pos.x + self.size.x;
        let bottomy = self.pos.y + self.size.y;
        return x >= self.pos.x && x <= rightx
        && y >= self.pos.y && y <= bottomy;
    }

    fn get_slider_x(&self) -> u32
    {
        let delta_val = self.end_val - self.start_val;
        self.current_val * self.size.x / delta_val + self.pos.x 
    }
}

fn pre_process_input(cam_iter: &mut camera_capture::ImageIterator, input_mode: InputMode, frame: &mut Image)
{
    let input = match cam_iter.next()
    {
        Some(f) => f,
        None => return
    };
    match input_mode
    {
        InputMode::Normal
        =>  {
                for (i, pixel) in input.pixels().enumerate()
                {
                    frame.pixels[i] = Color::new(pixel.data[0], pixel.data[1], pixel.data[2]);
                }
            }
        ,

        InputMode::TimeBlend
        => 
            // fraction.0 is proportional to the influence of the next frame
            for (i, pixel) in input.pixels().enumerate()
            {
                let fraction = (2, 10);
                let pa = frame.pixels[i];
                let pb = Color::new(pixel.data[0], pixel.data[1], pixel.data[2]);

                let mut r = pa.r as u32 * fraction.1;
                let mut g = pa.g as u32 * fraction.1;
                let mut b = pa.b as u32 * fraction.1;

                r -= pa.r as u32 * fraction.0;
                g -= pa.g as u32 * fraction.0;
                b -= pa.b as u32 * fraction.0;

                r += pb.r as u32 * fraction.0;
                g += pb.g as u32 * fraction.0;
                b += pb.b as u32 * fraction.0;

                r /= fraction.1;
                g /= fraction.1;
                b /= fraction.1;

                frame.pixels[i] = Color::new(r as u8, g as u8, b as u8);
            }
        ,

        InputMode::Denoising
        => 
            //denoising based on pixel difference between frames
            for (i, pixel) in input.pixels().enumerate()
            {
                let p = temporal_denoising(frame.pixels[i], Color::new(pixel.data[0], pixel.data[1], pixel.data[2]));
                frame.pixels[i] = p;
            }
        ,
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