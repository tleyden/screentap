use rusqlite::{params, Connection, Result};
use chrono::Utc;
use std::path::Path;


/**
 * Helper function to create the DB if it doesn't exist
 */
pub fn create_db(dataset_root: &str, db_filename: &str) -> Result<()> {

    let dataset_root_path = Path::new(dataset_root);
    let db_filename_fq_path = dataset_root_path.join(db_filename);
    println!("Creating db_filename: {} if it doesn't exist", db_filename_fq_path.to_str().unwrap());

    let conn = Connection::open(db_filename_fq_path.to_str().unwrap())?;

    // Create a table with the desired columns
    conn.execute(
        "CREATE TABLE IF NOT EXISTS documents (
                id INTEGER PRIMARY KEY,
                timestamp TIMESTAMP NOT NULL,
                ocr_text TEXT NOT NULL,
                file_path TEXT NOT NULL
            )",
        [],
    )?;

    // Enable full-text search on the OCR text column
    conn.execute(
        "CREATE VIRTUAL TABLE IF NOT EXISTS ocr_text_index 
            USING fts5(ocr_text);",
        [],
    )?;

    Ok(())

}

/**
 * Helper function to save screenshot meta to the DB
 */
pub fn save_screenshot_meta(screenshot_file_path: &str, ocr_text: &str, dataset_root: &str, db_filename: &str) -> Result<()> {

    let dataset_root_path = Path::new(dataset_root);
    let db_filename_fq_path = dataset_root_path.join(db_filename);
    let conn = Connection::open(db_filename_fq_path.to_str().unwrap())?;

    let now = Utc::now().naive_utc();

    conn.execute(
        "INSERT INTO documents (timestamp, ocr_text, file_path) VALUES (?1, ?2, ?3)",
        params![now.timestamp(), ocr_text, screenshot_file_path],
    )?;

    // Insert the OCR text into the full-text search index
    conn.execute(
        "INSERT INTO ocr_text_index (ocr_text) VALUES (?1)",
        [ocr_text],
    )?;

    Ok(())

}
