use olc_pge as olc;
use lerp::Lerp;

pub trait Illuminator
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
    }
}

pub trait MagnitudeSquared
{
    type Output;
    fn mag2(&self) -> Self::Output;
}

impl MagnitudeSquared for olc::Pixel
{
    type Output = u32;
    fn mag2(&self) -> Self::Output
    {
        let r = self.r as u32;
        let g = self.g as u32;
        let b = self.b as u32;
        return r*r+g*g+b*b;
    }
}

pub trait DistanceSquared
{
    type Output;
    fn distance_squared(&self, other:&Self) -> Self::Output;
}

impl DistanceSquared for olc::Pixel
{
    type Output = u32;
    fn distance_squared(&self, other: &olc::Pixel) -> Self::Output
    {
        let r = self.r as i32 - other.r as i32;
        let g = self.g as i32 - other.g as i32;
        let b = self.b as i32 - other.b as i32;

        return (r*r+g*g+b*b) as u32;
    }
}

pub trait PixelArithmetic
{
    fn clamping_add(&self, other: &Self) -> Self;

    fn clamping_mul(&self, factor: u8) -> Self;

    fn clamping_sub(&self, other: &Self) -> Self;

    fn clamping_div(&self, divisor: u8) -> Self;

    fn normalised_mul(&self, other:& Self) -> Self;
}

impl PixelArithmetic for olc::Pixel
{
    fn clamping_add(&self, other: &Self) -> Self
    {
        let mut r = self.r as u16;
        let mut g = self.g as u16;
        let mut b = self.b as u16;
        r += other.r as u16;
        g += other.g as u16;
        b += other.b as u16;
        r = r.min(255);
        g = g.min(255);
        b = b.min(255);
        return olc::Pixel::rgb(r as u8, g as u8, b as u8);
    }

    fn clamping_mul(&self, factor: u8) -> Self
    {
        let mut r = self.r as u16;
        let mut g = self.g as u16;
        let mut b = self.b as u16;
        r *= factor as u16;
        g *= factor as u16;
        b *= factor as u16;
        r = r.min(255);
        g = g.min(255);
        b = b.min(255);
        return olc::Pixel::rgb(r as u8, g as u8, b as u8);
    }

    fn clamping_sub(&self, other: &Self) -> Self
    {
        let mut r = self.r as i16;
        let mut g = self.g as i16;
        let mut b = self.b as i16;
        r -= other.r as i16;
        g -= other.g as i16;
        b -= other.b as i16;
        r = r.max(0);
        g = g.max(0);
        b = b.max(0);
        return olc::Pixel::rgb(r as u8, g as u8, b as u8);
    }

    fn clamping_div(&self, divisor: u8) -> Self
    {
        if divisor == 0
        {
            panic!("Function clamping_div may not have a divisor equal to 0.");
        }
        let mut r = self.r as u16;
        let mut g = self.g as u16;
        let mut b = self.b as u16;
        r /= divisor as u16;
        g /= divisor as u16;
        b /= divisor as u16;
        return olc::Pixel::rgb(r as u8, g as u8, b as u8);
    }

    fn normalised_mul(&self, other:& Self) -> Self
    {
        let mut r = self.r as u16;
        let mut g = self.g as u16;
        let mut b = self.b as u16;
        r *= other.r as u16;
        g *= other.g as u16;
        b *= other.b as u16;
        r /= 255;
        g /= 255;
        b /= 255;
        return olc::Pixel::rgb(r as u8, g as u8, b as u8);
    }
}

pub fn temporal_denoising(current_pixel: olc::Pixel, next_pixel: olc::Pixel) -> olc::Pixel
{
    let dist2 = current_pixel.distance_squared(&next_pixel);
    
    if dist2 > 100000
    {
        return next_pixel;
    }
    let fraction = (((dist2 as f32).sqrt() as u32 / 4).min(9) + 1,10);
    let p = current_pixel;
    let mut r = p.r as u32 * fraction.1 - p.r as u32 * fraction.0 + next_pixel.r as u32 * fraction.0;
    let mut g = p.g as u32 * fraction.1 - p.g as u32 * fraction.0 + next_pixel.g as u32 * fraction.0;
    let mut b = p.b as u32 * fraction.1 - p.b as u32 * fraction.0 + next_pixel.b as u32 * fraction.0;

    r /= fraction.1;
    g /= fraction.1;
    b /= fraction.1;
    return olc::Pixel::rgb(r as u8, g as u8, b as u8);
}