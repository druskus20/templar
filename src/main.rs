#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

mod templating;

use std::path::PathBuf;
use templating::*;

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn test_templar() {
        let config = templating::ParserConfig {
            include: "include".to_string(),
            transform: "transform".to_string(),
            to: "to".to_string(),
            end: "end".to_string(),
            odelim: "<%".to_string(),
            cdelim: "%>".to_string(),
            ..Default::default()
        };

        let template_str = indoc!(
            r#"
                <% include header.html %>
                <% include footer.html %>
            "#
        );

        let t = templating::Template::parse_str(&config, template_str).unwrap();
        dbg!(&t);
    }
}
