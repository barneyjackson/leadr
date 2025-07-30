use leadr_api::ApiDoc;
use std::fs;
use utoipa::OpenApi;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let doc = ApiDoc::openapi();
    let json = doc.to_pretty_json()?;
    
    // Create docs directory if it doesn't exist
    fs::create_dir_all("docs")?;
    
    // Write OpenAPI spec
    fs::write("docs/openapi.json", json)?;
    
    println!("OpenAPI specification generated at docs/openapi.json");
    Ok(())
}