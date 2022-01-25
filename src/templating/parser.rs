// https://github.com/fflorent/nom_locate/ Line numbers?

/*
 * A template parser that allows for runtime configuration using ParserConfig
 */
use super::template::TemplateBlock;
use super::{directive, template};
use super::{directive::Generator, template::Template};
use anyhow::Result;
use nom::character::complete::{multispace0, space0};
use nom::combinator::opt;
use nom::error::ParseError;
use nom::sequence::{self, preceded, tuple};
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

#[derive(Debug, Clone)]
pub struct ParserConfig {
    pub(super) odelim: String,
    pub(super) cdelim: String,
    pub(super) comment: String,
    pub(super) if_: String,
    pub(super) else_: String,
    pub(super) end: String,
    pub(super) include: String,
    pub(super) transform: String,
    pub(super) to: String,
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
            transform: "transform".to_string(),
            to: "to".to_string(),
        }
    }
}

#[rustfmt::skip]
pub(super) fn parse_template_str<'a>(c: &'a ParserConfig, i: &'a str) -> IResult<&'a str, Vec<TemplateBlock>> {
    many0(
        alt((
            template_block(&c),
            // Text
            map(is_not(c.odelim.as_str()), |t: &str| {
                let boxed_text: TemplateBlock = Rc::new(t.trim().to_string());
                boxed_text
            })
        )),
    )(i)
}

/* Either text or some directive */
fn template_block<'a>(
    c: &'a ParserConfig,
) -> impl FnMut(&'a str) -> IResult<&'a str, TemplateBlock> {
    alt((
        include_block(c),
        if_block(c),
        if_else_block(c),
        transform_block(c),
        // NOTE: cdelim? odelim?
        // Text
        map(is_not(c.odelim.as_str()), |t: &str| {
            let boxed_text: TemplateBlock = Rc::new(trim_keep_newline(t));
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
    |i| {
        let (i, (_, path)) = delimited(
            odelim(c),
            pair(tag(c.include.as_str()), is_not(c.cdelim.as_str())),
            cdelim(c),
        )(i)?;

        Ok((
            i,
            Rc::new(directive::Include {
                path: path.trim().to_string(),
            }),
        ))
    }
}

/* < */
fn odelim<'a>(c: &'a ParserConfig) -> impl FnMut(&'a str) -> IResult<&'a str, ()> {
    |i| {
        // Technically, by the time we parse odelim, the text before has already been parsed
        // in template_block. (and then trimmed manually)
        let (i, _) = whitespaced(tag(c.odelim.as_str()))(i)?;
        Ok((i, ()))
    }
}

/* space > space \n */
fn cdelim<'a>(c: &'a ParserConfig) -> impl FnMut(&'a str) -> IResult<&'a str, ()> {
    |i| {
        //let (i, _) = whitespaced(pair(tag(c.cdelim.as_str()), multispace0))(i)?;
        let (i, _) = pair(whitespaced(tag(c.cdelim.as_str())), opt(char('\n')))(i)?;
        Ok((i, ()))
    }
}

// Wraps another parser to allow for whitespaces
fn whitespaced<'a, O1, E, P>(p: P) -> impl FnMut(&'a str) -> IResult<&'a str, O1, E>
where
    P: Parser<&'a str, O1, E>,
    E: ParseError<&'a str>,
{
    delimited(space0, p, space0)
}

/*
 * < transform >
 * lua
 * < to >
 * text
 * < end >
 */
fn transform_block<'a>(
    c: &'a ParserConfig,
) -> impl FnMut(&'a str) -> IResult<&'a str, TemplateBlock> {
    |i| {
        let (i, _) = named_tag(c, c.transform.as_str())(i)?;
        let (i, transform) = is_not(c.odelim.as_str())(i)?;

        let (i, _) = named_tag(c, c.to.as_str())(i)?;
        let (i, blocks) = many0(template_block(c))(i)?;
        let (i, _) = named_tag(c, c.end.as_str())(i)?;

        Ok((
            i,
            Rc::new(directive::Transform {
                transform: trim_keep_newline(transform),
                blocks,
            }),
        ))
    }
}

/* like &str::trim_end but not removing \n's */
fn trim_keep_newline(s: &str) -> String {
    // trims the end of a string of spaces and tabs
    s.chars()
        .rev()
        .skip_while(|c| *c == ' ' || *c == '\t')
        .collect::<String>()
        .chars()
        .rev()
        .collect()
}

/* < if condition >
 *   template_block
 *   template_block
 *   ...
 * < end >
 */
#[rustfmt::skip]
fn if_block<'a>(c: &'a ParserConfig) -> impl FnMut(&'a str) -> IResult<&'a str, TemplateBlock> {
    |i| {
        let (i, (condition, blocks, _)) = tuple((
            if_line(c), 
            many0(template_block(c)), 
            named_tag(c, c.end.as_str()),
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
    |i| {
        let (i, (_, condition)) = delimited(
            odelim(c),
            pair(tag(c.if_.as_str()), is_not(c.cdelim.as_str())),
            cdelim(c),
        )(i)?;
        Ok((i, condition.trim()))
    }
}

// < "tag_msg" >
fn named_tag<'a>(
    c: &'a ParserConfig,
    tag_msg: &'a str,
) -> impl FnMut(&'a str) -> IResult<&'a str, ()> {
    move |i| {
        let (i, _) = delimited(odelim(c), tag(tag_msg), cdelim(c))(i)?;
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
    |i| {
        // (&str, (&str, Vec<Rc<dyn templating::directive::Generator>>, &str))
        let (i, condition) = if_line(c)(i)?;
        let (i, if_blocks) = many0(template_block(c))(i)?;
        let (i, _) = named_tag(c, c.else_.as_str())(i)?;
        let (i, else_blocks) = many0(template_block(c))(i)?;
        let (i, _) = named_tag(c, c.end.as_str())(i)?;

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
                transform: "transform".to_string(),
                to: "to".to_string(),
            }
        };
    }

    // Use Debug to compare the output for the purpose of testing. (since Eq / ParialEq are not
    // object-safe)
    fn compare_vec_templateblocks(t1: Vec<TemplateBlock>, t2: Vec<TemplateBlock>) {
        assert_eq!(t1.len(), t2.len());
        for (i, j) in t1.iter().zip(t2.iter()) {
            compare_templateblocks(i, j);
        }
    }

    fn compare_templateblocks(t1: &TemplateBlock, t2: &TemplateBlock) {
        assert_eq!(format!("{:?}", t1), format!("{:?}", t2))
    }

    #[test]
    fn test_parse_template_str() {
        let template = indoc!(
            // TODO: Text at the beginning
            r#"
            !% include ./test.html %!

            !% if true %!
                Text inside an If
            !% end %!


            Some Text In between


            !% if true %!
                !% include ./test.html %!

                !% transform %!
                    lua
                !% to %!
                    text
                !% end %!

                text ouside transform
            !% else %!
                !% include ./test.html %!

                Some Text Inside
            !% end %!


            Some Text Outside

            "#
        );

        let expected: Vec<TemplateBlock> = vec![
            Rc::new(directive::Include {
                path: "./test.html".to_string(),
            }),
            Rc::new("\n"),
            Rc::new(directive::If {
                condition: "true".to_string(),
                blocks: vec![Rc::new("    Text inside an If\n")],
            }),
            Rc::new("\n\nSome Text In between\n\n\n"),
            Rc::new(directive::IfElse {
                condition: "true".to_string(),
                if_blocks: vec![
                    Rc::new(directive::Include {
                        path: "./test.html".to_string(),
                    }),
                    Rc::new("\n"),
                    Rc::new(directive::Transform {
                        transform: "        lua\n".to_string(),
                        blocks: vec![Rc::new("        text\n")],
                    }),
                    Rc::new("\n    text ouside transform\n"),
                ],
                else_blocks: vec![
                    Rc::new(directive::Include {
                        path: "./test.html".to_string(),
                    }),
                    Rc::new("\n    Some Text Inside\n"),
                ],
            }),
            Rc::new("\n\nSome Text Outside\n\n"),
        ];

        let result = parse_template_str(&PARSER_CONFIG, template).unwrap().1;
        dbg!(&result);
        compare_vec_templateblocks(result, expected);
    }

    #[test]
    fn test_odelim() {
        let input = "   !%";
        let expected = Ok(("", ()));
        let result = odelim(&PARSER_CONFIG)(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_cdelim() {
        let input = "%!   \n ";
        let expected = Ok((" ", ()));
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
            blocks: vec![Rc::new("    text\n    text\n")],
        };

        let result = if_block(&PARSER_CONFIG)(input).unwrap().1;
        dbg!(&result);
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
    fn test_named_tag() {
        let input = "!% name %!";
        let result = named_tag(&PARSER_CONFIG, "name")(input);
        let expected = Ok(("", ()));
        assert_eq!(result, expected);
        // Tag doesnt return, but we can test the unwrap
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

    #[test]
    fn test_tranform_block() {
        let input = indoc!(
            r#"
                !% transform %!
                    luacode
                    luacode
                !% to %!
                    text
                    text
                !% end %!
            "#
        );

        let expected = directive::Transform {
            transform: "    luacode\n    luacode\n".to_string(),
            blocks: vec![Rc::new("    text\n    text\n")],
        };

        let result = transform_block(&PARSER_CONFIG)(input).unwrap().1;
        assert_eq!(format!("{:?}", result), format!("{:?}", expected));
    }
}
