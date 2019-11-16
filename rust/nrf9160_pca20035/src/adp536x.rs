/*!
Reference:
* fw-nrfconnect-nrf/drivers/adp536x/adp536x.c
*/

use crate::i2cregacc::I2CRegAcc;

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

pub struct ADP536X<I: I2CRegAcc>
{
    i2c: I
}

impl<I: I2CRegAcc> ADP536X<I>
{
    pub fn new(mut i2c: I) -> ADP536X<I>
    {
        i2c.enable(12, 11, true);
    
        let manuf_model = i2c.reg_read(ADP536X_I2C_ADDR, 0x00);
        assert_eq!(manuf_model, 0x10);
        let silicon_rev = i2c.reg_read(ADP536X_I2C_ADDR, 0x01);
        assert_eq!(silicon_rev, 0x08);
    
        ADP536X {
            i2c
        }
    }

    pub fn release(mut self) -> I
    {
        self.i2c.disable();

        self.i2c
    }

    pub fn buck_1v8_set(&mut self)
    {
        /* 1.8V equals to 0b11000 = 0x18 according to ADP536X datasheet. */
        let value = 0x18;

        self.i2c.reg_write_mask(ADP536X_I2C_ADDR, ADP536X_BUCK_OUTPUT,
            ADP536X_BUCK_OUTPUT_VOUT_BUCK_MSK,
            value << ADP536X_BUCK_OUTPUT_VOUT_BUCK_SFT);
    }

    pub fn buckbst_3v3_set(&mut self)
    {
        /* 3.3V equals to 0b10011 = 0x13, according to ADP536X datasheet. */
        let value = 0x13;
    
        self.i2c.reg_write_mask(ADP536X_I2C_ADDR, ADP536X_BUCKBST_OUTPUT,
            ADP536X_BUCKBST_OUTPUT_VOUT_BUCKBST_MSK,
            value << ADP536X_BUCKBST_OUTPUT_VOUT_BUCKBST_SFT);
    }
    
    pub fn buckbst_enable(&mut self, enable: bool)
    {
        let value = if enable {1} else {0};
    
        self.i2c.reg_write_mask(ADP536X_I2C_ADDR, ADP536X_BUCKBST_CFG,
            ADP536X_BUCKBST_CFG_EN_BUCKBST_MSK,
            value << ADP536X_BUCKBST_CFG_EN_BUCKBST_SFT);
    }
}
