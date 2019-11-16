/*!
Reference:
* fw-nrfconnect-nrf/drivers/adp536x/adp536x.c
*/

use cortex_m::asm;

use nrf91::TWIM0_S;

//

pub trait I2CRegAcc
{
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

impl I2CRegAcc for TWIM0_S
{
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

//

const fn mask_bits(h: u8, l: u8) -> u8
{
    let v = 1 << h;
    let v = v - (1 << l);
    let v = v | (1 << h);
    v
}

const ADP536X_I2C_ADDR: u8 = 0x46;

/* Register addresses */
const ADP536X_BUCK_OUTPUT: u8 = 0x2A;
const ADP536X_BUCKBST_OUTPUT: u8 = 0x2C;
const ADP536X_BUCKBST_CFG: u8 = 0x2B;

/* Buck output voltage setting register. */
const ADP536X_BUCK_OUTPUT_VOUT_BUCK_MSK: u8 = mask_bits(5, 0);
const ADP536X_BUCK_OUTPUT_VOUT_BUCK_SFT: u8 = 0;

/* Buck/boost output voltage setting register. */
const ADP536X_BUCKBST_OUTPUT_VOUT_BUCKBST_MSK: u8 = mask_bits(5, 0);
const ADP536X_BUCKBST_OUTPUT_VOUT_BUCKBST_SFT: u8 = 0;

/* Buck/boost configure register. */
const ADP536X_BUCKBST_CFG_EN_BUCKBST_MSK: u8 = mask_bits(0, 0);
const ADP536X_BUCKBST_CFG_EN_BUCKBST_SFT: u8 = 0;

//

pub struct ADP536X
{
    twim0_s: TWIM0_S
}

impl ADP536X
{
    pub fn new(mut twim0_s: TWIM0_S) -> ADP536X
    {
        twim0_s.psel.scl.write(|w| unsafe { w
            .pin().bits(12)
            .connect().connected()
        } );
        twim0_s.psel.sda.write(|w| unsafe { w
            .pin().bits(11)
            .connect().connected()
        } );
        twim0_s.frequency.write(|w| w.
            frequency().k400()
        );
    
        twim0_s.enable.write(|w| w.
            enable().enabled()
        );
    
        let manuf_model = twim0_s.reg_read(ADP536X_I2C_ADDR, 0x00);
        assert_eq!(manuf_model, 0x10);
        let silicon_rev = twim0_s.reg_read(ADP536X_I2C_ADDR, 0x01);
        assert_eq!(silicon_rev, 0x08);
    
        ADP536X {
            twim0_s
        }
    }

    pub fn release(self) -> TWIM0_S
    {

        self.twim0_s.enable.write(|w| w.
            enable().disabled()
        );

        self.twim0_s
    }

    pub fn buck_1v8_set(&mut self)
    {
        /* 1.8V equals to 0b11000 = 0x18 according to ADP536X datasheet. */
        let value = 0x18;

        self.twim0_s.reg_write_mask(ADP536X_I2C_ADDR, ADP536X_BUCK_OUTPUT,
            ADP536X_BUCK_OUTPUT_VOUT_BUCK_MSK,
            value << ADP536X_BUCK_OUTPUT_VOUT_BUCK_SFT);
    }

    pub fn buckbst_3v3_set(&mut self)
    {
        /* 3.3V equals to 0b10011 = 0x13, according to ADP536X datasheet. */
        let value = 0x13;
    
        self.twim0_s.reg_write_mask(ADP536X_I2C_ADDR, ADP536X_BUCKBST_OUTPUT,
            ADP536X_BUCKBST_OUTPUT_VOUT_BUCKBST_MSK,
            value << ADP536X_BUCKBST_OUTPUT_VOUT_BUCKBST_SFT);
    }
    
    pub fn buckbst_enable(&mut self, enable: bool)
    {
        let value = if enable {1} else {0};
    
        self.twim0_s.reg_write_mask(ADP536X_I2C_ADDR, ADP536X_BUCKBST_CFG,
            ADP536X_BUCKBST_CFG_EN_BUCKBST_MSK,
            value << ADP536X_BUCKBST_CFG_EN_BUCKBST_SFT);
    }
}
