use std::io::Write;

pub fn read_input(prompt: &str) -> std::io::Result<Option<String>> {
    let mut stdout = std::io::stdout();
    let stdin = std::io::stdin();

    stdout.write_all(prompt.as_bytes())?;
    stdout.flush()?;

    let mut buf = String::new();
    stdin.read_line(&mut buf)?;
    println!();

    let trimmed = buf.trim();

    if trimmed.is_empty() {
        Ok(None)
    } else {
        Ok(Some(trimmed.into()))
    }
}
