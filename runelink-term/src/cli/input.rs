use std::io::Write;

pub fn read_input(prompt: &str) -> std::io::Result<String> {
    let mut stdout = std::io::stdout();
    let stdin = std::io::stdin();

    stdout.write(prompt.as_bytes())?;
    stdout.flush()?;

    let mut buf = String::new();
    stdin.read_line(&mut buf)?;

    // strip trailing newline
    while buf.ends_with('\n') || buf.ends_with('\r') {
        buf.pop();
    }
    println!();
    Ok(buf)
}
