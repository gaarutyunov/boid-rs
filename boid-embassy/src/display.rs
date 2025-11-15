use display_interface_spi::SPIInterface;
use embedded_graphics::{pixelcolor::Rgb565, prelude::*};
use esp_idf_hal::{
    delay::FreeRtos,
    gpio::{Output, PinDriver},
    spi::SpiDeviceDriver,
};
use mipidsi::{models::ST7789, Builder};

pub type Display<'a> = mipidsi::Display<
    SPIInterface<SpiDeviceDriver<'a, &'a mut esp_idf_hal::spi::SpiDriver<'a>>, PinDriver<'a, esp_idf_hal::gpio::AnyOutputPin, Output>, PinDriver<'a, esp_idf_hal::gpio::AnyOutputPin, Output>>,
    ST7789,
    PinDriver<'a, esp_idf_hal::gpio::AnyOutputPin, Output>,
>;

pub struct DisplayWrapper<'a> {
    display: Display<'a>,
}

impl<'a> DisplayWrapper<'a> {
    pub fn new(
        spi: SpiDeviceDriver<'a, &'a mut esp_idf_hal::spi::SpiDriver<'a>>,
        dc: PinDriver<'a, esp_idf_hal::gpio::AnyOutputPin, Output>,
        mut rst: PinDriver<'a, esp_idf_hal::gpio::AnyOutputPin, Output>,
    ) -> Self {
        // Reset the display
        rst.set_low().ok();
        FreeRtos::delay_ms(10);
        rst.set_high().ok();
        FreeRtos::delay_ms(10);

        // Note: For SPIInterface::new, we need to pass a dummy CS pin
        // since SpiDeviceDriver already handles CS
        let di = SPIInterface::new(spi, dc);

        let display = Builder::new(ST7789, di)
            .reset_pin(rst)
            .display_size(240, 240)
            .invert_colors(mipidsi::options::ColorInversion::Inverted)
            .init(&mut FreeRtos)
            .unwrap();

        Self { display }
    }

    pub fn clear(&mut self, color: Rgb565) -> Result<(), mipidsi::Error> {
        self.display.clear(color)
    }
}

impl<'a> DrawTarget for DisplayWrapper<'a> {
    type Color = Rgb565;
    type Error = mipidsi::Error;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        self.display.draw_iter(pixels)
    }
}

impl<'a> OriginDimensions for DisplayWrapper<'a> {
    fn size(&self) -> Size {
        self.display.size()
    }
}
