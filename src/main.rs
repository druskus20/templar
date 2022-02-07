#![allow(dead_code)]

mod config;
mod templating;

fn main() {
    println!("Hello, world!, \"main()\" is not yet implemented, however, however, there are tests");
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
                <% if "something" == "something" %>
                text
                <% if "something" == "NO" %>
                text2
                <% end %>
                <% end %>
                <% if "something" == "something" %>
                text3
                <% end %>
            "#
        );

        let t = templating::Template::parse_str(&config, template_str).unwrap();
        let _ = t.process().unwrap();
        //println!("{}", r);

        let template_str = indoc!(
            r#"
                <% transform input %>
                local text = "wooo";
                return text;
                <% to %>
                text1
                text2
                text3
                <% end %>
            "#
        );

        let t = templating::Template::parse_str(&config, template_str).unwrap();
        let _ = t.process().unwrap();
        //println!("{}", r);
    }
}
