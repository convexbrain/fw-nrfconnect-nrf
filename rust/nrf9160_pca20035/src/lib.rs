/*!
Reference:
* fw-nrfconnect-nrf/boards/arm/nrf9160_pca20035/board_secure.c
* fw-nrfconnect-nrf/drivers/adp536x/adp536x.c
* fw-nrfconnect-nrf/drivers/adp536x/adp536x.h
*/

#![no_std]

mod adp536x;
mod i2cregacc;

/* Definition of VBUS current limit values. */
const ADP536X_VBUS_ILIM_500MA: u8 = 0x07;

/* Definition of charging current values. */
const ADP536X_CHG_CURRENT_320MA: u8 = 0x1F;

/* Definition of overcharge protection threshold values. */
const ADP536X_OC_CHG_THRESHOLD_400MA: u8 = 0x07;

pub fn power_mgmt_init(twim: nrf91::TWIM2_S) -> nrf91::TWIM2_S
{
    let mut adp536x = adp536x::ADP536X::new(twim);

    adp536x.buck_1v8_set();

    adp536x.buckbst_3v3_set();

    adp536x.buckbst_enable(true);

	/* Enables discharge resistor for buck regulator that brings the voltage
	 * on its output faster down when it's inactive. Needed because some
	 * components require to boot up from ~0V.
	 */
    adp536x.buck_discharge_set(true);

    /* Sets the VBUS current limit to 500 mA. */
    adp536x.vbus_current_set(ADP536X_VBUS_ILIM_500MA);

    /* Sets the charging current to 320 mA. */
    adp536x.charger_current_set(ADP536X_CHG_CURRENT_320MA);

    /* Sets the charge current protection threshold to 400 mA. */
    adp536x.oc_chg_current_set(ADP536X_OC_CHG_THRESHOLD_400MA);

    adp536x.charging_enable(true);

    adp536x.release()
}
