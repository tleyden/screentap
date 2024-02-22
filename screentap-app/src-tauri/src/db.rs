extern crate screen_ocr_swift_rs;

use rusqlite::{params, Connection, Result};
use chrono::NaiveDateTime;
use std::{path::Path, collections::HashMap, path::PathBuf};
use base64::engine::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;
use backtrace::Backtrace;


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

    // The file_path is the fully qualified path to the screenshot png file
    file_path: String,

    // The mp4_file_path is the fully qualified path to the mp4 file, post compaction
    mp4_file_path: String,

    // The frame id in the mp4 file where this screenshot can be found.  If not yet compacted
    // into an mp4, this will be -1
    mp4_frame_id: i32,

    // The screenshot image as a base64 string
    base64_image: String,
}

impl ScreenshotRecord {
    
    // Used by unit tests
    #[allow(dead_code)]
    pub fn get_file_path(&self) -> &str {
        &self.file_path
    }
    // Used by unit tests
    #[allow(dead_code)]
    pub fn get_mp4_file_path(&self) -> &str {
        &self.mp4_file_path
    }

    // Used by unit tests
    #[allow(dead_code)]
    pub fn get_mp4_frame_id(&self) -> i32 {
        self.mp4_frame_id
    }

    #[allow(dead_code)]
    pub fn get_base64_image(&self) -> &str {
        &self.base64_image
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
    map.insert("mp4_file_path".to_string(), record.mp4_file_path.clone());
    map.insert("mp4_frame_id".to_string(), record.mp4_frame_id.to_string());  // TODO: this should be an i32 rather than a String
    map.insert("base64_image".to_string(), record.base64_image.clone());
    map
}

pub fn get_db_conn(dataset_root: &Path, db_filename: &Path) -> Connection {
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
                mp4_file_path TEXT NOT NULL DEFAULT '',
                mp4_frame_id INTEGER NOT NULL DEFAULT -1
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

    // Create a UNIQUE index on the file_path column
    conn.execute(
        "CREATE UNIQUE INDEX IF NOT EXISTS file_path_index ON documents (file_path)",
        [],
    )?;

    Ok(())

}

/**
 * Helper function to save screenshot meta to the DB
 * 
 * Returns a Result with the screenshot_id (primary key)
 */
pub fn save_screenshot_meta(screenshot_file_path: &Path, ocr_text: &str, dataset_root: &Path, db_filename: &Path, now: NaiveDateTime) -> Result<i64> {

    let conn = get_db_conn(dataset_root, db_filename);

    let screenshot_file_path_str = screenshot_file_path.to_str().expect("Failed to get screenshot_file_path_str");

    // TODO: change table name to 'screenshots'
    conn.execute(
        "INSERT INTO documents (timestamp, ocr_text, file_path, mp4_file_path) VALUES (?1, ?2, ?3, ?4)",
        params![now.timestamp(), ocr_text, screenshot_file_path_str, ""],
    )?;

    let last_id = conn.last_insert_rowid();

    // Insert the OCR text into the full-text search index
    conn.execute(
        "INSERT INTO ocr_text_index (ocr_text) VALUES (?1)",
        [ocr_text],
    )?;

    Ok(last_id)

}

/**
 * Helper function to get a screenshot from the DB by ID
 */
pub fn get_screenshot_by_id(dataset_root: &Path, db_filename: &Path, target_id: i32) -> Result<Vec<ScreenshotRecord>, rusqlite::Error> {

    let conn = get_db_conn(dataset_root, db_filename);

    let mut stmt = conn.prepare("SELECT id, timestamp, ocr_text, file_path, mp4_file_path, mp4_frame_id FROM documents WHERE id = ? ORDER BY timestamp DESC")?;
    let screenshots = stmt.query_map(params![target_id], |row| {

        // open the file_path and convert to base64
        let file_path_str: String = row.get(3).expect("Failed to get file_path");
        let mp4_file_path_str: String = row.get(4).expect("Failed to get mp4_file_path");
        let mp4_frame_id: i32 = row.get(5).expect("Failed to get mp4_frame_id");

        let fully_qualified_file_path = dataset_root.join(file_path_str.clone());

        let base64_image: String = get_screenshot_as_base64_string(
            fully_qualified_file_path.to_str().unwrap(), 
            &mp4_file_path_str, 
            mp4_frame_id
        );

        Ok(ScreenshotRecord {
            id: row.get(0)?,
            timestamp: row.get(1)?,
            ocr_text: row.get(2)?,
            file_path: file_path_str,
            mp4_file_path: mp4_file_path_str,
            mp4_frame_id,
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

    let mut stmt = conn.prepare("SELECT id, timestamp, ocr_text, file_path, mp4_file_path, mp4_frame_id FROM documents ORDER BY timestamp DESC LIMIT ?")?;
    let screenshots = stmt.query_map(params![limit], |row| {

        // open the file_path and convert to base64
        let file_path_str: String = row.get(3).expect("Failed to get file_path");
        let mp4_file_path_str: String = row.get(4).expect("Failed to get mp4_file_path");
        let mp4_frame_id: i32 = row.get(5).expect("Failed to get mp4_frame_id");

        let fully_qualified_file_path = dataset_root.join(file_path_str.clone());

        let base64_image: String = get_screenshot_as_base64_string(
            fully_qualified_file_path.to_str().unwrap(), 
            &mp4_file_path_str, 
            mp4_frame_id
        );

        Ok(ScreenshotRecord {
            id: row.get(0)?,
            timestamp: row.get(1)?,
            ocr_text: row.get(2)?,
            file_path: file_path_str,
            mp4_file_path: mp4_file_path_str,
            mp4_frame_id,
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
        SELECT ocr_text_index.rowid, d.timestamp, d.ocr_text, d.file_path, d.mp4_file_path, d.mp4_frame_id
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
        let mp4_frame_id: i32 = row.get(5).expect("Failed to get mp4_frame_id");

        let fully_qualified_file_path = dataset_root.join(file_path_str.clone());

        let base64_image = get_screenshot_as_base64_string(
            fully_qualified_file_path.to_str().unwrap(), 
            &mp4_file_path_str, 
            mp4_frame_id
        );


        Ok(ScreenshotRecord {
            id: row.get(0)?,
            timestamp: row.get(1)?,
            ocr_text: row.get(2)?,
            file_path: file_path_str,
            mp4_file_path: mp4_file_path_str,
            mp4_frame_id,
            base64_image,
        })
    })?
    .collect::<Result<Vec<_>, _>>()?;

    Ok(screenshots)

}

pub fn get_screenshot_as_base64_string(file_path: &str, mp4_file_path: &str, mp4_frame_id: i32) -> String {

    // If there is a non-empty mp4_file_path, then the screenshot has been compacted into an mp4
    if !mp4_file_path.is_empty() {
        get_screenshot_base64_from_mp4(mp4_file_path, mp4_frame_id)
    } else {

        // Does the file_path exists?
        let file_path_check = PathBuf::from(file_path);
        if !file_path_check.exists() {
            let bt = Backtrace::new();

            // If this happens, the screenshot file will not be shown in the UI
            println!("Error: get_screenshot_as_base64_string() called with non-existent file.  Returning empty data for file.  Stack trace:\n{:?}", bt);

            return String::from("");
        }
        
        let file_contents = std::fs::read(file_path).unwrap();
        BASE64.encode(file_contents)    
    }
}

fn get_screenshot_base64_from_mp4(mp4_file_path: &str, mp4_frame_id: i32) -> String {

    let frame_data_option = screen_ocr_swift_rs::extract_frame_from_mp4(
        mp4_file_path, 
        mp4_frame_id as isize
    );

    match frame_data_option {
        Some(frame_data) => {
            BASE64.encode(frame_data)
        },
        None => {
            let bt = Backtrace::new();
            println!("Error: get_screenshot_base64_from_mp4() called with non-existent frame.  Returning empty data for frame.  Stack trace:\n{:?}", bt);
            String::from("")
        }
    }

}

