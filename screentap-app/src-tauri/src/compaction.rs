
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
        let mut count = 0;
        for entry in std::fs::read_dir(&self.app_data_dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == "png" {
                        count += 1;
                    }
                }
            }
        }
        count
    }

    pub fn should_compact_screenshots(&self) -> bool {
            
        // Count the number of .png files in self.app_data_dir
        let num_png_files = self.count_png_files();

        num_png_files > MAX_IMAGE_FILES
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
    use super::MAX_IMAGE_FILES;
    use std::path::PathBuf;
    use image::{ImageBuffer, Rgba};
    use rand::{Rng, thread_rng};

    
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
            let mut img = ImageBuffer::<Rgba<u8>, Vec<u8>>::new(10, 10);
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

