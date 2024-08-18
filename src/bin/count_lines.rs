use std::path::Path;
use tokei::{Config, Languages};

fn main() {
    // Create a new Languages object
    let mut languages = Languages::new();

    // Define the path to your Git repository
    let repo_path = Path::new(".");

    // Use the default configuration
    let config = Config::default();

    // Get statistics for the repository
    languages.get_statistics(&[repo_path], &[], &config);

    // Print the statistics
    for (name, language) in &languages {
        println!("Language: {}", name);
        println!("  Lines: {}", language.lines());
        println!("  Code: {}", language.code);
        println!("  Comments: {}", language.comments);
        println!("  Blanks: {}", language.blanks);
    }
}
