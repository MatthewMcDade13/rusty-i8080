use rustyi8080 as cpu;



fn fun(h: u8, l: u8) {
    let r = register_pair(h, l);
    let r = r + 1;
    println!("RP+1: {}", r);
    let low = r as u8;
    let high = (r >> 8) as u8;
    println!("RP: {}", register_pair(high, low));
}

const fn register_pair(high: u8, low: u8) -> u16 {
    ((high as u16) << 8) | low as u16
}

fn main() -> std::io::Result<()> {
    let f = cpu::read_file("invaders_rom/invaders.h")?;

    let x: u16 = 300;
    println!("AS: {}, AND: {}", x as u8, x & 0xFF);

    Ok(())
}



