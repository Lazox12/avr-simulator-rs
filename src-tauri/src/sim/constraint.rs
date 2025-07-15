#[allow(non_camel_case_types)]
pub enum Constraint {
    r, //- any register
    d, //- `ldi' register (r16-r31)
    v, //- `movw' even register (r0, r2, ..., r28, r30)
    a, //- `fmul' register (r16-r23)
    w, //- `adiw' register (r24,r26,r28,r30)
    e, //- pointer registers (X,Y,Z)
    b, //- base pointer register and displacement ([YZ]+disp)
    z, //- Z pointer register (for [e]lpm Rd,Z[+])
    M, //- immediate value from 0 to 255
    n, //- immediate value from 0 to 255 ( n = ~M ). Relocation impossible
    N, //- immediate value from 0 to 255. Relocation impossible
    s, //- immediate value from 0 to 7
    P, //- Port address value from 0 to 63. (in, out)
    p, //- Port address value from 0 to 31. (cbi, sbi, sbic, sbis)
    K, //- immediate value from 0 to 63 (used in `adiw', `sbiw')
    i, //- immediate value
    j, //- 7 bit immediate value from 0x40 to 0xBF (for 16-bit 'lds'/'sts')
    l, //- signed pc relative offset from -64 to 63
    L, //- signed pc relative offset from -2048 to 2047
    h, //- absolute code address (call, jmp)
    S, //- immediate value from 0 to 7 (S = s << 4)
    E, //- immediate value from 0 to 15, shifted left by 4 (des)
    Q, //- use this opcode entry if no parameters, else use next opcode entry
}