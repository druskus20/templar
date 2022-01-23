// https://github.com/fflorent/nom_locate/ Line numbers?

use super::directive;
use super::{directive::Generator, template::Template};
use anyhow::Result;
use nom::sequence::{self, tuple};
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
 * text_outisde_chunk | template_chunk
 */
fn template(input: &str) -> IResult<&str, Vec<Rc<dyn Generator>>> {
    many0(alt((template_chunk, text_outside_chunk)))(input)
}

/*
 * text_inside_chunk | if_block | if_else_block ...
 */
fn template_chunk(input: &str) -> IResult<&str, Rc<dyn Generator>> {
    alt((
        useless_block_with_text,
        text_inside_chunk,
        if_block,
        //if_else_block,
    ))(input)
}

/*
 * < include condition >
 */
fn include_block(input: &str) -> IResult<&str, Rc<dyn Generator>> {
    let (rest, path) =
        delimited(tag(OPENING_MARK), is_not(CLOSING_MARK), tag(CLOSING_MARK))(input)?;

    Ok((
        rest,
        Rc::new(directive::Include {
            path: path.to_string(),
        }),
    ))
}

/*
 * < if condition
 *   template_block
 *   template_block
 *   ...
 * >
 * < else
 *  template_block
 *  template_block
 *  ...
 * >
 */
fn if_else_block(input: &str) -> IResult<&str, Rc<dyn Generator>> {
    let (rest, ((condition, if_blocks), else_blocks)) = pair(
        delimited(
            tag(OPENING_MARK),
            pair(if_line, many0(template_chunk)),
            tag(CLOSING_MARK),
        ),
        delimited(tag(OPENING_MARK), many0(template_chunk), tag(CLOSING_MARK)),
    )(input)?;

    Ok((
        rest,
        Rc::new(directive::IfElse {
            condition: condition.to_string(),
            if_blocks,
            else_blocks,
        }),
    ))
}

/* < if condition
 *   template_block
 *   template_block
 *   ...
 * >
 */
fn if_block(input: &str) -> IResult<&str, Rc<dyn Generator>> {
    let (rest, (condition, blocks)) = delimited(
        tag(OPENING_MARK),
        pair(if_line, many0(template_chunk)),
        tag(CLOSING_MARK),
    )(input)?;

    Ok((rest, Rc::new(directive::If { condition, blocks })))
}

/* if condition \n */
fn if_line(input: &str) -> IResult<&str, String> {
    let (rest, (_, condition, _)) = tuple((tag("if"), is_not("\n"), tag("\n")))(input)?;
    Ok((rest, condition.to_string()))
}

/*
 * < useless_text
 *   template_block
 *   template_block
 *   ...
 * >
 */
fn useless_block_with_text(input: &str) -> IResult<&str, Rc<dyn Generator>> {
    let (rest, (useless_text, blocks)) = delimited(
        tag(OPENING_MARK),
        pair(useless_text, many0(template_chunk)),
        tag(CLOSING_MARK),
    )(input)?;

    Ok((
        rest,
        Rc::new(directive::UselessBlockWithText {
            useless_text,
            blocks,
        }),
    ))
}

/* text \n */
fn useless_text(input: &str) -> IResult<&str, String> {
    terminated(
        map(is_not("\n"), |t: &str| t.trim().to_string()),
        char('\n'),
    )(input)
}

// TODO: Change these 2
fn text_outside_chunk(input: &str) -> IResult<&str, Rc<dyn Generator>> {
    map(is_not(OPENING_MARK), |t: &str| {
        let boxed_text: Rc<dyn Generator> = Rc::new(t.trim().to_string());
        boxed_text
    })(input)
}

fn text_inside_chunk(input: &str) -> IResult<&str, Rc<dyn Generator>> {
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
                    useless_text: "directive1".to_string(),
                    blocks: vec![Rc::new("text1")],
                }),
                Rc::new("textbetween".to_string()),
                Rc::new(directive::UselessBlockWithText {
                    useless_text: "directive2".to_string(),
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
            useless_text: "directive1".to_string(),
            blocks: vec![
                Rc::new(directive::UselessBlockWithText {
                    useless_text: "directive2".to_string(),
                    blocks: vec![Rc::new("text".to_string())],
                }),
                Rc::new("text2".to_string()),
            ],
        });

        let result = template_chunk(input.as_str()).unwrap().1;
        assert_eq!(format!("{:?}", result), format!("{:?}", expected));
    }
}
