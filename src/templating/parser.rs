// https://github.com/fflorent/nom_locate/ Line numbers?

use super::{
    directive::{BlockDirective, DoNothingBlock},
    template::{Template, TemplateBlock, TemplateBlockDirective},
};
use anyhow::Result;
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
fn template(input: &str) -> IResult<&str, Vec<TemplateBlock>> {
    many0(alt((
        map(directive_block, TemplateBlock::BlockDirective),
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
        map(directive_block, TemplateBlock::BlockDirective),
        map(is_not(CLOSING_MARK), |t: &str| {
            TemplateBlock::Text(t.trim().to_string())
        }),
    ))(input)
}

/*
 * ( directive template_blocks )
 */
fn directive_block(input: &str) -> IResult<&str, TemplateBlockDirective> {
    let (rest, (directive, blocks)) = delimited(
        tag(OPENING_MARK),
        pair(directive, many0(template_block)),
        tag(CLOSING_MARK),
    )(input)?;

    Ok((
        rest,
        TemplateBlockDirective {
            directive: &(*directive),
            blocks,
        },
    ))
}

fn directive(input: &str) -> IResult<&str, Box<dyn BlockDirective>> {
    let (rest, parsed) = terminated(map(is_not("\n"), |t: &str| t.trim()), char('\n'))(input)?;
    let directive = DoNothingBlock {
        text: parsed.to_string(),
    };
    Ok((rest, Box::new(directive)))
}
/*
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
            TemplateBlock::BlockDirective(TemplateBlockDirective {
                directive: BlockDirective::DoNothing(DoNothingBlock {
                    text: "directive1".to_string(),
                }),
                blocks: vec![TemplateBlock::Text("text1".to_string())],
            }),
            TemplateBlock::Text("textbetween".to_string()),
            TemplateBlock::BlockDirective(TemplateBlockDirective {
                directive: BlockDirective::DoNothing(DoNothingBlock {
                    text: "directive2".to_string(),
                }),
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
        let expected = TemplateBlock::BlockDirective(TemplateBlockDirective {
            directive: BlockDirective::DoNothing(DoNothingBlock {
                text: "directive1".to_string(),
            }),
            blocks: vec![
                TemplateBlock::BlockDirective(TemplateBlockDirective {
                    directive: BlockDirective::DoNothing(DoNothingBlock {
                        text: "directive2".to_string(),
                    }),
                    blocks: vec![TemplateBlock::Text("text".to_string())],
                }),
                TemplateBlock::Text("asd".to_string()),
            ],
        });

        let result = template_block(input.as_str()).unwrap().1;
        assert_eq!(result, expected);
    }
}*/
