use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "grammar/axis_core.pest"]
pub struct AxisCoreParser;

pub fn parse_program(input: &str) -> Result<(), pest::error::Error<Rule>> {
    AxisCoreParser::parse(Rule::program, input)?;
    Ok(())
}
