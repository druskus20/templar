/*
 * This should have a builder pattern that can be reused for multiple templates.
 *
 * For now: First run directives, then replace values
 * Later optimizations: try to make as little passes as possible, by saving positions on the text
 * and them applying everything in one pass.
 */

#[derive(Debug, Clone)]
struct TemplateProcessor {
    // placeholders: todo!(), // Positions of the placeholders
// directives: todo!(),   // Positions of the directives
// values: todo!(), // Placeholders - Values to replace
}

impl TemplateProcessor {
    pub fn new() -> TemplateProcessor {
        todo!()
    }
}
