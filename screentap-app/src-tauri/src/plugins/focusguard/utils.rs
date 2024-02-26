use std::fs::File;
use std::io::copy;
use reqwest::blocking::Client;
use std::error::Error;
use regex::Regex;

pub fn download_file(url: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let client = Client::new();
    let mut response = client.get(url)
        .send()? // Send the request and get the response
        .error_for_status()?; // Ensure we got a 2xx response

    let mut dest = File::create(output_path)?; // Create the output file

    // Stream the response body directly into the file
    copy(&mut response, &mut dest)?;

    Ok(())
}

pub fn find_first_number(text: &str) -> Option<i32> {
    // Create a Regex to find numbers
    let re = Regex::new(r"\d+").unwrap();

    // Search for the first match
    re.find(text)
        .and_then(|mat| mat.as_str().parse::<i32>().ok())

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_finds_the_first_number() {
        assert_eq!(find_first_number("This is a test 10, 145, 100034"), Some(10));
    }

    #[test]
    fn it_finds_the_first_number2() {
        assert_eq!(find_first_number("This is a test 100034, 8"), Some(100034));
    }

    #[test]
    fn it_returns_none_when_no_number_is_present() {
        assert_eq!(find_first_number("This is a test, no numbers here!"), None);
    }

}