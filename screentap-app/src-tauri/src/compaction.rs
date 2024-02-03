extern crate screen_ocr_swift_rs;

use std::path::PathBuf;

pub const MAX_IMAGE_FILES: u32 = 10;


/**
 * Compact screenshot images to MP4 video
 */
pub struct CompactionHelper {
    app_data_dir: PathBuf,
    db_filename_path: PathBuf,
}

impl CompactionHelper {

    pub fn new(app_data_dir: PathBuf, db_filename_path: PathBuf) -> Self {

        if !app_data_dir.is_dir() {
            panic!("app_data_dir is not a directory");
        }

        Self {
            app_data_dir,
            db_filename_path,
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

        num_png_files > MAX_IMAGE_FILES
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
    pub fn compact_screenshots_to_mp4(&self) -> () {

        if self.should_compact_screenshots() {
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

        println!("png_files: {:?}", png_files);

        // Create an MP4 file for the png files in the directory
        

        

        // Update the DB

        // Delete all png files in the incoming dir

    }

}



#[cfg(test)]
mod test {

    use super::CompactionHelper;
    use super::MAX_IMAGE_FILES;
    use std::path::PathBuf;
    use image::{ImageBuffer, Rgba};
    use rand::{Rng, thread_rng};

    
    #[test]
    fn test_compact_screenshots_in_dir_to_mp4() {
        let app_data_dir = PathBuf::from("/tmp");
        let db_filename_path = PathBuf::from("test.db");

        create_dummy_image_files(&app_data_dir, MAX_IMAGE_FILES + 1);
    
        let compaction_helper = CompactionHelper::new(
            app_data_dir.clone(), 
            db_filename_path.to_path_buf()
        );

        compaction_helper.compact_screenshots_in_dir_to_mp4(
            PathBuf::from("/tmp/test.mp4")
        );

    }

    #[test]
    fn test_should_compact_screenshots() {
        
        let app_data_dir = PathBuf::from("/tmp");
        let db_filename_path = PathBuf::from("test.db");

        create_dummy_image_files(&app_data_dir, MAX_IMAGE_FILES + 1);
    
        let compaction_helper = CompactionHelper::new(
            app_data_dir.clone(), 
            db_filename_path.to_path_buf()
        );
        let result = compaction_helper.should_compact_screenshots();
        assert_eq!(result, true);
    }

    fn create_dummy_image_files(target_dir: &PathBuf, num_files: u32) -> () {
        // Create real image files because we eventually want to test the actual compaction into mp4
        for i in 0..num_files {
            let filename = format!("{}.png", i);
            let target_file = target_dir.join(filename);
            // let mut img = ImageBuffer::<Rgba<u8>, Vec<u8>>::new(10, 10);
            let mut img = ImageBuffer::<Rgba<u8>, Vec<u8>>::new(3456, 2234);

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
            // img.put_pixel(0, 0, Rgba([255, 0, 0, 255]));
            img.save(target_file).unwrap();
        }

    }

}

