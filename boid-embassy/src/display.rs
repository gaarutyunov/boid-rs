use display_interface_spi::SPIInterface;
use embedded_graphics::{pixelcolor::Rgb565, prelude::*};
use embedded_hal::digital::OutputPin;
use esp_hal::{
    gpio::{Output, Pin},
    peripherals::SPI2,
    spi::master::{Spi, SpiDma},
    Blocking,
};
use mipidsi::{models::ST7789, Builder};

pub type Display = mipidsi::Display<
    SPIInterface<Spi<'static, SPI2, Blocking>, Output<'static>, Output<'static>>,
    ST7789,
    Output<'static>,
>;

pub struct DisplayWrapper {
    display: Display,
}

impl DisplayWrapper {
    pub fn new<CS: Pin, DC: Pin, RST: Pin>(
        spi: Spi<'static, SPI2, Blocking>,
        cs: Output<'static>,
        dc: Output<'static>,
        mut rst: Output<'static>,
    ) -> Self {
        // Reset the display
        rst.set_low();
        // Small delay would be good here, but we'll skip it for simplicity
        rst.set_high();

        let di = SPIInterface::new(spi, dc, cs);

        let display = Builder::new(ST7789, di)
            .reset_pin(rst)
            .display_size(240, 240)
            .invert_colors(mipidsi::options::ColorInversion::Inverted)
            .init(&mut embassy_time::Delay)
            .unwrap();

        Self { display }
    }

    pub fn clear(&mut self, color: Rgb565) -> Result<(), mipidsi::Error> {
        self.display.clear(color)
    }
}

impl DrawTarget for DisplayWrapper {
    type Color = Rgb565;
    type Error = mipidsi::Error;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        self.display.draw_iter(pixels)
    }
}

impl OriginDimensions for DisplayWrapper {
    fn size(&self) -> Size {
        self.display.size()
    }
}
