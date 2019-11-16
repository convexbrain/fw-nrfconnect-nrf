use cortex_m::asm;

use nrf91::TWIM2_S;

pub trait I2CRegAcc
{
    fn enable(&mut self, pn_scl: u8, pn_sda: u8, fast: bool);
    fn disable(&mut self);
    
    fn reg_read(&mut self, i2c_addr: u8, reg_addr: u8) -> u8;
    fn reg_write(&mut self, i2c_addr: u8, reg_addr: u8, data: u8);
    fn reg_write_mask(&mut self, i2c_addr: u8, reg_addr: u8, mask: u8, data: u8)
    {
        let tmp = self.reg_read(i2c_addr, reg_addr);
        let tmp = tmp & !mask;
        let data = data & mask;
        let tmp = tmp | data;
        self.reg_write(i2c_addr, reg_addr, tmp);
    }
}

impl I2CRegAcc for TWIM2_S
{
    fn enable(&mut self, pn_scl: u8, pn_sda: u8, fast: bool)
    {
        self.psel.scl.write(|w| unsafe { w
            .pin().bits(pn_scl)
            .connect().connected()
        } );
        self.psel.sda.write(|w| unsafe { w
            .pin().bits(pn_sda)
            .connect().connected()
        } );

        if fast {
            self.frequency.write(|w| w.
                frequency().k400()
            );
        }
        else {
            self.frequency.write(|w| w.
                frequency().k100()
            );
        }

        self.enable.write(|w| w.
            enable().enabled()
        );
    }

    fn disable(&mut self)
    {
        self.enable.write(|w| w.
            enable().disabled()
        );
    }

    fn reg_read(&mut self, i2c_addr: u8, reg_addr: u8) -> u8
    {
        let preg = &reg_addr;
        let preg = preg as *const u8;
        let preg = preg as u32;
        let mut data = 0_u8;
        let pdata = &mut data;
        let pdata = pdata as *mut u8;
        let pdata = pdata as u32;
    
        self.shorts.write(|w| w
            .lasttx_startrx().enabled()
            .lastrx_stop().enabled()
        );
        self.address.write(|w| unsafe { w
            .address().bits(i2c_addr)
        } );
        self.txd.list.write(|w| w
            .list().disabled()
        );
        self.txd.maxcnt.write(|w| unsafe { w
            .maxcnt().bits(1)
        } );
        self.txd.ptr.write(|w| unsafe { w
            .ptr().bits(preg)
        } );
        self.rxd.list.write(|w| w
            .list().disabled()
        );
        self.rxd.maxcnt.write(|w| unsafe { w
            .maxcnt().bits(1)
        } );
        self.rxd.ptr.write(|w| unsafe { w
            .ptr().bits(pdata)
        } );
    
        self.tasks_starttx.write(|w| w
            .tasks_starttx().trigger()
        );
    
        // NOTE: wait by polling
        let notgen = nrf91::twim0_ns::events_stopped::EVENTS_STOPPEDR::NOTGENERATED;
        while self.events_stopped.read().events_stopped() == notgen {
            asm::nop();
        }

        assert_eq!(self.txd.amount.read().amount().bits(), 1);
        assert_eq!(self.rxd.amount.read().amount().bits(), 1);
    
        self.events_lasttx.write(|w| w
            .events_lasttx().clear_bit()
        );
        self.events_lastrx.write(|w| w
            .events_lastrx().clear_bit()
        );
        self.events_stopped.write(|w| w
            .events_stopped().clear_bit()
        );
    
        unsafe {
            (pdata as *const u8).read()
        }
    }

    fn reg_write(&mut self, i2c_addr: u8, reg_addr: u8, data: u8)
    {
        let buf = [reg_addr, data];
        let ptr = &buf;
        let ptr = ptr as *const [u8];
        let ptr = ptr as *const u8;
        let ptr = ptr as u32;
    
        self.shorts.write(|w| w
            .lasttx_stop().enabled()
        );
        self.address.write(|w| unsafe { w
            .address().bits(i2c_addr)
        } );
        self.txd.list.write(|w| w
            .list().disabled()
        );
        self.txd.maxcnt.write(|w| unsafe { w
            .maxcnt().bits(2)
        } );
        self.txd.ptr.write(|w| unsafe { w
            .ptr().bits(ptr)
        } );
    
        self.tasks_starttx.write(|w| w
            .tasks_starttx().trigger()
        );
    
        // NOTE: wait by polling
        let notgen = nrf91::twim0_ns::events_stopped::EVENTS_STOPPEDR::NOTGENERATED;
        while self.events_stopped.read().events_stopped() == notgen {
            asm::nop();
        }

        assert_eq!(self.txd.amount.read().amount().bits(), 2);
    
        self.events_lasttx.write(|w| w
            .events_lasttx().clear_bit()
        );
        self.events_stopped.write(|w| w
            .events_stopped().clear_bit()
        );
    }
}
