use rusqlite::{params, Connection, Result};
use chrono::NaiveDateTime;
use std::{path::Path, collections::HashMap};
use base64::engine::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;

/**
 * Struct to represent screenshot records in the DB
 */
pub struct ScreenshotRecord {
    id: i32,
    // timestamp: NaiveDateTime,  // TODO: use this instead of i32.  Currently panics with called `Result::unwrap()` on an `Err` value: InvalidColumnType(1, "timestamp", Integer)
    timestamp: i32,    
    ocr_text: String,
    file_path: String,
    base64_image: String,
}

/**
 * Helper fn to convert a vector of ScreenshotRecords to a vector of HashMaps
 */
pub fn create_hashmap_vector(records: &[ScreenshotRecord]) -> Vec<HashMap<String, String>> {
    records.iter().map(screenshot_record_to_hashmap).collect()
}

/**
 * Helper fn to convert a ScreenshotRecord to a HashMap
 */
pub fn screenshot_record_to_hashmap(record: &ScreenshotRecord) -> HashMap<String, String> {
    let mut map = HashMap::new();
    map.insert("id".to_string(), record.id.to_string());
    map.insert("timestamp".to_string(), record.timestamp.to_string());
    map.insert("ocr_text".to_string(), record.ocr_text.clone());
    map.insert("file_path".to_string(), record.file_path.clone());
    map.insert("base64_image".to_string(), record.base64_image.clone());
    map
}

/**
 * Helper function to create the DB if it doesn't exist
 */
pub fn create_db(dataset_root: &str, db_filename: &str) -> Result<()> {

    let dataset_root_path = Path::new(dataset_root);
    let db_filename_fq_path = dataset_root_path.join(db_filename);

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

    conn.execute(
        "CREATE VIRTUAL TABLE IF NOT EXISTS ocr_text_index USING fts5(
            content='documents',
            ocr_text,
            content_rowid='id'
        );",
        [],
    )?;



    Ok(())

}

/**
 * Helper function to save screenshot meta to the DB
 */
pub fn save_screenshot_meta(screenshot_file_path: &str, ocr_text: &str, dataset_root: &str, db_filename: &str, now: NaiveDateTime) -> Result<()> {

    let dataset_root_path = Path::new(dataset_root);
    let db_filename_fq_path = dataset_root_path.join(db_filename);
    let conn = Connection::open(db_filename_fq_path.to_str().unwrap())?;

    // let now = Utc::now().naive_utc();

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

/**
 * Helper function to get all screenshots from the DB
 */
pub fn get_all_screenshots(dataset_root: &str, db_filename: &str, limit: i8) -> Result<Vec<ScreenshotRecord>, rusqlite::Error> {

    let dataset_root_path = Path::new(dataset_root);
    let db_filename_fq_path = dataset_root_path.join(db_filename);
    let conn = Connection::open(db_filename_fq_path.to_str().unwrap())?;

    let mut stmt = conn.prepare("SELECT id, timestamp, ocr_text, file_path FROM documents ORDER BY timestamp DESC LIMIT ?")?;
    let screenshots = stmt.query_map(params![limit], |row| {

        // open the file_path and convert to base64
        let file_path: String = row.get(3)?;
        let base64_image: String = load_file_as_base_64(&file_path, dataset_root);

        Ok(ScreenshotRecord {
            id: row.get(0)?,
            timestamp: row.get(1)?,
            ocr_text: row.get(2)?,
            file_path: file_path,
            base64_image: base64_image,
        })
    })?
    .collect::<Result<Vec<_>, _>>()?;

    Ok(screenshots)

}

/**
 * Helper function to search screenshots in the db matching ocr term
 */
pub fn search_screenshots_ocr(term: &str, dataset_root: &str, db_filename: &str, limit: i8) -> Result<Vec<ScreenshotRecord>, rusqlite::Error> {

    let dataset_root_path = Path::new(dataset_root);
    let db_filename_fq_path = dataset_root_path.join(db_filename);
    let conn = Connection::open(db_filename_fq_path.to_str().unwrap())?;

    let mut stmt = conn.prepare(r#"
        SELECT ocr_text_index.rowid, d.timestamp, d.ocr_text, d.file_path 
        FROM ocr_text_index 
        JOIN documents d on d.id = ocr_text_index.rowid 
        WHERE ocr_text_index.ocr_text MATCH ?
        ORDER BY rank, d.timestamp DESC
        LIMIT ?
    "#)?;

    let screenshots = stmt.query_map(params![term, limit], |row| {

        // open the file_path and convert to base64
        let file_path: String = row.get(3)?;
        let base64_image: String = load_file_as_base_64(&file_path, dataset_root);

        Ok(ScreenshotRecord {
            id: row.get(0)?,
            timestamp: row.get(1)?,
            ocr_text: row.get(2)?,
            file_path: file_path,
            base64_image: base64_image,
        })
    })?
    .collect::<Result<Vec<_>, _>>()?;

    Ok(screenshots)

}

fn load_file_as_base_64(file_path: &str, dataset_root: &str) -> String {
    let dataset_root_path = Path::new(dataset_root);
    let file_path_fq = dataset_root_path.join(file_path);
    let file_contents = std::fs::read(file_path_fq).unwrap();
    let base64_image = BASE64.encode(file_contents);
    base64_image
}
