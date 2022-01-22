// https://github.com/fflorent/nom_locate/ Line numbers?

use super::directive;
use super::{directive::Generator, template::Template};
use anyhow::Result;
use std::rc::Rc;

use nom::{
    branch::alt,
    bytes::complete::{is_not, tag},
    character::complete::char,
    combinator::map,
    multi::many0,
    sequence::{delimited, pair, terminated},
    IResult,
};

// Parses a raw template string into a Template
pub(super) fn parse_template(raw_template: &str) -> Result<Template> {
    match template(raw_template) {
        Ok((_, blocks)) => Ok(Template { blocks }),
        Err(e) => anyhow::bail!("{}", e), // Rethrow the error (lifetimes stuff)
    }
}

// PARSER CODE
const OPENING_MARK: &str = "!!%";
const CLOSING_MARK: &str = "%!!";

/*
 * text    TemplateBlock::Text
 * ( ... ) TemplateBlock::BlockDirective
 * text    TemplateBlock::Text
 * ( ... ) TemplateBlock::BlockDirective
 * text    TemplateBlock::Text
 */
fn template(input: &str) -> IResult<&str, Vec<Rc<dyn Generator>>> {
    many0(alt((useless_block_with_text, text_outside_block)))(input)
}

/*
 * text
 * directive_block
 */
fn template_block(input: &str) -> IResult<&str, Rc<dyn Generator>> {
    alt((useless_block_with_text, text_inside_block))(input)
}

/*
 * ( directive template_blocks )
 */
fn useless_block_with_text(input: &str) -> IResult<&str, Rc<dyn Generator>> {
    let (rest, (useless_text, blocks)) = delimited(
        tag(OPENING_MARK),
        pair(useless_text, many0(template_block)),
        tag(CLOSING_MARK),
    )(input)?;

    Ok((
        rest,
        Rc::new(directive::UselessBlockWithText {
            text: useless_text.to_string(),
            blocks,
        }),
    ))
}

fn useless_text(input: &str) -> IResult<&str, &str> {
    let (rest, parsed) = terminated(map(is_not("\n"), |t: &str| t.trim()), char('\n'))(input)?;

    Ok((rest, parsed))
}

// TODO: Change these 2
fn text_outside_block(input: &str) -> IResult<&str, Rc<dyn Generator>> {
    map(is_not(OPENING_MARK), |t: &str| {
        let boxed_text: Rc<dyn Generator> = Rc::new(t.trim().to_string());
        boxed_text
    })(input)
}

fn text_inside_block(input: &str) -> IResult<&str, Rc<dyn Generator>> {
    map(is_not(CLOSING_MARK), |t: &str| {
        let boxed_text: Rc<dyn Generator> = Rc::new(t.trim().to_string());
        boxed_text
    })(input)
}

#[cfg(test)]
mod tests {
    use std::fmt::format;

    use super::*;

    #[test]
    fn test_template() {
        let input = format!(
            r#"
     textbefore
     {} directive1
       text1
     {}
     textbetween
     {} directive2
       text2
     {}
     textafter
     "#,
            OPENING_MARK, CLOSING_MARK, OPENING_MARK, CLOSING_MARK
        );
        let expected = Template {
            blocks: vec![
                Rc::new("textbefore".to_string()),
                Rc::new(directive::UselessBlockWithText {
                    text: "directive1".to_string(),
                    blocks: vec![Rc::new("text1")],
                }),
                Rc::new("textbetween".to_string()),
                Rc::new(directive::UselessBlockWithText {
                    text: "directive2".to_string(),
                    blocks: vec![Rc::new("text2")],
                }),
                Rc::new("textafter".to_string()),
            ],
        };

        let result = Template {
            blocks: template(input.as_str()).unwrap().1,
        };

        assert_eq!(format!("{:?}", result), format!("{:?}", expected));
    }
    #[test]
    fn test_template_block() {
        // < directive1
        //   < directive2
        //     text
        //     >
        //   text2
        // >
        let input = format!(
            "{} directive1 \n{} directive2 \ntext{} text2{}",
            OPENING_MARK, OPENING_MARK, CLOSING_MARK, CLOSING_MARK
        );

        let expected: Rc<dyn Generator> = Rc::new(directive::UselessBlockWithText {
            text: "directive1".to_string(),
            blocks: vec![
                Rc::new(directive::UselessBlockWithText {
                    text: "directive2".to_string(),
                    blocks: vec![Rc::new("text".to_string())],
                }),
                Rc::new("text2".to_string()),
            ],
        });

        let result = template_block(input.as_str()).unwrap().1;
        assert_eq!(format!("{:?}", result), format!("{:?}", expected));
    }
}
