/*!
Reference:
* fw-nrfconnect-nrf/boards/arm/nrf9160_pca20035/board_secure.c
*/

#![no_std]

mod adp536x;
mod i2cregacc;

pub fn power_mgmt_init(twim: nrf91::TWIM2_S) -> nrf91::TWIM2_S
{
    let mut adp536x = adp536x::ADP536X::new(twim);

    adp536x.buck_1v8_set();
    adp536x.buckbst_3v3_set();
    adp536x.buckbst_enable(true);

    adp536x.release()
}
