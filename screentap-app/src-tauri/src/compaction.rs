extern crate screen_ocr_swift_rs;

use std::path::PathBuf;
use crate::db;

// The maximum number of image files allowed to accumulate before compacting to an MP4
pub const DEFAULT_MAX_IMAGE_FILES: u32 = 150;


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
        png_files.sort_by(|a, b| a.metadata().unwrap().modified().unwrap().cmp(&b.metadata().unwrap().modified().unwrap()));
        png_files
    }

    pub fn should_compact_screenshots(&self) -> bool {
            
        // Count the number of .png files in self.app_data_dir
        let num_png_files = self.count_png_files();

        num_png_files > self.max_image_files
    }

    /**
     * Given a directory of images, write them to an mp4
     * TODO: return a Result<>
     */
    pub fn compact_screenshots_in_dir_to_mp4(&self, target_mp4_fn: PathBuf) -> () {  
        
        screen_ocr_swift_rs::write_images_in_dir_to_mp4(
            self.app_data_dir.to_str().unwrap(), 
            target_mp4_fn.to_str().unwrap()
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
    pub fn compact_screenshots_to_mp4(&self, target_mp4_fn: PathBuf) -> () {

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
        // let png_files = self.get_png_files_chronologically();
        // println!("png_files: {:?}", png_files);

        // Make sure these files are in the DB, otherwise throw an error

        // Create an MP4 file for the png files in the directory
        self.compact_screenshots_in_dir_to_mp4(target_mp4_fn.clone());
        
        // Update the DB

        // Delete all png files in the incoming dir


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

    // Use a small number of image files for testing, because I have to make
    // the images relatively big to avoid the isReadyForMoreMediaData=False error
    const MAX_IMAGE_FILES: u32 = 1;

    // compact_screenshots_to_mp4

    /**
     * This tests the compact_screenshots_to_mp4() method
     */
    #[test]
    fn test_compact_screenshots_to_mp4() {

        let app_data_dir = PathBuf::from("/tmp");

        let target_mp4_file = PathBuf::from("/tmp/test_compact_screenshots_to_mp4.mp4");
        if target_mp4_file.exists() {
            std::fs::remove_file(target_mp4_file.as_path()).unwrap();
        }

        let db_filename_path = PathBuf::from("test.db");

        // Delete the target mp4 file if it exists
        // let target_mp4_file = PathBuf::from("/tmp/test_compact_screenshots_to_mp4.mp4");
        if target_mp4_file.exists() {
            std::fs::remove_file(target_mp4_file.as_path()).unwrap();
        }

    
        // Create a bunch of image files
        let image_file_paths = create_dummy_image_files(
            &app_data_dir, 
            MAX_IMAGE_FILES + 1,
            false
        );

        // Create a DB with the image file paths
        create_db_with_image_files(
            image_file_paths,
            &db_filename_path
        );

        // Create a compaction helper
        let compaction_helper = CompactionHelper::new(
            app_data_dir.clone(), 
            db_filename_path.to_path_buf(),
            MAX_IMAGE_FILES
        );

        // Run compaction
        println!("Running compaction and saving to {:?}", target_mp4_file.as_path());
        compaction_helper.compact_screenshots_to_mp4(
            target_mp4_file.clone()
        );

        // Assert that the mp4 file was created and has non-zero size
        let metadata = std::fs::metadata(target_mp4_file.as_path()).unwrap();
        let mp4_file_size = metadata.len();
        assert!(mp4_file_size > 0);

        // Assert that png files were deleted

        // Assert that the DB was updated to point to the MP4 file with the correct frame IDs




    }


    /**
     * This is an isolated test on the compact_screenshots_in_dir_to_mp4() method
     */
    #[test]
    fn test_compact_screenshots_in_dir_to_mp4() {
        let app_data_dir = PathBuf::from("/tmp");
        let db_filename_path = PathBuf::from("test.db");

        let target_mp4_file = PathBuf::from("/tmp/test.mp4");
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
            db_filename_path.to_path_buf(),
            MAX_IMAGE_FILES
        );

        compaction_helper.compact_screenshots_in_dir_to_mp4(
            target_mp4_file.clone()
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

    fn create_db_with_image_files(image_file_paths: Vec<PathBuf>, db_filename_path: &PathBuf) {

        // Deelte the db file if it exists
        let dataset_root = PathBuf::from("/tmp");
        let db_filename_fq_path = dataset_root.join(db_filename_path);
        if db_filename_fq_path.exists() {
            std::fs::remove_file(db_filename_fq_path.as_path()).unwrap();
        }

        // Create the database if it doesn't exist
        match db::create_db(&dataset_root, db_filename_path) {
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
                &dataset_root,
                db_filename_path,
                now
            );
        
            match save_result {
                Ok(()) => println!("Screenshot saved to DB successfully"),
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

            img.save(target_file.clone());
            image_files.push(target_file);
        }

        // Return the list of image files
        image_files

    }

}

