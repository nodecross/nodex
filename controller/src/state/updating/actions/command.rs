use std::process::Command;
use std::error::Error;

pub fn execute(command: &str) -> Result<(), Box<dyn Error>> {
    println!("Running command '{}'", command);
    
    let output = Command::new("sh")
        .arg("-c")
        .arg(command)
        .output()?;

    if output.status.success() {
        println!("Command executed successfully: {}", String::from_utf8_lossy(&output.stdout));
    } else {
        eprintln!("Command failed: {}", String::from_utf8_lossy(&output.stderr));
    }

    Ok(())
}
