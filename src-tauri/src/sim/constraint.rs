use serde::{Deserialize, Serialize};
#[allow(non_camel_case_types)]
use strum::{EnumIter,EnumString};
#[derive(Debug, EnumIter, EnumString, Clone, Copy,Serialize,Deserialize)]
#[allow(non_camel_case_types)]
pub enum Constraint {
    r, //- any register
    d, //- `ldi' register (r16-r31)
    v, //- `movw' even register (r0, r2, ..., r28, r30)
    a, //- `fmul' register (r16-r23)
    w, //- `adiw' register (r24,r26,r28,r30)
    e, //- pointer registers (X,Y,Z)
    b, //- pointer register (Y,Z)
    z, //- Z pointer register increment
    M, //- immediate Value from 0 to 255
    n, //- immediate Value from 0 to 255 ( n = ~M ). Relocation impossible
    s, //- immediate Value from 0 to 7
    P, //- Port address Value from 0 to 63. (in, out)
    p, //- Port address Value from 0 to 31. (cbi, sbi, sbic, sbis)
    K, //- immediate Value from 0 to 63 (used in `adiw', `sbiw')
    i, //- immediate Value
    j, //- 7 bit immediate Value from 0x40 to 0xBF (for 16-bit 'lds'/'sts')
    l, //- signed pc relative offset from -64 to 63
    L, //- signed pc relative offset from -2048 to 2047
    h, //- absolute code address (call, jmp)
    S, //- immediate Value from 0 to 7 (S = s << 4)
    E, //- immediate Value from 0 to 15, shifted left by 4 (des)
    o,  //- Displacement value from 0 to 63 (std,ldd)
}

impl PartialEq for Constraint {
    fn eq(&self, other: &Self) -> bool {
        if self ==other {
            return true;
        }
        false
    }
}
