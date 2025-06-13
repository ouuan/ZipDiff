use anyhow::Result;

pub mod c1;
pub mod c2;
pub mod c3;
pub mod c4;
pub mod c5;

pub fn main() -> Result<()> {
    c1::main()?;
    c2::main()?;
    c3::main()?;
    c4::main()?;
    c5::main()?;
    Ok(())
}
