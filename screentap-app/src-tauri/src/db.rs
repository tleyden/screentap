use rusqlite::{params, Connection, Result};
use chrono::NaiveDateTime;
use std::{path::Path, collections::HashMap, path::PathBuf};
use base64::engine::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;

/**
 * Struct to represent screenshot records in the DB
 * 
 * The derived Clone trait is used to create a copy of the struct
 */
#[derive(Clone)]
pub struct ScreenshotRecord {
    id: i32,
    // timestamp: NaiveDateTime,  // TODO: use this instead of i32.  Currently panics with called `Result::unwrap()` on an `Err` value: InvalidColumnType(1, "timestamp", Integer)
    timestamp: i32,    
    ocr_text: String,

    // the file_path is the path to the screenshot png file
    file_path: String,

    // the mp4_file_path is the path to the mp4 file, post compaction
    mp4_file_path: String,

    base64_image: String,
}

impl ScreenshotRecord {
    pub fn get_file_path(&self) -> &str {
        &self.file_path
    }
    pub fn get_mp4_file_path(&self) -> &str {
        &self.mp4_file_path
    }
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

fn get_db_conn(dataset_root: &Path, db_filename: &Path) -> Connection {
    let db_filename_fq_path = dataset_root.join(db_filename);
    let path_str = db_filename_fq_path.to_str().expect("Failed to get db_filename_fq_path");
    Connection::open(path_str).expect("Failed to open db connection")
}

/**
 * Helper function to create the DB if it doesn't exist
 */
pub fn create_db(dataset_root: &Path, db_filename: &Path) -> Result<()> {

    let conn = get_db_conn(dataset_root, db_filename);

    // Create a table with the desired columns
    conn.execute(
        "CREATE TABLE IF NOT EXISTS documents (
                id INTEGER PRIMARY KEY,
                timestamp TIMESTAMP NOT NULL,
                ocr_text TEXT NOT NULL,
                file_path TEXT NOT NULL,
                mp4_file_path TEXT NOT NULL
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
pub fn save_screenshot_meta(screenshot_file_path: &Path, ocr_text: &str, dataset_root: &Path, db_filename: &Path, now: NaiveDateTime) -> Result<()> {

    let conn = get_db_conn(dataset_root, db_filename);

    let screenshot_file_path_str = screenshot_file_path.to_str().expect("Failed to get screenshot_file_path_str");

    conn.execute(
        "INSERT INTO documents (timestamp, ocr_text, file_path, mp4_file_path) VALUES (?1, ?2, ?3, ?4)",
        params![now.timestamp(), ocr_text, screenshot_file_path_str, ""],
    )?;

    // Insert the OCR text into the full-text search index
    conn.execute(
        "INSERT INTO ocr_text_index (ocr_text) VALUES (?1)",
        [ocr_text],
    )?;

    Ok(())

}

/**
 * Helper function to get a screenshot from the DB by ID
 */
pub fn get_screenshot_by_id(dataset_root: &Path, db_filename: &Path, target_id: i32) -> Result<Vec<ScreenshotRecord>, rusqlite::Error> {

    let conn = get_db_conn(dataset_root, db_filename);

    let mut stmt = conn.prepare("SELECT id, timestamp, ocr_text, file_path, mp4_file_path FROM documents WHERE id = ? ORDER BY timestamp DESC")?;
    let screenshots = stmt.query_map(params![target_id], |row| {

        // open the file_path and convert to base64
        let file_path_str: String = row.get(3).expect("Failed to get file_path");
        let mp4_file_path_str: String = row.get(4).expect("Failed to get mp4_file_path");
        let file_path = PathBuf::from(file_path_str.clone());
        let base64_image: String = load_file_as_base_64(file_path.as_path(), dataset_root);

        Ok(ScreenshotRecord {
            id: row.get(0)?,
            timestamp: row.get(1)?,
            ocr_text: row.get(2)?,
            file_path: file_path_str,
            mp4_file_path: mp4_file_path_str,
            base64_image,
        })
    })?
    .collect::<Result<Vec<_>, _>>()?;

    Ok(screenshots)

}

/**
 * Helper function to get all screenshots from the DB
 */
pub fn get_all_screenshots(dataset_root: &Path, db_filename: &Path, limit: i32) -> Result<Vec<ScreenshotRecord>, rusqlite::Error> {

    let conn = get_db_conn(dataset_root, db_filename);

    let mut stmt = conn.prepare("SELECT id, timestamp, ocr_text, file_path, mp4_file_path FROM documents ORDER BY timestamp DESC LIMIT ?")?;
    let screenshots = stmt.query_map(params![limit], |row| {

        // open the file_path and convert to base64
        let file_path_str: String = row.get(3).expect("Failed to get file_path");
        let mp4_file_path_str: String = row.get(4).expect("Failed to get mp4_file_path");

        let file_path = PathBuf::from(file_path_str.clone());
        let base64_image: String = load_file_as_base_64(file_path.as_path(), dataset_root);

        Ok(ScreenshotRecord {
            id: row.get(0)?,
            timestamp: row.get(1)?,
            ocr_text: row.get(2)?,
            file_path: file_path_str,
            mp4_file_path: mp4_file_path_str,
            base64_image,
        })
    })?
    .collect::<Result<Vec<_>, _>>()?;

    Ok(screenshots)

}

/**
 * Helper function to search screenshots in the db matching ocr term
 */
pub fn search_screenshots_ocr(term: &str, dataset_root: &Path, db_filename: &Path, limit: i32) -> Result<Vec<ScreenshotRecord>, rusqlite::Error> {

    let conn = get_db_conn(dataset_root, db_filename);

    let mut stmt = conn.prepare(r#"
        SELECT ocr_text_index.rowid, d.timestamp, d.ocr_text, d.file_path, d.mp4_file_path 
        FROM ocr_text_index 
        JOIN documents d on d.id = ocr_text_index.rowid 
        WHERE ocr_text_index.ocr_text MATCH ?
        ORDER BY rank, d.timestamp DESC
        LIMIT ?
    "#)?;

    let screenshots = stmt.query_map(params![term, limit], |row| {

        // open the file_path and convert to base64
        let file_path_str: String = row.get(3).expect("Failed to get file_path");
        let mp4_file_path_str: String = row.get(4).expect("Failed to get mp4_file_path");

        let file_path = PathBuf::from(file_path_str.clone());
        let base64_image: String = load_file_as_base_64(file_path.as_path(), dataset_root);

        Ok(ScreenshotRecord {
            id: row.get(0)?,
            timestamp: row.get(1)?,
            ocr_text: row.get(2)?,
            file_path: file_path_str,
            mp4_file_path: mp4_file_path_str,
            base64_image,
        })
    })?
    .collect::<Result<Vec<_>, _>>()?;

    Ok(screenshots)

}

fn load_file_as_base_64(file_path: &Path, dataset_root: &Path) -> String {
    let dataset_root_path = Path::new(dataset_root);  // no longer needed
    let file_path_fq = dataset_root_path.join(file_path);
    let file_contents = std::fs::read(file_path_fq).unwrap();
    BASE64.encode(file_contents)
}
