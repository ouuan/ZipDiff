use anyhow::Result;

mod a;
mod b;
mod c;
mod utils;

fn main() -> Result<()> {
    a::main()?;
    b::main()?;
    c::main()?;
    Ok(())
}
