// https://github.com/fflorent/nom_locate/ Line numbers?

/*
 * A template parser that allows for runtime configuration using ParserConfig
 */

/*
 * TODO Nesting an If inside a transform (lua) block fails
 * there's still a lot of work to do with edge cases
 */

use super::directives;
use super::directives::Directive;
use super::directives::DynDirective;

use nom::character::complete::{alphanumeric1, space0, space1};
use nom::combinator::opt;
use nom::error::ParseError;
use nom::sequence::tuple;
use std::rc::Rc;

use nom::{
    branch::alt,
    bytes::complete::{is_not, tag},
    character::complete::char,
    combinator::map,
    multi::many0,
    sequence::{delimited, pair},
    IResult,
};

#[derive(Debug, Clone)]
pub(crate) struct ParserConfig {
    pub odelim: String,
    pub cdelim: String,
    pub comment: String,
    pub if_: String,
    pub else_: String,
    pub end: String,
    pub include: String,
    pub transform: String,
    pub to: String,
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

pub(super) struct Parser {
    pub config: ParserConfig,
}

impl Parser {
    // We tried inventing trait type aliases
    // Parser<I, O> === FnMut(I) -> IResult<I, O>
    // trait Parser<I, O>: FnMut(I) -> IResult<I, O> {}
    // impl<T, I, O> Parser<I, O> for T where T: FnMut(I) -> IResult<I, O> {}
    // Unfortunately, dyn Generator is not infered correctly, so we can't use it

    pub(super) fn parse_template_str<'a>(&self, i: &'a str) -> anyhow::Result<Vec<DynDirective>> {
        let r = many0(alt((
            template_block(&self.config),
            // Text
            map(is_not(self.config.odelim.as_str()), |t: &str| {
                let boxed_text: DynDirective = Rc::new(t.trim().to_string());
                boxed_text
            }),
        )))(i);

        // Litefimes
        let template = match r {
            Ok((_, blocks)) => Ok(blocks),
            Err(e) => Err(anyhow::anyhow!("{}", e)),
        }?;

        anyhow::Ok(template)
    }
}

// TODO: this is hacky
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

/* Either text or some directive */
fn template_block<'a>(
    c: &'a ParserConfig,
) -> impl FnMut(&'a str) -> IResult<&'a str, DynDirective> {
    alt((
        include_block(c),
        if_block(c),
        ifelse_block(c),
        transform_block(c),
        // NOTE: cdelim? odelim?
        // Text
        map(is_not(c.odelim.as_str()), |t: &str| {
            let boxed_text: DynDirective = Rc::new(trim_keep_newline(t));
            boxed_text
        }),
    ))
}

/*
 * < include str >
 */
fn include_block<'a>(c: &'a ParserConfig) -> impl FnMut(&'a str) -> IResult<&'a str, DynDirective> {
    |i: &'a str| {
        let (i, (_, path)) = delimited(
            odelim(c),
            pair(tag(c.include.as_str()), is_not(c.cdelim.as_str())),
            cdelim(c),
        )(i)?;

        let include_block: Rc<dyn Directive> = Rc::new(directives::Include {
            path: path.trim().to_string(),
            parser_config: c.clone(),
        });

        Ok((i, include_block))
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
        let (i, _) = pair(whitespaced(tag(c.cdelim.as_str())), opt(char('\n')))(i)?;
        Ok((i, ()))
    }
}

// Wraps another parser to allow for whitespaces
fn whitespaced<'a, O1, E, P>(p: P) -> impl FnMut(&'a str) -> IResult<&'a str, O1, E>
where
    P: nom::Parser<&'a str, O1, E>,
    E: ParseError<&'a str>,
{
    delimited(space0, p, space0)
}

/*
 * < transform input_name >
 * lua
 * < to >
 * text
 * < end >
 */
fn transform_block<'a>(
    c: &'a ParserConfig,
) -> impl FnMut(&'a str) -> IResult<&'a str, DynDirective> {
    |i| {
        let (i, input_name) = transform_line(c)(i)?;
        let (i, transform) = is_not(c.odelim.as_str())(i)?;
        let (i, _) = named_tag(c, c.to.as_str())(i)?;
        let (i, blocks) = many0(template_block(c))(i)?;
        let (i, _) = named_tag(c, c.end.as_str())(i)?;

        Ok((
            i,
            Rc::new(directives::Transform {
                transform: trim_keep_newline(transform),
                blocks,
                input_name: input_name.to_string(),
            }),
        ))
    }
}

/* < transform input_name > */
fn transform_line<'a>(c: &'a ParserConfig) -> impl FnMut(&'a str) -> IResult<&'a str, &'a str> {
    |i| {
        let (i, (_, _, input_name)) = delimited(
            odelim(c),
            tuple((tag(c.transform.as_str()), space1, alphanumeric1)),
            cdelim(c),
        )(i)?;

        Ok((i, input_name))
    }
}

/* < if condition >
 *   template_block
 *   template_block
 *   ...
 * < end >
 */
fn if_block<'a>(c: &'a ParserConfig) -> impl FnMut(&'a str) -> IResult<&'a str, DynDirective> {
    |i| {
        let (i, (condition, blocks, _)) = tuple((
            if_line(c),
            many0(template_block(c)),
            named_tag(c, c.end.as_str()),
        ))(i)?;

        Ok((
            i,
            Rc::new(directives::If {
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

// < tag_name >
fn named_tag<'a>(
    c: &'a ParserConfig,
    tag_name: &'a str,
) -> impl FnMut(&'a str) -> IResult<&'a str, ()> {
    move |i| {
        let (i, _) = delimited(odelim(c), tag(tag_name), cdelim(c))(i)?;
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
fn ifelse_block<'a>(c: &'a ParserConfig) -> impl FnMut(&'a str) -> IResult<&'a str, DynDirective> {
    |i| {
        // (&str, (&str, Vec<Rc<dyn templating::directive::Generator>>, &str))
        let (i, condition) = if_line(c)(i)?;
        let (i, if_blocks) = many0(template_block(c))(i)?;
        let (i, _) = named_tag(c, c.else_.as_str())(i)?;
        let (i, else_blocks) = many0(template_block(c))(i)?;
        let (i, _) = named_tag(c, c.end.as_str())(i)?;

        Ok((
            i,
            Rc::new(directives::IfElse {
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
    fn compare_vec_templateblocks(t1: Vec<DynDirective>, t2: Vec<DynDirective>) {
        assert_eq!(t1.len(), t2.len());
        for (i, j) in t1.iter().zip(t2.iter()) {
            compare_templateblocks(i, j);
        }
    }

    fn compare_templateblocks(t1: &DynDirective, t2: &DynDirective) {
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

                !% transform i %!
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

        let expected: Vec<DynDirective> = vec![
            Rc::new(directives::Include {
                path: "./test.html".to_string(),
                parser_config: PARSER_CONFIG.clone(),
            }),
            Rc::new("\n"),
            Rc::new(directives::If {
                condition: "true".to_string(),
                blocks: vec![Rc::new("    Text inside an If\n")],
            }),
            Rc::new("\n\nSome Text In between\n\n\n"),
            Rc::new(directives::IfElse {
                condition: "true".to_string(),
                if_blocks: vec![
                    Rc::new(directives::Include {
                        path: "./test.html".to_string(),
                        parser_config: PARSER_CONFIG.clone(),
                    }),
                    Rc::new("\n"),
                    Rc::new(directives::Transform {
                        transform: "        lua\n".to_string(),
                        blocks: vec![Rc::new("        text\n")],
                        input_name: "i".to_string(),
                    }),
                    Rc::new("\n    text ouside transform\n"),
                ],
                else_blocks: vec![
                    Rc::new(directives::Include {
                        path: "./test.html".to_string(),
                        parser_config: PARSER_CONFIG.clone(),
                    }),
                    Rc::new("\n    Some Text Inside\n"),
                ],
            }),
            Rc::new("\n\nSome Text Outside\n\n"),
        ];

        let parser = Parser {
            config: PARSER_CONFIG.clone(),
        };
        let result = parser.parse_template_str(template).unwrap();
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
        let expected = directives::Include {
            path: "path".to_string(),
            parser_config: PARSER_CONFIG.clone(),
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

        let expected = directives::If {
            condition: "condition".to_string(),
            blocks: vec![Rc::new("    text\n    text\n")],
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
    fn test_named_tag() {
        let input = "!% name %!";
        let result = named_tag(&PARSER_CONFIG, "name")(input);
        let expected = Ok(("", ()));
        assert_eq!(result, expected);
        // Tag doesnt return, but we can test the unwrap
    }

    #[test]
    fn test_ifelse_block() {
        let input = indoc!(
            r#"
                !% if condition %!
                text
                !% else %!
                text
                !% end %!
            "#
        );

        let expected = directives::IfElse {
            condition: "condition".to_string(),
            if_blocks: vec![Rc::new("text\n")],
            else_blocks: vec![Rc::new("text\n")],
        };

        let result = ifelse_block(&PARSER_CONFIG)(input).unwrap().1;
        assert_eq!(format!("{:?}", result), format!("{:?}", expected));
    }

    #[test]
    fn test_include_block() {
        let input = "!% include ./some/path %!";
        let expected = directives::Include {
            path: "./some/path".to_string(),
            parser_config: PARSER_CONFIG.clone(),
        };

        let result = include_block(&PARSER_CONFIG)(input).unwrap().1;
        assert_eq!(format!("{:?}", result), format!("{:?}", expected));
    }

    #[test]
    fn test_tranform_block() {
        let input = indoc!(
            r#"
                !% transform input %!
                    luacode
                    luacode
                !% to %!
                    text
                    text
                !% end %!
            "#
        );

        let expected = directives::Transform {
            transform: "    luacode\n    luacode\n".to_string(),
            blocks: vec![Rc::new("    text\n    text\n")],
            input_name: "input".to_string(),
        };

        let result = transform_block(&PARSER_CONFIG)(input).unwrap().1;
        assert_eq!(format!("{:?}", result), format!("{:?}", expected));
    }

    #[test]
    fn test_transform_line() {
        let input = "!% transform input %!";
        let expected = Ok(("", "input"));

        let result = transform_line(&PARSER_CONFIG)(input);
        assert_eq!(result, expected);
    }
}
