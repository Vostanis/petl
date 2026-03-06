use anyhow::Result;
use petl::fs;
use petl::http::extract::HttpExtractExt;
use petl::session::Session;

#[tokio::main]
async fn main() -> Result<()> {
    // Use a session to: -----------------------------------------
    //
    // 1. download a file
    // 2. read it
    // 3. remove the file
    let sx = Session::default();
    let url = "https://api.ipify.org/?format=json";
    let path = "./my_public_ip.json";
    let _download = sx.http.download_file(url, path).await?;
    let data: serde_json::Value = fs::read_json(path).await?;
    println!("{data:#?}");

    tokio::fs::remove_file(path).await?;

    Ok(())
}
