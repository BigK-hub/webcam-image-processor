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
