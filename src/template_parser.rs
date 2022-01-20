// https://github.com/fflorent/nom_locate/ Line numbers?

use std::path::PathBuf;

use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, take_till, take_until},
    character::complete::char,
    combinator::{map, not},
    multi::many0,
    sequence::{delimited, pair, terminated},
    IResult,
};

const OPENING_MARK: &str = "!!%";
const CLOSING_MARK: &str = "%!!";

#[derive(Debug, Clone)]
pub struct Template {
    pub settings: String,
    pub blocks: Vec<TemplateBlock>,
}

impl TryFrom<PathBuf> for Template {
    type Error = anyhow::Error;

    fn try_from(value: PathBuf) -> Result<Self, Self::Error> {
        let file_contents = std::fs::read_to_string(value)?;

        match template(&file_contents) {
            Ok((_, blocks)) => Ok(Template {
                blocks,
                settings: String::new(),
            }),
            Err(e) => anyhow::bail!("{}", e), // Rethrow the error (lifetimes stuff)
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TemplateBlock {
    Text(String),
    DirectiveBlock(DirectiveBlock),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DirectiveBlock {
    directive: String,
    blocks: Vec<TemplateBlock>,
}

// TODO: FIx this (base case, not wrapped in OPENING_MARK CLOSING_MARK )
/*
 * text    TemplateBlock::Text
 * ( ... ) TemplateBlock::Directive block
 * text    TemplateBlock::Text
 * ( ... ) TemplateBlock::Directive block
 * text    TemplateBlock::Text
 */

fn template(input: &str) -> IResult<&str, Vec<TemplateBlock>> {
    many0(alt((
        map(directive_block, TemplateBlock::DirectiveBlock),
        map(text, |t| TemplateBlock::Text(t.to_string())),
    )))(input)
}

fn text(input: &str) -> IResult<&str, &str> {
    map(is_not(OPENING_MARK), |t: &str| t.trim())(input)
}

/*
 * text
 * directive_block
 */
fn template_block(input: &str) -> IResult<&str, TemplateBlock> {
    alt((
        map(directive_block, TemplateBlock::DirectiveBlock),
        map(is_not(CLOSING_MARK), |t: &str| {
            TemplateBlock::Text(t.trim().to_string())
        }),
    ))(input)
}

/*
 * ( directive template_blocks )
 */
fn directive_block(input: &str) -> IResult<&str, DirectiveBlock> {
    let (rest, (directive, template_blocks)) = delimited(
        tag(OPENING_MARK),
        pair(directive, many0(template_block)),
        tag(CLOSING_MARK),
    )(input)?;

    Ok((
        rest,
        DirectiveBlock {
            directive: directive.to_string(),
            blocks: template_blocks,
        },
    ))
}

fn directive(input: &str) -> IResult<&str, &str> {
    terminated(map(is_not("\n"), |t: &str| t.trim()), char('\n'))(input)
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
        let expected = vec![
            TemplateBlock::Text("textbefore".to_string()),
            TemplateBlock::DirectiveBlock(DirectiveBlock {
                directive: "directive1".to_string(),
                blocks: vec![TemplateBlock::Text("text1".to_string())],
            }),
            TemplateBlock::Text("textbetween".to_string()),
            TemplateBlock::DirectiveBlock(DirectiveBlock {
                directive: "directive2".to_string(),
                blocks: vec![TemplateBlock::Text("text2".to_string())],
            }),
            TemplateBlock::Text("textafter".to_string()),
        ];

        let result = template(input.as_str()).unwrap().1;
        assert_eq!(result, expected);
    }

    #[test]
    fn test_template_block() {
        let input = format!(
            "{} directive1 \n{} directive2 \ntext{} asd{}",
            OPENING_MARK, OPENING_MARK, CLOSING_MARK, CLOSING_MARK
        );
        let expected = TemplateBlock::DirectiveBlock(DirectiveBlock {
            directive: "directive1".to_string(),
            blocks: vec![
                TemplateBlock::DirectiveBlock(DirectiveBlock {
                    directive: "directive2".to_string(),
                    blocks: vec![TemplateBlock::Text("text".to_string())],
                }),
                TemplateBlock::Text("asd".to_string()),
            ],
        });

        let result = template_block(input.as_str()).unwrap().1;
        assert_eq!(result, expected);
    }
}
