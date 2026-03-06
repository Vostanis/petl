use std::fs::File;
use std::io::{self, Write};
use std::process::{Command, Stdio};

fn dump_and_compress(
    host: &str,
    port: u16,
    user: &str,
    password: &str,
    db_name: &str,
    output_path: &str,
) -> io::Result<()> {
    // Build the connection URL for pg_dump
    let conn_url = format!("postgresql://{user}:{password}@{host}:{port}/{db_name}");

    let mut pg_dump = Command::new("pg_dump")
        .arg("--format=custom") // -Fc: pg_dump's own compressed format
        .arg("--no-password") // never prompt, fail loudly instead
        .arg(&conn_url)
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit()) // surface pg_dump errors directly
        .spawn()?;

    let output = File::create(output_path)?;

    // zstd encoder with multithreading
    let mut encoder = zstd::Encoder::new(output, 19)?;
    encoder.multithread(0)?; // 0 = all available cores

    if let Some(mut stdout) = pg_dump.stdout.take() {
        io::copy(&mut stdout, &mut encoder)?;
    }

    encoder.finish()?;

    let status = pg_dump.wait()?;
    if !status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("pg_dump failed with status: {status}"),
        ));
    }

    Ok(())
}
