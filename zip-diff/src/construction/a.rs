use anyhow::Result;

pub mod a1;
pub mod a2;
pub mod a3;
pub mod a4;
pub mod a5;

pub fn main() -> Result<()> {
    a1::main()?;
    a2::main()?;
    a3::main()?;
    a4::main()?;
    a5::main()?;
    Ok(())
}
