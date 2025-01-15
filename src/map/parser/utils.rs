use nom::{branch::alt, bytes::complete::{is_not, tag, take, take_until, take_while}, character::complete::{alphanumeric1, char, digit1}, combinator::{cut, eof, fail, map_res, value}, error::{context, ContextError, ParseError}, multi::separated_list0, sequence::{delimited, pair, preceded, terminated, tuple}, IResult, Parser};

pub fn name_and_expression<'a, E: ParseError<&'a str> + ContextError<&'a str> + nom::error::FromExternalError<&'a str, std::num::ParseIntError>>(
    i: &'a str,
) -> IResult<&'a str, (&'a str, &'a str), E> {
    context("Expression with '=' (e.g. \"name = expr, expr, ...\" )", tuple((
        delimited(space, is_not("="), space),
        preceded(char('='), take_all))))(i)
}

pub fn name_and_value<'a, E: ParseError<&'a str> + ContextError<&'a str> + nom::error::FromExternalError<&'a str, std::num::ParseIntError>>(
    i: &'a str,
) -> IResult<&'a str, (&'a str, &'a str), E> {
    context("Expression with ':' (e.g. \"name: value\" )", tuple((
        delimited(space, is_not(":,"), space),
        preceded(char(':'), take_all))))(i)
}

pub fn blueprint_dimensions<'a, E: ParseError<&'a str> + ContextError<&'a str> + nom::error::FromExternalError<&'a str, std::num::ParseIntError>>(
    i: &'a str,
) -> IResult<&'a str, (u32, u32), E> {
    context("Blueprint dimensions (width x height)", tuple((
        delimited(space, map_res(digit1, str::parse::<u32>), space),
        delimited(pair(char('x'), space), 
        map_res(digit1, str::parse::<u32>), 
        space)
    )))(i)
}

/// Separates expressions by the `;` separator.
pub fn separate_expressions<'a, E: ParseError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, Vec<&'a str>, E> {
    let (i, expressions) = separated_list0(tag(";"), take_while(|c| c != ';'))(i)?;
    let separated = expressions
        .iter()
        .map(|expr| expr.trim())
        .filter(|expr| !expr.is_empty())
        .collect();
    Ok((i, separated))
}

pub fn separate_objects<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, Vec<&'a str>, E> {
    context(
        "separate objects",
        separated_list0(tag(","), take_while(|c| c != ',')),
    )(i)
}

pub fn comma_separator<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, char, E> {
    context(
        "comma separator",
        delimited(space, char(','), space))(i)
}

/// Discards all following spaces (' ', '\t', '\r', '\n') or none if there aren't any.
pub fn space<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, &'a str, E> {
    let chars = " \t\r\n";
    take_while(|c| chars.contains(c))(i)
}

pub fn string<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, &'a str, E> {
    context(
        "string",
        preceded(char('\"'), cut(terminated(take_until("\""), char('\"')))),
    )(i)
}

pub fn take_all<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, &'a str, E> {
    take_while(|_| true)(i)
}

pub fn path<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, &'a str, E> {
    context("path", delimited(char('"'), is_not("\""), char('"')))(i)
}

pub fn vox_model_name<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, &'a str, E> {
    context("vox model name", preceded(space, alphanumeric1))(i)
}


pub fn expression_key<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, &'a str, E> {
    context("expression key", preceded(space, take(1usize)))(i)
}

pub fn boolean<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, bool, E> {
    let parse_true = value(true, tag("true"));
    let parse_false = value(false, tag("false"));
    alt((parse_true, parse_false))(i)
}

pub fn is_empty_or_fail<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
    err_msg: &'static str,
) -> IResult<&'a str, &'a str, E> {
    if !i.is_empty() {
        context(err_msg, fail)(i)
    } else {
        Ok(("", ""))
    }
}

/// Removes comments and empty lines from input.
pub fn clean_input(input: String) -> String {
    input
        .lines()
        .map(|line| {
            let mut line = line.split("//").next().unwrap().trim().to_owned();
            line.push('\n');
            line
        })
        .collect()
}