// https://github.com/fflorent/nom_locate/ Line numbers?

use super::template::TemplateBlock;
use super::{directive, template};
use super::{directive::Generator, template::Template};
use anyhow::Result;
use nom::character::complete::{multispace0, space0};
use nom::error::ParseError;
use nom::sequence::{self, tuple};
use nom::Parser;
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

// TODO: make this a struct at some point
// mod constants {
//     pub(super) const ODELIM: &str = "!!%";
//     pub(super) const CDELIM: &str = "%!!";
//     pub(super) const COMMENT: &str = "##";
//     pub(super) const IF: &str = "if";
//     pub(super) const ELSE: &str = "else";
//     pub(super) const END: &str = "end";
//     pub(super) const INCLUDE: &str = "include";
// }

#[derive(Debug, Clone)]
pub struct ParserConfig {
    pub(super) odelim: String,
    pub(super) cdelim: String,
    pub(super) comment: String,
    pub(super) if_: String,
    pub(super) else_: String,
    pub(super) end: String,
    pub(super) include: String,
}

impl Default for ParserConfig {
    fn default() -> Self {
        ParserConfig {
            odelim: "!!%".to_string(),
            cdelim: "%!!".to_string(),
            comment: "##".to_string(),
            if_: "if".to_string(),
            else_: "else".to_string(),
            end: "end".to_string(),
            include: "include".to_string(),
        }
    }
}

#[rustfmt::skip]
pub(super) fn parse_template_str<'a>(c: &'a ParserConfig, i: &'a str) -> IResult<&'a str, Vec<TemplateBlock>> {
    many0(
        alt((
            template_chunk(&c),
            // Text
            map(is_not(c.odelim.as_str()), |t: &str| {
                let boxed_text: TemplateBlock = Rc::new(t.trim().to_string());
                boxed_text
            })
        )),
    )(i)
}

/*
 * text_inside_chunk | if_block | if_else_block ...
 */
fn template_chunk<'a>(
    c: &'a ParserConfig,
) -> impl FnMut(&'a str) -> IResult<&'a str, TemplateBlock> {
    alt((
        include_block(c),
        // Text
        map(is_not(c.cdelim.as_str()), |t: &str| {
            let boxed_text: TemplateBlock = Rc::new(t.to_string());
            boxed_text
        }),
    ))
}

/*
 * < include str >
 */
fn include_block<'a>(
    c: &'a ParserConfig,
) -> impl FnMut(&'a str) -> IResult<&'a str, TemplateBlock> {
    move |i| {
        let (i, (_, path)) = delimited(
            odelim(&c),
            pair(tag(c.include.as_str()), is_not(c.cdelim.as_str())),
            cdelim(&c),
        )(i)?;

        Ok((
            i,
            Rc::new(directive::Include {
                path: path.trim().to_string(),
            }),
        ))
    }
}

fn odelim<'a>(c: &'a ParserConfig) -> impl FnMut(&'a str) -> IResult<&'a str, ()> {
    move |i| {
        let (i, _) = whitespaced(tag(c.odelim.as_str()))(i)?;
        Ok((i, ()))
    }
}

fn cdelim<'a>(c: &'a ParserConfig) -> impl FnMut(&'a str) -> IResult<&'a str, ()> {
    move |i| {
        let (i, _) = whitespaced(pair(tag(c.cdelim.as_str()), multispace0))(i)?;
        Ok((i, ()))
    }
}

fn whitespaced<'a, O1, E, P>(p: P) -> impl FnMut(&'a str) -> IResult<&'a str, O1, E>
where
    P: Parser<&'a str, O1, E>,
    E: ParseError<&'a str>,
{
    delimited(space0, p, space0)
}

/* < if condition >
 *   template_block
 *   template_block
 *   ...
 * < end >
 */
#[rustfmt::skip]
fn if_block<'a>(c: &'a ParserConfig) -> impl FnMut(&'a str) -> IResult<&'a str, TemplateBlock> {
    move |i| {
        // (&str, (&str, Vec<Rc<dyn templating::directive::Generator>>, &str))
        let (i, (condition, blocks, _)) = tuple((
            if_line(c), 
            many0(template_chunk(c)),
            end_tag(c),
        ))(i)?;

        Ok((
            i,
            Rc::new(directive::If {
                condition: condition.to_string(),
                blocks,
            }),
        ))
    }
}

/* < if condition > */
fn if_line<'a>(c: &'a ParserConfig) -> impl FnMut(&'a str) -> IResult<&'a str, &'a str> {
    move |i| {
        let (i, (_, condition)) = delimited(
            odelim(&c),
            pair(tag(c.if_.as_str()), is_not(c.cdelim.as_str())),
            cdelim(&c),
        )(i)?;
        Ok((i, condition.trim()))
    }
}

// < end >
fn end_tag<'a>(c: &'a ParserConfig) -> impl FnMut(&'a str) -> IResult<&'a str, ()> {
    move |i| {
        let (i, _) = delimited(odelim(c), tag(c.end.as_str()), cdelim(c))(i)?;
        Ok((i, ()))
    }
}

/*
 * < if condition >
 *   template_block
 *   template_block
 *   ...
 * < else >
 *  template_block
 *  template_block
 *  ...
 * < end >
 */
fn if_else_block<'a>(
    c: &'a ParserConfig,
) -> impl FnMut(&'a str) -> IResult<&'a str, TemplateBlock> {
    move |i| {
        // (&str, (&str, Vec<Rc<dyn templating::directive::Generator>>, &str))
        let (i, condition) = if_line(c)(i)?;
        let (i, if_blocks) = many0(template_chunk(c))(i)?;
        let (i, _) = else_tag(c)(i)?;
        let (i, else_blocks) = many0(template_chunk(c))(i)?;
        let (i, _) = end_tag(c)(i)?;

        Ok((
            i,
            Rc::new(directive::IfElse {
                condition: condition.trim().to_string(),
                if_blocks,
                else_blocks,
            }),
        ))
    }
}

// < else >
fn else_tag<'a>(c: &'a ParserConfig) -> impl FnMut(&'a str) -> IResult<&'a str, ()> {
    move |i| {
        let (i, _) = delimited(odelim(c), tag(c.else_.as_str()), cdelim(c))(i)?;
        Ok((i, ()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;
    use lazy_static::lazy_static;
    use std::fmt::format;

    lazy_static! {
        static ref PARSER_CONFIG: ParserConfig = {
            ParserConfig {
                odelim: "!%".to_string(),
                cdelim: "%!".to_string(),
                include: "include".to_string(),
                if_: "if".to_string(),
                else_: "else".to_string(),
                end: "end".to_string(),
                comment: "//".to_string(),
            }
        };
    }

    #[test]
    fn test_odelim() {
        let input = "!%";
        let expected = Ok(("", ()));
        let result = odelim(&PARSER_CONFIG)(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_cdelim() {
        let input = "%!";
        let expected = Ok(("", ()));
        let result = cdelim(&PARSER_CONFIG)(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_include_block() {
        let input = "!% include path %!";
        let expected = directive::Include {
            path: "path".to_string(),
        };

        let result = include_block(&PARSER_CONFIG)(input).unwrap().1;
        //let result = include_block(input.as_str()).unwrap().1;
        assert_eq!(format!("{:?}", result), format!("{:?}", expected));
    }

    #[test]
    fn test_if_block() {
        let input = indoc!(
            r#"
                !% if condition %!
                text
                text
                !% end %!
            "#
        );

        let expected = directive::If {
            condition: "condition".to_string(),
            blocks: vec![Rc::new("text\ntext\n")],
        };

        let result = if_block(&PARSER_CONFIG)(input).unwrap().1;
        assert_eq!(format!("{:?}", result), format!("{:?}", expected));
    }

    #[test]
    fn test_if_line() {
        let input = "!% if condition %!";
        let expected = "condition";

        let result = if_line(&PARSER_CONFIG)(input).unwrap().1;
        assert_eq!(format!("{:?}", result), format!("{:?}", expected));
    }

    #[test]
    fn test_else_tag() {
        let input = "!% else %!";
        let result = else_tag(&PARSER_CONFIG)(input).unwrap().1;
    }

    #[test]
    fn test_end_tag() {
        let input = "!% end %!";
        let result = end_tag(&PARSER_CONFIG)(input).unwrap().1;
    }

    #[test]
    fn test_if_else_block() {
        let input = indoc!(
            r#"
                !% if condition %!
                text
                !% else %!
                text
                !% end %!
            "#
        );

        let expected = directive::IfElse {
            condition: "condition".to_string(),
            if_blocks: vec![Rc::new("text\n")],
            else_blocks: vec![Rc::new("text\n")],
        };

        let result = if_else_block(&PARSER_CONFIG)(input).unwrap().1;
        assert_eq!(format!("{:?}", result), format!("{:?}", expected));
    }

    #[test]
    fn test_include_block() {
        let input = "!% include ./some/path %!";
        let expected = directive::Include {
            path: "./some/path".to_string(),
        };

        let result = include_block(&PARSER_CONFIG)(input).unwrap().1;
        assert_eq!(format!("{:?}", result), format!("{:?}", expected));
    }
}
