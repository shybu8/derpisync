use std::collections::BTreeSet;
use std::fs;
use std::io::{BufRead, Write};

pub fn load_index(filepath: &str) -> std::io::Result<BTreeSet<String>> {
    let file = match fs::File::open(filepath) {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(BTreeSet::new()),
        Err(e) => return Err(e),
    };
    let reader = std::io::BufReader::new(file);
    Ok(reader.lines().filter_map(Result::ok).collect())
}

pub fn save_index(filepath: &str, items: BTreeSet<String>) -> std::io::Result<()> {
    let mut file = fs::File::create(filepath)?;
    for item in items {
        writeln!(file, "{}", item)?;
    }
    Ok(())
}
