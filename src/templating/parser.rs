// https://github.com/fflorent/nom_locate/ Line numbers?

use super::{
    directive::{BlockDirective, DoNothing},
    template::{Generator, Template, TemplateDirectiveBlock},
};
use anyhow::Result;
use std::{path::PathBuf, rc::Rc};

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
fn template(input: &str) -> IResult<&str, Vec<Rc<dyn Generator>>> {
    many0(alt((directive_block, text)))(input)
}
//
fn text(input: &str) -> IResult<&str, Rc<dyn Generator>> {
    map(is_not(OPENING_MARK), |t: &str| {
        let boxed_text: Rc<dyn Generator> = Rc::new(t.trim().to_string());
        boxed_text
    })(input)
}

/*
 * text
 * directive_block
 */
fn template_block(input: &str) -> IResult<&str, Rc<dyn Generator>> {
    alt((
        directive_block,
        map(is_not(CLOSING_MARK), |t: &str| {
            let boxed_text: Rc<dyn Generator> = Rc::new(t.trim().to_string());
            boxed_text
        }),
    ))(input)
}

/*
 * ( directive template_blocks )
 */
fn directive_block(input: &str) -> IResult<&str, Rc<dyn Generator>> {
    let (rest, (directive, blocks)) = delimited(
        tag(OPENING_MARK),
        pair(block_directive, many0(template_block)),
        tag(CLOSING_MARK),
    )(input)?;

    // OOooh?
    Ok((rest, Rc::new(TemplateDirectiveBlock { directive, blocks })))
}

fn block_directive(input: &str) -> IResult<&str, Rc<dyn BlockDirective>> {
    let (rest, parsed) = terminated(map(is_not("\n"), |t: &str| t.trim()), char('\n'))(input)?;

    let directive = Rc::new(DoNothing {
        text: parsed.to_string(),
    });

    Ok((rest, directive))
}

//#[cfg(test)]
//mod tests {
//    use std::fmt::format;
//
//    use super::*;
//
//    fn compare_vec_template_blocks(v1: &Vec<TemplateBlock>, v2: &Vec<TemplateBlock>) -> bool {
//        assert_eq!(v1.len(), v2.len());
//        for (b1, b2) in v1.iter().zip(v2.iter()) {
//            if compare_template_blocks(b1, b2) == false {
//                return false;
//            }
//        }
//        true
//    }
//
//    // Because TemplateBlock doesnt implement Eq / PartialEq (because of Rc), we use
//    // std::fmt::Debug for the purposes of testing
//    fn compare_template_blocks(t1: &TemplateBlock, t2: &TemplateBlock) -> bool {
//        match (t1, t2) {
//            (TemplateBlock::Text(t1), TemplateBlock::Text(t2)) => t1 == t2,
//            (TemplateBlock::BlockDirective(t1), TemplateBlock::BlockDirective(t2)) => {
//                format!("{:?}", t1.directive).cmp(&format!("{:?}", t2.directive))
//                    == std::cmp::Ordering::Equal
//                    && t1
//                        .blocks
//                        .iter()
//                        .zip(t2.blocks.iter())
//                        .all(|(t1, t2)| compare_template_blocks(t1, t2))
//            }
//            (TemplateBlock::LineDirective(t1), TemplateBlock::LineDirective(t2)) => {
//                format!("{:?}", t1.directive).cmp(&format!("{:?}", t2.directive))
//                    == std::cmp::Ordering::Equal
//            }
//            _ => false,
//        }
//    }
//
//    #[test]
//    fn test_template() {
//        let input = format!(
//            r#"
// textbefore
// {} directive1
//   text1
// {}
// textbetween
// {} directive2
//   text2
// {}
// textafter
// "#,
//            OPENING_MARK, CLOSING_MARK, OPENING_MARK, CLOSING_MARK
//        );
//        let expected = vec![
//            TemplateBlock::Text("textbefore".to_string()),
//            TemplateBlock::BlockDirective(TemplateDirectiveBlock {
//                directive: Rc::new(DoNothing {
//                    text: "directive1".to_string(),
//                }),
//                blocks: vec![TemplateBlock::Text("text1".to_string())],
//            }),
//            TemplateBlock::Text("textbetween".to_string()),
//            TemplateBlock::BlockDirective(TemplateDirectiveBlock {
//                directive: Rc::new(DoNothing {
//                    text: "directive2".to_string(),
//                }),
//                blocks: vec![TemplateBlock::Text("text2".to_string())],
//            }),
//            TemplateBlock::Text("textafter".to_string()),
//        ];
//
//        let result = template(input.as_str()).unwrap().1;
//        assert!(compare_vec_template_blocks(&result, &expected));
//    }
//
//    #[test]
//    fn test_template_block() {
//        let input = format!(
//            "{} directive1 \n{} directive2 \ntext{} text2{}",
//            OPENING_MARK, OPENING_MARK, CLOSING_MARK, CLOSING_MARK
//        );
//        let wrong_input1 = format!(
//            "{} directive1 \n{} directive2 \ntext{} text2{}",
//            OPENING_MARK, "WRONG_OPENING_MARK", CLOSING_MARK, CLOSING_MARK
//        );
//        let wrong_input2 = format!(
//            "{} directive1 \n{} NOT_DIRECTIVE2 \ntext{} text2{}",
//            OPENING_MARK, OPENING_MARK, CLOSING_MARK, CLOSING_MARK
//        );
//        let expected = TemplateBlock::BlockDirective(TemplateDirectiveBlock {
//            directive: Rc::new(DoNothing {
//                text: "directive1".to_string(),
//            }),
//            blocks: vec![
//                TemplateBlock::BlockDirective(TemplateDirectiveBlock {
//                    directive: Rc::new(DoNothing {
//                        text: "directive2".to_string(),
//                    }),
//                    blocks: vec![TemplateBlock::Text("text".to_string())],
//                }),
//                TemplateBlock::Text("text2".to_string()),
//            ],
//        });
//
//        let result = template_block(input.as_str()).unwrap().1;
//        let wrong_result1 = template_block(wrong_input1.as_str()).unwrap().1;
//        let wrong_result2 = template_block(wrong_input2.as_str()).unwrap().1;
//        assert!(false == compare_template_blocks(&wrong_result1, &expected));
//        assert!(false == compare_template_blocks(&wrong_result2, &expected));
//        assert!(compare_template_blocks(&result, &expected));
//    }
//}
