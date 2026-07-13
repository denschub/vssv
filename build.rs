use std::error::Error;

use vergen_git2::{Emitter, Git2};

fn main() -> Result<(), Box<dyn Error>> {
    let git2 = Git2::all_git();
    Emitter::default().add_instructions(&git2)?.emit()?;

    Ok(())
}
