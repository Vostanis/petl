use anyhow::Result;
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use std::sync::Arc;
use tracing::{debug, error, trace};

/// Reads a `.json` file from `path`.
pub async fn read_json<T: serde::de::DeserializeOwned>(path: &str) -> Result<T> {
    trace!(filepath=%path, "reading file");
    let file = tokio::fs::read(path).await?;

    trace!(filepath=%path, "file read - deserializing bytes");
    let data: T = serde_json::from_slice(&file)?;
    Ok(data)
}

/// Unzip a `.zip` file, `zip_file`, to a target directory, `dir`.
///
/// `std::fs::create_dir_all(to_dir)?` is used in creating `to_dir` path,
/// so directories will be created, as necessary, by the unzip() function.
pub async fn unzip(zip_file: &str, dir: &str) -> anyhow::Result<()> {
    debug!("unzipping {zip_file} to {dir}");

    // Open the file, but `std::fs` has to be used, instead of tokio.
    let file = std::fs::File::open(zip_file)?;
    let archive = zip::ZipArchive::new(file).map_err(|e| {
        error!("failed to open zip file at {}: {}", zip_file, e);
        e
    })?;
    let zip_length = archive.len();

    // Async wrappings for archive.
    let archive = Arc::new(std::sync::Mutex::new(archive));

    // Ensure the target directory exists.
    tokio::fs::create_dir_all(dir).await?;

    // Parallel iteration across zipped files.
    (0..zip_length).into_par_iter().for_each(|i| {
        let archive = archive.clone();
        let mut archive = archive.lock().expect("unlock zip archive");
        let mut file = archive.by_index(i).expect("file from zip archive");
        let outpath = format!("{dir}/{}", file.mangled_name().display());
        let outdir = std::path::Path::new(&outpath)
            .parent()
            .expect("parent directory of output path");

        // If output directory does not exist, create it.
        if !outdir.exists() {
            std::fs::create_dir_all(&outdir).expect("failed to create directory");
        }

        // Extract the file.
        let mut outfile = std::fs::File::create(&outpath).expect("creation of output file");
        trace!("extracting {} to {}", file.name(), outpath);
        std::io::copy(&mut file, &mut outfile).expect("copying of zip file to output");
    });

    debug!("{zip_file} unzipped to {dir}");

    Ok(())
}

/// Return a vector of String-type file paths for a given directory path.
pub fn file_list(
    dir: &str,
    starts_with: Option<&str>,
    ends_with: Option<&str>,
) -> Result<Vec<String>> {
    let mut file_paths = Vec::new();

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(file_name) = path.file_name().and_then(|name| name.to_str()) {
                // if there is starting pattern input
                if let Some(starting_pattern) = starts_with {
                    // ignore the file path if it DOESN'T start with the pattern
                    if !file_name.starts_with(starting_pattern) {
                        continue;
                    }
                };

                // if there is ending pattern input
                if let Some(ending_pattern) = ends_with {
                    // ignore the file path if it DOESN'T end in the pattern
                    if !file_name.ends_with(ending_pattern) {
                        continue;
                    }
                };

                file_paths.push(
                    path.to_str()
                        .expect("failed to convert file path to str type")
                        .to_string(),
                );
            }
        }
    }

    Ok(file_paths)
}

/// Convert a u32 timestamp to a chrono::NaiveDate.
pub fn convert_timestamp(timestamp: u32) -> chrono::NaiveDate {
    chrono::DateTime::from_timestamp(timestamp.into(), 0)
        .expect("failed to convert timestamp integer")
        .date_naive()
}

/// Convert a &String to a chrono::NaiveDate (so that it can inserted directly as DATE)
pub fn convert_date_type(str_date: &String) -> anyhow::Result<chrono::NaiveDate> {
    let date = chrono::NaiveDate::parse_from_str(&str_date, "%Y-%m-%d").map_err(|err| {
        tracing::error!("failed to parse date string; expected form YYYYMMDD - received: {str_date}, error({err})");
        err
    })?;
    Ok(date)
}

/// Reads a delimited file from `path`.
pub fn read_delimited<Row: serde::de::DeserializeOwned>(
    path: &str,
    has_headers: bool,
    delimiter: u8,
) -> Result<Vec<Row>> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(has_headers)
        .delimiter(delimiter)
        // .flexible(true) // Allow records with varying number of fields
        // .quoting(false) // TSV files typically don't use quotes
        .from_path(path)?;
    let mut table: Vec<Row> = vec![];
    for result in reader.deserialize() {
        let row: Row = result?;
        table.push(row)
    }
    Ok(table)
}

/// Convenience function for CSV files
pub fn read_csv<Row: serde::de::DeserializeOwned>(
    path: &str,
    has_headers: bool,
) -> Result<Vec<Row>> {
    read_delimited(path, has_headers, b',')
}

/// Convenience function for TSV files
pub fn read_tsv<Row: serde::de::DeserializeOwned>(
    path: &str,
    has_headers: bool,
) -> Result<Vec<Row>> {
    read_delimited(path, has_headers, b'\t')
}

/// If a csv file fails to read, retry without headers.
pub fn read_csv_autodetect<Row: serde::de::DeserializeOwned>(path: &str) -> Result<Vec<Row>> {
    read_csv::<Row>(path, true).or_else(|_| read_csv::<Row>(path, false))
}

/// If a tsv file fails to read, retry without headers.
pub fn read_tsv_autodetect<Row: serde::de::DeserializeOwned>(path: &str) -> Result<Vec<Row>> {
    read_tsv::<Row>(path, true).or_else(|_| read_tsv::<Row>(path, false))
}

/// Encode a jpg file into a string.
pub fn stringify_jpg(path: &str) -> Result<String> {
    use base64::Engine;

    // Read the bytes from the byte
    let image_bytes = std::fs::read(path)?;

    // Encode bytes to base64 string
    let s = base64::engine::general_purpose::STANDARD.encode(&image_bytes);

    Ok(s)
}
