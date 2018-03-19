use core::marker::PhantomData;

use hal;
use nb;

use nrf51::UART0;
use gpio::gpio::PIN;
use gpio::{Floating, Input, Output, PushPull};

pub use nrf51::uart0::baudrate::BAUDRATEW;
pub use nrf51::uart0::baudrate::BAUDRATEW::*;

/// Serial abstraction
pub struct Serial<UART> {
    uart: UART,
    txpin: PIN<Output<PushPull>>,
    rxpin: PIN<Input<Floating>>,
}

/// Serial receiver
pub struct Rx<UART> {
    _uart: PhantomData<UART>,
}

/// Serial transmitter
pub struct Tx<UART> {
    _uart: PhantomData<UART>,
}

#[derive(Debug)]
pub enum Error {}

impl Serial<UART0> {
    pub fn uart0(
        uart: UART0,
        txpin: PIN<Output<PushPull>>,
        rxpin: PIN<Input<Floating>>,
        speed: BAUDRATEW,
    ) -> Self {
        /* Tell UART which pins to use for sending and receiving */
        uart.pseltxd
            .write(|w| unsafe { w.bits(txpin.get_id().into()) });
        uart.pselrxd
            .write(|w| unsafe { w.bits(rxpin.get_id().into()) });

        /* Set baud rate */
        uart.baudrate.write(|w| w.baudrate().variant(speed));

        /* Enable UART interrupt */
        uart.intenset.write(|w| w.rxdrdy().set().txdrdy().set());

        /* Enable UART function */
        uart.enable.write(|w| w.enable().enabled());

        /* Fire up transmitting and receiving task */
        uart.tasks_starttx.write(|w| unsafe { w.bits(1) });
        uart.tasks_startrx.write(|w| unsafe { w.bits(1) });

        /* Write an initial byte to force events_txdrdy to trigger */
        uart.txd.write(|w| unsafe { w.bits(0) });

        Serial { uart, txpin, rxpin }
    }

    pub fn release(self) -> (UART0, PIN<Output<PushPull>>, PIN<Input<Floating>>) {
        (self.uart, self.txpin, self.rxpin)
    }

    pub fn split(self) -> (Tx<UART0>, Rx<UART0>) {
        (Tx { _uart: PhantomData }, Rx { _uart: PhantomData })
    }
}

impl hal::serial::Read<u8> for Rx<UART0> {
    type Error = Error;

    fn read(&mut self) -> nb::Result<u8, Error> {
        let uart = unsafe { &*UART0::ptr() };
        match uart.events_rxdrdy.read().bits() {
            0 => Err(nb::Error::WouldBlock),
            _ => {
                /* We're going to pick up the data soon, let's signal the buffer is already waiting for
                 * more data */
                uart.events_rxdrdy.reset();

                /* Read one 8bit value */
                Ok(uart.rxd.read().bits() as u8)
            }
        }
    }
}

impl hal::serial::Write<u8> for Tx<UART0> {
    type Error = !;

    fn flush(&mut self) -> nb::Result<(), !> {
        let uart = unsafe { &*UART0::ptr() };
        match uart.events_txdrdy.read().bits() {
            0 => Err(nb::Error::WouldBlock),
            _ => Ok(()),
        }
    }

    fn write(&mut self, byte: u8) -> nb::Result<(), !> {
        let uart = unsafe { &*UART0::ptr() };
        match uart.events_txdrdy.read().bits() {
            0 => Err(nb::Error::WouldBlock),
            _ => {
                uart.events_txdrdy.reset();
                uart.txd.write(|w| unsafe { w.bits(u32::from(byte)) });
                Ok(())
            }
        }
    }
}
