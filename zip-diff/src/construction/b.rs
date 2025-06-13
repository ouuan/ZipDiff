use anyhow::Result;

pub mod b1;
pub mod b2;
pub mod b3;
pub mod b4;

pub fn main() -> Result<()> {
    b1::main()?;
    b2::main()?;
    b3::main()?;
    b4::main()?;
    Ok(())
}
