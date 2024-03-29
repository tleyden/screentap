extern crate screen_ocr_swift_rs;

use std::path::PathBuf;
use crate::db;
use rusqlite::params;


// The maximum number of image files allowed to accumulate before compacting to an MP4
// 
// TODO: make this configurable via toml file.  
//       I need to make it larger for dev purposes, but it should be more like 100
pub const DEFAULT_MAX_IMAGE_FILES: u32 = 500;


/**
 * Compact screenshot images to MP4 video
 */
pub struct CompactionHelper {
    app_data_dir: PathBuf,
    db_filename_path: PathBuf,
    max_image_files: u32,
}

impl CompactionHelper {

    pub fn new(app_data_dir: PathBuf, db_filename_path: PathBuf, max_image_files: u32) -> Self {

        if !app_data_dir.is_dir() {
            panic!("app_data_dir is not a directory");
        }

        Self {
            app_data_dir,
            db_filename_path,
            max_image_files,
        }
    }

    fn count_png_files(&self) -> u32 {
        let png_files = self.get_png_files();
        png_files.len() as u32
    }

    fn get_png_files(&self) -> Vec<PathBuf> {
        let mut png_files = Vec::new();
        for entry in std::fs::read_dir(&self.app_data_dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == "png" {
                        png_files.push(path);
                    }
                }
            }
        }
        png_files
    }

    /**
     * Get the .png files in the app_data_dir ordered chronologically, oldest to newest
     */
    fn get_png_files_chronologically(&self) -> Vec<PathBuf> {
        let mut png_files = self.get_png_files();
        png_files.sort_by_key(|a| a.metadata().unwrap().modified().unwrap());
        png_files
    }

    fn update_db_rows_with_mp4_file(&self, png_files: &[PathBuf], target_mp4_fn: &str) {

        // Open connection to DB
        let conn = db::get_db_conn(
            self.app_data_dir.as_path(), 
            self.db_filename_path.as_path()
        );

        // Loop over the image file paths, for each one, update the row to set the 
        // mp4_file_path to the target_mp4_fn for each matching row
        for (frame_id, png_file) in png_files.iter().enumerate() {

            let png_file_str = png_file.to_str().unwrap();

            // Update the row to set the mp4_file_path to the target_mp4_fn for each matching row
            let update_result = conn.execute(
                "UPDATE documents SET mp4_file_path = ?, mp4_frame_id = ? WHERE file_path = ?",
                params![target_mp4_fn, frame_id, png_file_str],
            );

            match update_result {
                Ok(_) => (),
                Err(e) => panic!("Error setting mp4_file_path in DB: {} for png file {}", e, png_file_str),
            }

        }

    }


    /**
     * Is it time to run compaction?
     */
    pub fn should_compact_screenshots(&self) -> bool {
            
        // Count the number of .png files in self.app_data_dir
        let num_png_files = self.count_png_files();

        num_png_files > self.max_image_files
    }

    /**
     * Given a directory of images, write them to an mp4
     * TODO: return a Result<>
     */
    pub fn compact_screenshots_in_dir_to_mp4(&self, target_mp4_fn: PathBuf, use_bitrate_key: bool) {  
        
        screen_ocr_swift_rs::write_images_in_dir_to_mp4(
            self.app_data_dir.to_str().unwrap(), 
            target_mp4_fn.to_str().unwrap(),
            use_bitrate_key
        );

    }

    /**
     * 1. Check if incoming is full (>= 150 images.  30 images per min, 5 mins)
     * 2. Create target dir if it doesn’t exist
     * 3. Create the MP4 file
     * 4. In a single DB transaction, update all entries in the incoming directory to
     *     1. Set IS_MP4 = True
     *     2. Add the Frame ID
     *     3. Update the filename to the MP4 file
     * 5. Delete all entries in the incoming dir
     */
    pub fn compact_screenshots_to_mp4(&self, target_mp4_fn: PathBuf, use_bitrate_key: bool) {

        if !self.should_compact_screenshots() {
            return;
        }


        // TODO: create the MP4 file in a subdirectory divided by date (year/month/day/hour)
        //       but for now, just keep it flat

        // Get the list of png files in the app dir ordered chronologically.
        // Even though we don't pass this in to the swift function, due to limitations
        // of the rust->swift interface that cannot pass arrays of params, we can
        // be assured that this list won't change because this is happening on 
        // the same thread that is writing the screenshots to disk.  We can use this
        // list for updating the DB
        let png_files = self.get_png_files_chronologically();

        // Make sure these files are all in the DB, otherwise throw an error

        // TODO: it should skip compacting any png files that are already in an mp4 file, based on the DB state.
        //       this prevents write multiplication in a worst case scenario where png images end up in multiple mp4 files.
        //       Delete any PNG files that are on disk but already in an MP4 file according to the DB.

        // Create an MP4 file for the png files in the directory
        self.compact_screenshots_in_dir_to_mp4(target_mp4_fn.clone(), use_bitrate_key);
        
        // Update the DB
        self.update_db_rows_with_mp4_file(&png_files, target_mp4_fn.to_str().unwrap());

        // Delete all png files in the incoming dir
        // TODO: only enable this after we know we can access the images from the mp4 files
        self.cleanup_screenshot_images(&png_files)


    }

    fn cleanup_screenshot_images(&self, png_files: &Vec<PathBuf>) {
        for png_file in png_files {
            std::fs::remove_file(png_file.as_path()).unwrap();
        }
    }

}



#[cfg(test)]
mod test {

    use super::CompactionHelper;
    use std::path::PathBuf;
    use image::{ImageBuffer, Rgba};
    use rand::{Rng, thread_rng};
    use crate::db;
    use chrono::Local;
    use tempfile::tempdir;
    use std::collections::HashMap;


    // Use a small number of image files for testing, because I have to make
    // the images relatively big to avoid the isReadyForMoreMediaData=False error
    const MAX_IMAGE_FILES: u32 = 1;

    /**
     * This tests the compact_screenshots_to_mp4() method
     */
    #[test]
    fn test_compact_screenshots_to_mp4() {

        println!("Running test_compact_screenshots_to_mp4");

        // Generate a random temp directory
        let tmp_dir = tempdir().unwrap();
        let app_data_dir = PathBuf::from(tmp_dir.path());

        // Create the app_data_dir if it doesn't exist
        if !app_data_dir.exists() {
            std::fs::create_dir_all(app_data_dir.as_path()).unwrap();
        }

        // Create paths to test assets
        let db_filename = PathBuf::from("test.db");
        let target_mp4_file = app_data_dir.join("test_compact_screenshots_to_mp4.mp4");

        // Delete the target mp4 file if it exists
        if target_mp4_file.exists() {
            std::fs::remove_file(target_mp4_file.as_path()).unwrap();
        }

        // Create a bunch of image files
        let image_file_paths = create_dummy_image_files(
            &app_data_dir, 
            MAX_IMAGE_FILES + 1,
            true
        );

        let base64_images: HashMap<String, String> = image_file_paths.iter().map(
            |image_file_path| {
                let base64_string = db::get_screenshot_as_base64_string(
                    image_file_path.as_path().to_str().unwrap(), // Convert PathBuf to &str
                    "", // Placeholder for any additional parameters
                    -1  // Placeholder for any additional parameters
                );
                (image_file_path.as_path().to_str().unwrap().to_string(), base64_string)
            }
        ).collect();

        // Create a DB with the image file paths
        create_db_with_image_files(
            &image_file_paths,
            &app_data_dir, 
            &db_filename
        );

        // Create a compaction helper
        let compaction_helper = CompactionHelper::new(
            app_data_dir.clone(), 
            db_filename.to_path_buf(),
            MAX_IMAGE_FILES
        );

        // Run compaction
        println!("Running compaction and saving to {:?}", target_mp4_file.as_path());
        compaction_helper.compact_screenshots_to_mp4(
            target_mp4_file.clone(),
            // Use bitrate key since this potentially runs on Github Actions, which runs 
            // on hardware that doessn't support the AVVideoQualityKey
            true
        );

        // Assert that the mp4 file was created and has non-zero size
        let metadata = std::fs::metadata(target_mp4_file.as_path()).unwrap();
        let mp4_file_size = metadata.len();
        assert!(mp4_file_size > 0);

        // Assert that the DB was updated to point to the MP4 file with the correct frame IDs
        assert_screenshots_point_to_mp4_in_db(
            &image_file_paths, 
            &app_data_dir, 
            &db_filename,
            &target_mp4_file
        );

        // Assert that the screenshot png files were deleted
        assert_screenshot_files_deleted(&image_file_paths);

        // Get all the screenshots, and ensure that the base64 of the images
        // are what should be expected 
        let all_screenshots = db::get_all_screenshots(
            app_data_dir.as_path(), 
            db_filename.as_path(), 
            1000
        );

        // Loop over screenshots and compare base64 images to values stored in base64_images
        for (_, screenshot) in all_screenshots.unwrap().iter().enumerate() {
            let expected_base_64 = base64_images.get(screenshot.get_file_path()).unwrap().as_str();
            let actual_base_64 = screenshot.get_base64_image();

            // if the length of the actual base64 is greater than or equal to half the length of the expected base64, then we can assume that the actual base64 is a valid image
            assert!(
                actual_base_64.len() >= expected_base_64.len() / 2, 
                "Base64 images do not match for screenshot {}", screenshot.get_file_path()
            );

            // TODO: this won't work!  It's lossy compression .. how do I compare them?
            // Convert to image data and find the percentage of pixels that are the same 
            // Also make sure the dimesions of the images are the same
            // assert_eq!(expected_base_64, actual_base_64, "Base64 images do not match for screenshot {}", screenshot.get_file_path());
            
        }


    }

    /**
     * Compact a hardcoded directory of image files to an mp4
     */
    // #[test] // - disable this since it only works on my machine
    #[allow(dead_code)]
    fn test_compact_screenshots_in_harcoded_dir_to_mp4() {

        println!("Running test_compact_screenshots_in_harcoded_dir_to_mp4");

        let images_dir = PathBuf::from("/Users/tleyden/Development/screentap/local_test_dataset/video_compression");
        let db_filename: PathBuf = PathBuf::from("dummmy.db");
        let target_mp4_file = images_dir.join("test_compact_screenshots_in_dir_to_mp4.mp4");

        if target_mp4_file.exists() {
            std::fs::remove_file(target_mp4_file.as_path()).unwrap();
        }

        let compaction_helper = CompactionHelper::new(
            images_dir, 
            db_filename,
            1
        );

        compaction_helper.compact_screenshots_in_dir_to_mp4(
            target_mp4_file,
            // Use bitrate key since this potentially runs on Github Actions, which runs 
            // on hardware that doessn't support the AVVideoQualityKey
            false
        );

    }

    /**
     * This is an isolated test on the compact_screenshots_in_dir_to_mp4() method
     */
    #[test]
    fn test_compact_screenshots_in_dir_to_mp4() {
        
        println!("Running test_compact_screenshots_in_dir_to_mp4");

        // Generate a random temp directory
        let tmp_dir = tempdir().unwrap();
        let db_filename: PathBuf = PathBuf::from("test.db");
        let app_data_dir = PathBuf::from(tmp_dir.path());

        // Create the app_data_dir if it doesn't exist
        if !app_data_dir.exists() {
            std::fs::create_dir_all(app_data_dir.as_path()).unwrap();
        }

        let target_mp4_file = app_data_dir.join("test_compact_screenshots_in_dir_to_mp4.mp4");
        if target_mp4_file.exists() {
            std::fs::remove_file(target_mp4_file.as_path()).unwrap();
        }

        create_dummy_image_files(
            &app_data_dir, 
            MAX_IMAGE_FILES + 1,
            false
        );
    
        let compaction_helper = CompactionHelper::new(
            app_data_dir.clone(), 
            db_filename.to_path_buf(),
            MAX_IMAGE_FILES
        );

        compaction_helper.compact_screenshots_in_dir_to_mp4(
            target_mp4_file.clone(),
            // Use bitrate key since this potentially runs on Github Actions, which runs 
            // on hardware that doessn't support the AVVideoQualityKey
            true 
        );

        // Assert that the mp4 file was created
        assert!(target_mp4_file.exists());

        // Assert that the mp4 has nonzero size
        let metadata = std::fs::metadata(target_mp4_file.as_path()).unwrap();
        let mp4_file_size = metadata.len();
        assert!(mp4_file_size > 0);

        // TODO: assert that the mp4 file has expected number of frames.  Could use 
        // a swift bridge for this



    }

    #[test]
    fn test_should_compact_screenshots() {

        println!("Running test_should_compact_screenshots()");
        
        let app_data_dir = PathBuf::from("/tmp");
        let db_filename_path = PathBuf::from("test.db");

        create_dummy_image_files(
            &app_data_dir, 
            MAX_IMAGE_FILES + 1,
            false,
    );
    
        let compaction_helper = CompactionHelper::new(
            app_data_dir.clone(), 
            db_filename_path.to_path_buf(),
            MAX_IMAGE_FILES
        );
        let result = compaction_helper.should_compact_screenshots();
        assert_eq!(result, true);
    }


    fn assert_screenshots_point_to_mp4_in_db(image_file_paths: &Vec<PathBuf>, app_data_dir: &PathBuf, db_filename_path: &PathBuf, target_mp4_file: &PathBuf) {

        let screenshot_records_result = db::get_all_screenshots(
            app_data_dir.as_path(), 
            db_filename_path, 
            1000
        );

        let screenshot_records = match screenshot_records_result {
            Ok(screenshot_records) => screenshot_records,
            Err(e) => {
                panic!("Error getting screenshots from DB: {}", e);
            },
        };

        // Build a hashmap of the screenshot_records, using image path as key and record as value
        let screenshot_records_map: std::collections::HashMap<String, db::ScreenshotRecord> = 
            screenshot_records.iter().cloned().map(
                |record| 
                    (String::from(record.get_file_path()), record)
            ).collect();
        

        // Enumerate over the image file paths and get the path and frame_id
        for (frame_id, image_file_path) in image_file_paths.iter().enumerate() {

            // Check that its record exists in the screenshot_records_map
            let record = screenshot_records_map.get(image_file_path.to_str().unwrap());
            assert!(record.is_some(), "Record not found for image file path: {:?}", image_file_path);

            //   Assert that the path points to the MP4 file (maybe each record can have both an image path and an mp4 path until switching to uuids?)
            let mp4_file_path = record.unwrap().get_mp4_file_path();
            assert_eq!(mp4_file_path, target_mp4_file.to_str().unwrap(), "The paths do not match: expected {}, got {}", mp4_file_path, target_mp4_file.to_str().unwrap());

            //   Assert that the frame ID is set to expected val
            assert_eq!(record.unwrap().get_mp4_frame_id(), frame_id as i32, "The frame IDs do not match: expected {}, got {}", frame_id, record.unwrap().get_mp4_frame_id());

        }


    }

    fn create_db_with_image_files(image_file_paths: &Vec<PathBuf>, app_data_dir: &PathBuf, db_filename_path: &PathBuf) {

        // Deelte the db file if it exists
        let db_filename_fq_path = app_data_dir.join(db_filename_path);
        if db_filename_fq_path.exists() {
            std::fs::remove_file(db_filename_fq_path.as_path()).unwrap();
        }

        // Create the database if it doesn't exist
        match db::create_db(&app_data_dir, db_filename_path) {
            Ok(()) => (),
            Err(e) => assert!(false, "Failed to create db: {}", e),
        }

        // Loop over image file paths and save each one to DB
        for image_file_path in image_file_paths {
            
            let now = Local::now().naive_utc();

            // Save screenshot meta to the DB
            let save_result = db::save_screenshot_meta(
                image_file_path.as_path(), 
                "fake ocr text",
                &app_data_dir,
                db_filename_path,
                now
            );
        
            match save_result {
                Ok(screenshot_id) => println!("Screenshot saved to DB successfully.  id: {}", screenshot_id),
                Err(e) => assert!(false, "Failed to save screenshot to db: {}", e),
            }

        }

    }

    fn create_dummy_image_files(target_dir: &PathBuf, num_files: u32, with_random_pixels: bool) -> Vec<PathBuf> {

        let mut image_files = Vec::new();

        // Create real image files because we eventually want to test the actual compaction into mp4
        for i in 0..num_files {
            let filename = format!("{}.png", i);
            let target_file = target_dir.join(filename);
            // let mut img = ImageBuffer::<Rgba<u8>, Vec<u8>>::new(10, 10);
            let mut img = ImageBuffer::<Rgba<u8>, Vec<u8>>::new(800, 600);

            // If with_random_pixels is true, fill the image with random pixels
            if with_random_pixels {
                let mut rng = thread_rng();
                for (_, _, pixel) in img.enumerate_pixels_mut() {
                    // Generate random values for RGBA components
                    let red = rng.gen_range(0..=255);
                    let green = rng.gen_range(0..=255);
                    let blue = rng.gen_range(0..=255);
                    let alpha = 255; // Full opacity
            
                    // Set the pixel to a random color
                    *pixel = Rgba([red, green, blue, alpha]);
                }
            }

            let _ = img.save(target_file.clone());
            image_files.push(target_file);
        }

        // Return the list of image files
        image_files

    }

    fn assert_screenshot_files_deleted(image_file_paths: &Vec<PathBuf>) {
        for image_file_path in image_file_paths {
            assert!(!image_file_path.exists(), "Image file still exists: {:?}", image_file_path);
        }
    }

}

