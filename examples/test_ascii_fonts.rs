//! Test all ASCII art fonts render correctly
//!
//! Run with: cargo run --example test_ascii_fonts

use corgiterm_core::all_fonts;

fn main() {
    let fonts = all_fonts();
    println!("═══════════════════════════════════════════════════════════");
    println!("  CorgiTerm ASCII Art Font Test - {} fonts available", fonts.len());
    println!("═══════════════════════════════════════════════════════════\n");

    let mut passed = 0;
    let mut failed = 0;

    for font in fonts {
        match font.render("AB") {
            Ok(result) => {
                let lines: Vec<&str> = result.lines().collect();
                let non_empty = lines.iter().any(|l| !l.trim().is_empty());

                if non_empty {
                    println!("✓ {} (height: {}, actual lines: {})", font.name, font.height, lines.len());
                    passed += 1;
                } else {
                    println!("✗ {} - Empty output", font.name);
                    failed += 1;
                }
            }
            Err(e) => {
                println!("✗ {} - Error: {}", font.name, e);
                failed += 1;
            }
        }
    }

    println!("\n═══════════════════════════════════════════════════════════");
    println!("  Results: {} passed, {} failed", passed, failed);
    println!("═══════════════════════════════════════════════════════════");

    // Show sample renders for a few fonts
    println!("\n--- Sample Renders ---\n");

    let sample_fonts = ["Standard", "Slant", "Doom", "Digital", "Bubble"];
    for name in sample_fonts {
        if let Some(font) = fonts.iter().find(|f| f.name == name) {
            println!("{}:", font.name);
            match font.render("CORGI") {
                Ok(output) => println!("{}", output),
                Err(e) => println!("  Error: {}", e),
            }
            println!();
        }
    }
}
