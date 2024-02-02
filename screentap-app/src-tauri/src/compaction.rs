
use std::path::PathBuf;

/**
 * Compact screenshot images to MP4 video
 */
pub struct CompactionHelper {
    app_data_dir: PathBuf,
    db_filename_path: PathBuf,
}

impl CompactionHelper {

    pub fn new(app_data_dir: PathBuf, db_filename_path: PathBuf) -> Self {
        Self {
            app_data_dir,
            db_filename_path,
        }
    }
    pub fn should_compact_screenshots(&self) -> bool {
        // TODO
        false
    }

    /**
     * 1. Check if incoming is full (>= 150 images.  30 images per min, 5 mins)
     * 2. Create target dir if it doesn’t exist
     * 3. Create the MP4 file
     * 4. In a single DB transaction, update all entries in the incoming directory to
     *     1. Set IS_MP4 = True
     *     2. Add the Frame ID
     *     3. Update the filename to the MP4 file
     *     4. Delete all entries in the incoming dir
     */
    pub fn compact_screenshots_to_mp4(&self) -> () {
        // TODO
    }

}



#[cfg(test)]
mod test {

    use super::CompactionHelper;
    use std::path::PathBuf;
    use image::{ImageBuffer, Rgba};

    
    #[test]
    fn test_should_compact_screenshots() {
        let app_data_dir = PathBuf::from("/tmp");
        let db_filename_path = PathBuf::from("test.db");

        create_dummy_image_files(&app_data_dir, 2);

        // Test setup
        //   - Create 150 files in the app_data_dir
        //   - Create a DB with the expected schema and rows
    
        let compaction_helper = CompactionHelper::new(app_data_dir.clone(), db_filename_path.to_path_buf());
        let result = compaction_helper.should_compact_screenshots();
        assert_eq!(result, false);
    }

    fn create_dummy_image_files(target_dir: &PathBuf, num_files: u32) -> () {
        // Create real image files because we eventually want to test the actual compaction into mp4
        for i in 0..num_files {
            let filename = format!("{}.png", i);
            let target_file = target_dir.join(filename);
            let img = ImageBuffer::<Rgba<u8>, Vec<u8>>::new(10, 10);
            img.save(target_file).unwrap();
        }

    }

}

