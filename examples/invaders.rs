use rustyi8080 as cpu;


fn main() -> std::io::Result<()> {
    let f = cpu::read_file("invaders_rom/invaders.h")?;

    Ok(())
}



