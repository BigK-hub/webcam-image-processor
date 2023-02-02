use pixel_engine::Color;
use crate::pixel_traits::*;
use pixel_engine::vector2::Vi2d;

#[derive(Clone)]
pub struct Image
{
    pub width:usize,
    pub height:usize,
    pub pixels: Vec<Color>
}

impl std::ops::Index<(usize, usize)> for Image
{
    type Output = Color;
    fn index(&self, index: (usize, usize)) -> &Self::Output
    {
        debug_assert!(index.0 < self.width && index.1 < self.height, "Can't index Image with invalid coordinates.");
        &self.pixels[index.1*self.width+index.0]
    }
}

impl std::ops::IndexMut<(usize, usize)> for Image
{
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output
    {
        debug_assert!(index.0 < self.width && index.1 < self.height, "Can't index Image with invalid coordinates.");
        &mut self.pixels[index.1*self.width+index.0]
    }
}

impl std::ops::Index<(u32, u32)> for Image
{
    type Output = Color;
    fn index(&self, index: (u32, u32)) -> &Self::Output
    {
        let x = index.0 as usize;
        let y = index.1 as usize;
        debug_assert!(x < self.width && y < self.height, "Can't index Image with invalid coordinates.");
        &self.pixels[y*self.width+x]
    }
}

impl std::ops::IndexMut<(u32, u32)> for Image
{
    fn index_mut(&mut self, index: (u32, u32)) -> &mut Self::Output
    {
        let x = index.0 as usize;
        let y = index.1 as usize;
        debug_assert!(x < self.width && y < self.height, "Can't index Image with invalid coordinates.");
        &mut self.pixels[y*self.width+x]
    }
}

impl std::ops::Index<Vi2d> for Image
{
    type Output = Color;
    fn index(&self, index: Vi2d) -> &Self::Output
    {
        debug_assert!((0..self.width).contains(&(index.x as usize)) &&  (0..self.height).contains(&(index.y as usize)), "Can't index Image with invalid coordinates.");
        let x = index.x as usize;
        let y = index.y as usize;
        &self.pixels[y*self.width+x]
    }
}

impl std::ops::IndexMut<Vi2d> for Image
{
    fn index_mut(&mut self, index: Vi2d) -> &mut Self::Output
    {
        debug_assert!((0..self.width).contains(&(index.x as usize)) &&  (0..self.height).contains(&(index.y as usize)), "Can't index Image with invalid coordinates.");
        let x = index.x as usize;
        let y = index.y as usize;
        &mut self.pixels[y*self.width+x]
    }
}

fn get_average_colour(colours: &[Color]) -> Color
{
    let mut r = 0;
    let mut g = 0;
    let mut b = 0;
    for &colour in colours
    {
        r += colour.r as u32;
        g += colour.g as u32;
        b += colour.b as u32;
    }
    r /= colours.len() as u32;
    g /= colours.len() as u32;
    b /= colours.len() as u32;
    let r = r as u8;
    let g = g as u8;
    let b = b as u8;
    return Color::new(r,g,b);
}

impl Image
{
    /// **Not** a [mathematical convolution].
    /// 
    /// 
    /// This function applies one [kernel] to an image and should be paired with the [handle_edges] function.
    /// 
    /// The kernel is applied to each channel, resulting in colourful images.
    /// 
    /// ## Expected arguments
    /// 
    /// `&self` is the Image from which pixels are read. 
    /// # 
    /// `target: &mut Image` is a mutable reference to the target Image that the result of the convolution will be written to.
    /// # 
    /// `kernel_size`. The convolve function assumes a square shaped kernel.
    /// `kernel_size` should be equal to the width or height of the kernel. Or given an example kernel `[0,0,0, 0,0,0, 0,0,0]` with `9` elements, 
    /// `kernel_size` should be equal to `sqrt(9)`. 
    /// # 
    /// `kernel_generator`. It is a function object that takes in 
    /// `kernel_size: usize, (kernel_x: usize, kernel_y: usize)`
    /// and returns an `i32`, representing an integer multiple of `1/denominator`.
    /// # 
    /// `denominator`. It can be thought of as the "divisor". Since convolve only uses integers, 
    /// fractions are represented by multiplying the kernel values with 1/denominator. 
    /// 
    /// Hence a kernel `[2,2,2, 4,4,4, 2,2,2]`, combined with the denominator equal to `4`, 
    /// works the same as applying this kernel with these floating point values:
    /// 
    /// `[0.5,0.5,0.5, 1.0,1.0,1.0, 0.5,0.5,0.5]`.
    /// 
    /// ## Example
    /// ```rust
    /// pub fn box_blur(&self, target: &mut Image)
    /// {
    ///     self.convolve(target, 3, |s,(x,y)| 1, 9); // here kernel_generator always returns 1
    ///     // self.handle_edges(target, 3,
    ///     // |img, _s, (x,y)|
    ///     // img[(x,y)]
    ///     // );
    /// }
    /// ```
    /// 
    /// Returning `1` with the `denominator` being `9` has the effect of multiplying the pixel with `1/9`.
    /// 
    /// ## When to use
    /// 
    /// This function gets rid of a lot of redundancy for image processing effects that:
    /// - make use of a kernel
    /// - only have one kernel
    /// - apply their kernel to each channel
    /// 
    /// If any of these do not apply, convolve will probably not be very useful.
    /// 
    /// [kernel]: https://en.wikipedia.org/wiki/Kernel_(image_processing)
    /// [mathematical convolution]: https://en.wikipedia.org/wiki/Convolution
    pub fn convolve<F>(&self, target: &mut Image, kernel_size: usize, mut kernel_generator: F, denominator: i32) where F: FnMut(usize, (usize, usize)) -> i32
    {
        if denominator == 0 {panic!("In function convolve of Image, denominator may not equal 0.");}
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
                        let pixel = self[(x - kernel_size/2 + kernel_x, y - kernel_size/2 + kernel_y)];
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
                target[(x, y)] =  Color::new(r as u8, g as u8, b as u8);
            }
        }
    }


    /// Also called padding, this function handles the pixels that an [image processing kernel] couldn't reach.
    /// 
    /// ## Expected arguments
    /// 
    /// `&self` is the Image from which pixels are read. 
    /// # 
    /// `target: &mut Image` is the Image that is written to.
    /// # 
    /// `kernel_size`. `handle_edges` assumes a square shaped kernel.
    /// `kernel_size` should be equal to the width or height of the kernel. Or given an example kernel `[0,0,0, 0,0,0, 0,0,0]` with `9` elements, 
    /// `kernel_size` should be equal to `sqrt(9)`. 
    /// # 
    /// `edge_handler` is a function object that, unlike the kernel generator in `convolve()`, takes in 
    /// 
    /// `kernel_size: usize, (image_x: usize, image_y: usize)`, and returns an `Color` instead of an `i32`, which the target pixel will be set to.
    /// 
    /// The x and y in this function do **not** represent the coordinates of the kernel values.
    /// 
    /// ## Example
    /// ```rust
    /// pub fn box_blur(&self, target: &mut Image)
    /// {
    ///     // self.convolve(target, 3, |s,(x,y)| 1, 9);
    ///     self.handle_edges(target, 3,
    ///     |img, _s, (x,y)|
    ///     img[(x,y)] // here the `edge_handler` just returns the original colour of the image.
    ///     );
    /// }
    /// ```
    /// 
    /// 
    /// [image processing kernel]: https://en.wikipedia.org/wiki/Kernel_(image_processing)
    pub fn handle_edges<F>(&self, target: &mut Image, kernel_size: usize, edge_handler: F) where F: Fn(&Image, usize, (usize, usize)) -> Color
    {
        for y in (0..kernel_size/2).chain(self.height - kernel_size/2 .. self.height)
        {
            for x in 0..self.width
            {
                let pixel = edge_handler(self, kernel_size, (x, y));
                target[(x, y)] = pixel;
            }
        }
        for y in 0..self.height
        {
            for x in (0..kernel_size/2).chain(self.width - kernel_size/2..self.width)
            {
                let pixel = edge_handler(self, kernel_size, (x, y));
                target[(x, y)] = pixel;
            }
        }
    }

    /// Applies `transformer` to each pixel in the image.
    /// 
    /// ## Expected arguments
    /// `&self` is the Image from which pixels are read.
    /// 
    /// `target: &mut Image` is the Image that is written to.
    /// 
    /// `tranformer: F` is a function that takes in a pixel, and returns a transformed version of it.
    /// 
    /// ## Example
    /// ```
    /// pub fn greyscale(&self, target: &mut Image)
    /// {
    ///     self.for_each(target,
    ///         |p|
    ///         {
    ///             let brt = p.brightness();
    ///             Color::new(brt,brt,brt)
    ///         }
    ///     );
    /// }
    /// ```
    pub fn map<F>(&self, target: &mut Image, transformer: F) where F: Fn(Color) -> Color
    {
        //let (prefix, simd, suffix) = target.pixels.iter().map(|p| [p.r, p.g, p.b, p.a].iter()).as_simd_mut();
        for (i, &pixel) in self.pixels.iter().enumerate()
        {
            target.pixels[i] = transformer(pixel);
        }
    }
    
    pub fn greyscale(&self, target: &mut Image)
    {
        self.map(target,
            |p|
            {
                let brt = p.brightness();
                Color::new(brt,brt,brt)
            }
        );
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
                    target[(x, y)] = Color::BLACK;
                    continue;
                }
                let mut output = 0;
                for kernel_y in 0..3
                {
                    for kernel_x in 0..3
                    {
                        let kernel_value = kernel[kernel_y*3+kernel_x];
                        let brightness = self[(x - 3/2 + kernel_x, y - 3/2 + kernel_y)].brightness() as i32;
                        output += brightness * kernel_value;
                    }
                }
                let value = output.min(255).max(0) as u8;
                target[(x, y)] = Color::new(value, value, value);
            }                   
        }
    }

    pub fn sharpen_colour(&self, target: &mut Image)
    {
        self.convolve(target, 3, |s,(x,y)| [0, -1, 0, -1, 5, -1, 0, -1, 0][y*s+x], 1);
        self.handle_edges(target, 3,
            |img, _s, (x,y)|
            img[(x,y)]
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
                    target[(x, y)] = Color::BLACK;
                    continue;
                }
                let mut val_x = 0;
                let mut val_y = 0;
                for kernel_y in 0..3
                {
                    for kernel_x in 0..3
                    {
                        let ix = x + kernel_x - 1;
                        let iy = y + kernel_y - 1;
                        let ik = kernel_y * 3 + kernel_x;
                        let current_brightness = self[(ix,iy)].brightness();
                        val_x += current_brightness as i32 * s_x[ik];
                        val_y += current_brightness as i32 * s_y[ik];
                    }
                }

                let value = ((val_x*val_x + val_y*val_y) as f32).sqrt() as u8;
                target[(x, y)] = Color::new(value, value, value);
            }                   
        }
    }

    pub fn threshold(&self, target: &mut Image, threshold: u8)
    {
        self.map(target,
            |p|
            {
                let brt = p.brightness(); 
                if brt >= threshold {Color::WHITE} else {Color::BLACK}
            }
        );
    }

    pub fn threshold_colour(&self, target: &mut Image, threshold: u8)
    {
        self.map(target,
            |p|
            {
                Color::new
                (
                    (p.r >= threshold) as u8 * 255,
                    (p.g >= threshold) as u8 * 255,
                    (p.b >= threshold) as u8 * 255,
                )
            }
        );
    }

    pub fn patterned_dithering(&self, target: &mut Image, bits_per_channel:usize)
    {
        assert_ne!(bits_per_channel, 0);
        let max_values_per_channel = (1<<bits_per_channel).min(255);
        self.patterned_dithering_impl(target, max_values_per_channel);
    }

    pub fn patterned_dithering_impl(&self, target: &mut Image, max_values_per_channel: usize)
    {
        let quantisation_factor = 255/(max_values_per_channel-1);
        let quantise = |mut p: Color, factor|
            {
                p = p.div(factor).clamping_mul(factor);
                p
            };
        for y in 0..self.height
        {
            for x in 0..self.width
            {
                let mut pixel = self[(x,y)];
                let mut bias = 0;
                let quantisation_factor = quantisation_factor as u16;
                let divisor = 20;

                if false
                {
                    //checkerboard kinda pattern
                    bias += (x%2 == y%2) as u16 * 4 * quantisation_factor / (divisor + 1);
                }
                else
                {
                    //square squiggles pattern
                    bias += (x%2==0) as u16 * 4 * quantisation_factor / (divisor + 1);
                    bias += (x%4==0) as u16 * 2 * quantisation_factor / (divisor + 1);
                    bias += (x%8==0) as u16 * 2 * quantisation_factor / (divisor + 1);

                    bias += (y%2==0) as u16 * 4 * quantisation_factor / (divisor + 1);
                    bias += (y%4==0) as u16 * 2 * quantisation_factor / (divisor + 1);
                    bias += (y%8==0) as u16 * 2 * quantisation_factor / (divisor + 1);
                }

                bias += ((x%4 == 0) == (y%4==0)) as u16 * 4 * quantisation_factor / (divisor + 1);
                let bias = bias.min(255) as u8;
                pixel = pixel.clamping_add(Color::new(bias,bias,bias));
                target[(x,y)] = quantise(pixel, quantisation_factor as u8);
            }
        }
    }

    pub fn random_bias_dithering(&self, target: &mut Image, bits_per_channel:usize)
    {
        assert_ne!(bits_per_channel, 0);
        let max_values_per_channel = (1<<bits_per_channel).min(255);
        self.random_bias_dithering_impl(target, max_values_per_channel);
    }

    pub fn random_bias_dithering_impl(&self, target: &mut Image, max_values_per_channel: usize)
    {
        let quantisation_factor = 255/(max_values_per_channel- 1) as u8;
        let quantise = |p: Color, factor|
            Color::new(
                (p.r / factor) * factor,
                (p.g / factor) * factor,
                (p.b / factor) * factor,
        );
        for y in 0..self.height
        {
            for x in 0..self.width
            {
                let mut pixel = self[(x,y)];
                let r = fastrand::u8(0..(quantisation_factor.max(2) as u16 *4/5) as u8);
                pixel = pixel.clamping_add(Color::new(r,r,r));
                target[(x,y)] = quantise(pixel, quantisation_factor);
            }
        }
    }

    pub fn floyd_steinberg_dithering(&self, target: &mut Image, bits_per_channel:usize)
    {
        assert_ne!(bits_per_channel, 0);
        let max_values_per_channel = if bits_per_channel > 7 {255} else{1 << bits_per_channel};
        self.floyd_steinberg_dithering_impl(target, max_values_per_channel);
    }

    pub fn floyd_steinberg_with_custom_colour_palette(&self, target: &mut Image, colour_palette: &[Color])
    {
        target.pixels.copy_from_slice(&self.pixels);

        let quantise = |pixel: Color|
        {
            let mut nearest_dist = 999999999;
            let mut nearest_pixel = Color::BLACK;
            for &other in colour_palette
            {
                if other.r <= pixel.r
                && other.g <= pixel.g
                && other.b <= pixel.b
                {
                    let dist = pixel.distance_squared(other);
                    if dist < nearest_dist
                    {
                        nearest_dist = dist;
                        nearest_pixel = other;
                    }
                }
            }
            nearest_pixel
        };

        let weighted_error = |error: Color, factor:u8| error.clamping_fraction_mul((factor, 16));

        for y in 1..self.height-1
        {
            for x in 1..self.width-1
            {
                let old_pixel = target[(x,y)];
                let new = quantise(old_pixel);

                //we know that quantising a pixel always makes it either the same or darker than before
                //this means that the error is always positive, and we can use u8s to store it
                let error = old_pixel.sub(new);
                
                let mut diffuse_error = |pos:(usize,usize), factor :u8|
                {
                    target[pos] = target[pos].clamping_add
                    (
                        weighted_error(error, factor)
                    );
                };

                diffuse_error((x+1,   y),7);
                diffuse_error((x-1, y+1),3);
                diffuse_error((x  , y+1),5);
                diffuse_error((x+1, y+1),1);
                target[(x,y)] = new;
            }
        }
    }

    pub fn floyd_steinberg_dithering_impl(&self, target: &mut Image, max_values_per_channel: usize)
    {
        let quantisation_factor = (255/(max_values_per_channel-1) as u16) as u8;

        target.pixels.copy_from_slice(&self.pixels);

        let quantise = |pixel: Color| pixel.div(quantisation_factor).clamping_mul(quantisation_factor);

        let weighted_error = |error: Color, factor:u8| error.clamping_fraction_mul((factor, 16));

        for y in 1..self.height-1
        {
            for x in 1..self.width-1
            {
                let old_pixel = target[(x,y)];
                let new = quantise(old_pixel);

                //we know that quantising a pixel always makes it either the same or darker than before
                //this means that the error is always positive, and we can use u8s to store it
                let error = old_pixel.sub(new);
                
                let mut diffuse_error = |pos:(usize,usize), factor :u8|
                {
                    target[pos] = target[pos].clamping_add
                    (
                        weighted_error(error, factor)
                    );
                };

                diffuse_error((x+1,   y),7);
                diffuse_error((x-1, y+1),3);
                diffuse_error((x  , y+1),5);
                diffuse_error((x+1, y+1),1);
                target[(x,y)] = new;
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
            img[(x,y)]
        );
    }
    pub fn emboss(&self, target: &mut Image)
    {
        self.convolve(target, 3, |s, (x,y)|
            [
                -2, -1, 0,
                -1, 1, 1,
                0, 1, 2
            ][y*s+x],1
        );
        self.handle_edges(target,3,
            |img, _s, (x,y)|
            img[(x,y)]
        );
    }
    pub fn outline(&self, target: &mut Image)
    {
        self.convolve(target, 3, |s, (x,y)|
            [
                -1, -1, -1,
                -1, 8, -1,
                -1, -1, -1
            ][y*s+x],1
        );
        self.handle_edges(target,3,
            |img, _s, (x,y)|
            img[(x,y)]
        );
    }

    pub fn map_brightness_to_colour_pallette(&self, target: &mut Image, colour_palette: &[Color])
    {
        if colour_palette.is_empty()
        {
            for pixel in target.pixels.iter_mut()
            {
                *pixel = Color::BLACK;
            }
        }
        let divisor = 255/colour_palette.len().min(255) + 1;
        for y in 0..self.height
        {
            for x in 0..self.width
            {
                target[(x,y)] = colour_palette[self[(x,y)].brightness() as usize / divisor];
            }
        }
    }

    fn get_average_colour_and_variance_of_block(&self, rangex: std::ops::Range<usize>, rangey: std::ops::Range<usize>) -> (Color, u32)
    {
        let mut avg_colour = (0i32,0i32,0i32);
        let mut counter = 0;
        for y in rangey.clone()
        {
            for x in rangex.clone()
            {
                let p = self[(x,y)];
                if counter == 0
                {
                    counter = 1;
                    avg_colour = (p.r as i32, p.g as i32, p.b as i32);
                    continue;
                }
                avg_colour.0 += p.r as i32;
                avg_colour.1 += p.g as i32;
                avg_colour.2 += p.b as i32;
                counter += 1;
            }
        }
        avg_colour.0 /= counter;
        avg_colour.1 /= counter;
        avg_colour.2 /= counter;
        
        let mut variance = 0;
        for y in rangey
        {
            for x in rangex.clone()
            {
                let p = self[(x,y)];
                let mut sum = 0;
                sum += (p.r as i32 - avg_colour.0).pow(2);
                sum += (p.g as i32 - avg_colour.1).pow(2);
                sum += (p.b as i32 - avg_colour.1).pow(2);
                variance += sum/3;
            }
        }

        (Color::new(avg_colour.0 as u8, avg_colour.1 as u8, avg_colour.2 as u8), variance as u32)
    }

    pub fn basic_kuwahara_filter(&self, target: &mut Image, kernel_size: usize)
    {
        if kernel_size % 2 == 0
        {
            self.basic_kuwahara_filter(target, kernel_size.max(2) - 1);
            return;
        }
        for y in 0..self.height
        {
            for x in 0..self.width
            {
                let mut output = Color::BLACK;
                if (kernel_size/2..self.width - kernel_size/2).contains(&x)
                && (kernel_size/2..self.height - kernel_size/2).contains(&y)
                {
                    let x1 = kernel_size/2;
                    let x2 = kernel_size - kernel_size / 2;
                    let y1 = kernel_size/2;
                    let y2 = kernel_size - kernel_size / 2;
                    let mut lowest_variance = u32::MAX;
                    let x = x - kernel_size/ 2;
                    let y = y - kernel_size/2;
                    let (avg_quadrant_1, variance) = self.get_average_colour_and_variance_of_block(x..x+x1, y..y+y1);
                    if variance < lowest_variance
                    {
                        lowest_variance = variance;
                        output = avg_quadrant_1;
                    }
                    let (avg_quadrant_2, variance) = self.get_average_colour_and_variance_of_block(x+x2..x+kernel_size, y+0..y+y1);
                    if variance < lowest_variance
                    {
                        lowest_variance = variance;
                        output = avg_quadrant_2;
                    }
                    let (avg_quadrant_3, variance) = self.get_average_colour_and_variance_of_block(x+0..x+x1, y+y2..y+kernel_size);
                    if variance < lowest_variance
                    {
                        lowest_variance = variance;
                        output = avg_quadrant_3;
                    }
                    let (avg_quadrant_4, variance) = self.get_average_colour_and_variance_of_block(x+x2..x+kernel_size, y+y2..y+kernel_size);
                    if variance < lowest_variance
                    {
                        output = avg_quadrant_4;
                    }
                }
                target[(x,y)] = output;
            }
        }
    }

    pub fn improved_kuwahara_filter(&self, target: &mut Image, kernel_size: usize)
    {
        if kernel_size % 2 == 0
        {
            self.basic_kuwahara_filter(target, kernel_size.max(2) - 1);
            return;
        }
        for y in 0..self.height
        {
            for x in 0..self.width
            {
                let mut output = Color::BLACK;
                if (kernel_size/2..self.width - kernel_size/2).contains(&x)
                && (kernel_size/2..self.height - kernel_size/2).contains(&y)
                {
                    let x1 = kernel_size/2;
                    let x2 = kernel_size - kernel_size / 2;
                    let y1 = kernel_size/2;
                    let y2 = kernel_size - kernel_size / 2;
                    let mut lowest_variance = u32::MAX;
                    let x = x - kernel_size/ 2;
                    let y = y - kernel_size/2;
                    let (avg_quadrant_1, variance) = self.get_average_colour_and_variance_of_block(x..x+x1, y..y+y1);
                    if variance < lowest_variance
                    {
                        lowest_variance = variance;
                        output = avg_quadrant_1;
                    }
                    else if variance == lowest_variance
                    {
                        lowest_variance = variance;
                        output = avg_quadrant_1.div(2).clamping_add(output.div(2));
                    }
                    let (avg_quadrant_2, variance) = self.get_average_colour_and_variance_of_block(x+x2..x+kernel_size, y+0..y+y1);
                    if variance < lowest_variance
                    {
                        lowest_variance = variance;
                        output = avg_quadrant_2;
                    }
                    else if variance == lowest_variance
                    {
                        lowest_variance = variance;
                        output = avg_quadrant_2.div(2).clamping_add(output.div(2));
                    }
                    let (avg_quadrant_3, variance) = self.get_average_colour_and_variance_of_block(x+0..x+x1, y+y2..y+kernel_size);
                    if variance < lowest_variance
                    {
                        lowest_variance = variance;
                        output = avg_quadrant_3;
                    }
                    else if variance == lowest_variance
                    {
                        lowest_variance = variance;
                        output = avg_quadrant_3.div(2).clamping_add(output.div(2));
                    }
                    let (avg_quadrant_4, variance) = self.get_average_colour_and_variance_of_block(x+x2..x+kernel_size, y+y2..y+kernel_size);
                    if variance < lowest_variance
                    {
                        output = avg_quadrant_4;
                    }
                    else if variance == lowest_variance
                    {
                        output = avg_quadrant_4.div(2).clamping_add(output.div(2));
                    }
                }
                target[(x,y)] = output;
            }
        }
    }


    /// Offsets each channel by the provided `offset`.
    pub fn chromatic_aberration(&self, target: &mut Image, offset: usize)
    {
        for y in 0..self.height
        {
            for x in 0..self.width
            {
                if (0..self.height-offset).contains(&y) && (0..self.width-offset).contains(&x)
                {
                    let r = self[(x+offset, y+offset)].r;
                    target[(x,y)].r = r;
                }
                else
                {
                    target[(x,y)].r = 0;
                }
                if (offset..self.height).contains(&y) && (offset..self.width).contains(&x)
                {
                    let b = self[(x - offset, y - offset)].b;
                    target[(x,y)].b = b;
                }
                else
                {
                    target[(x,y)].b = 0;
                }
                let g = self[(x,y)].g;
                target[(x,y)].g = g;
            }
        }
    }


    pub fn naive_box_blur(&self, target: &mut Image, kernel_size: usize)
    {
        self.convolve(target, kernel_size, |_s, (_x, _y)| 1, (kernel_size*kernel_size) as i32);
        let average_colour = get_average_colour(&self.pixels);
        self.handle_edges(target, kernel_size, 
            |_, _, _|
            average_colour
        );
    }

    pub fn efficient_box_blur(&mut self, target: &mut Image, temp: &mut Image, kernel_size: usize)
    {
        //i had no idea that kernel convolutions can be seperable
        //this seems so obvious now

        for y in 0..self.height
        {
            for x in kernel_size/2..self.width-kernel_size/2
            {
                let index = y*self.width+x;
                let avg = get_average_colour(&self.pixels[index-kernel_size/2..index+kernel_size/2]);
                temp[(x,y)] = avg;
            }
        }
        for y in kernel_size/2..self.height-kernel_size/2
        {
            for x in kernel_size/2..self.width-kernel_size/2
            {
                let mut r = 0;
                let mut g = 0;
                let mut b = 0;
                for _y in y-kernel_size/2..y+kernel_size-kernel_size/2
                {
                    let colour = temp[(x,_y)];
                    r += colour.r as u32;
                    g += colour.g as u32;
                    b += colour.b as u32;
                }
                r /= kernel_size as u32;
                g /= kernel_size as u32;
                b /= kernel_size as u32;
                let r = r as u8;
                let g = g as u8;
                let b = b as u8;
                target[(x,y)] = Color::new(r,g,b);
            }
        }
        self.handle_edges(target, kernel_size, 
            |_, _, _|
            Color::BLACK
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
                    target[(x, y)] = Color::BLACK;
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
                        let current_pixel = self[(ix,iy)];
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
                target[(x, y)] = Color::new(r, g, b);
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
                let mut val_x = 0;
                let mut val_y = 0;
                for kernel_y in 0..3
                {
                    for kernel_x in 0..3
                    {
                        let ix = x + kernel_x - 1;
                        let iy = y + kernel_y - 1;
                        let ik = kernel_y * 3 + kernel_x;
                        let current_brightness = self[(ix,iy)].brightness();
                        val_x += current_brightness as i32 * s_x[ik];
                        val_y += current_brightness as i32 * s_y[ik];
                    }
                }

                let value = ((val_x*val_x + val_y*val_y) as f32).sqrt() as u8;
                target[(x, y)] = Color::new(value, value, value);
            }
        }
    }
}