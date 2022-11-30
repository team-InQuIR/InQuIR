use crate::hir::ast::{
    Expr,
    InitExpr,
    ApplyExpr,
    MeasureExpr,
    BarrierExpr,
    MeasureKind,
    PrimitiveGate
};

use anyhow::Result;
use nom::{
    IResult,
    branch::alt,
    bytes::complete::{
        tag,
        take_while,
        take_until,
    },
    character::complete::{
        char,
        alpha1,
        alphanumeric0,
    },
    combinator::{opt, map_res},
    error::VerboseError,
    multi::{many1, separated_list0, separated_list1},
    sequence::{
        tuple,
        delimited,
        preceded,
        terminated,
    },
    number::complete::double,
};

#[derive(Debug)]
pub enum Error<'a> {
    Unexpected(&'a str),
    Nom(nom::Err<VerboseError<&'a str>>),
}

impl<'a> From<nom::Err<VerboseError<&'a str>>> for Error<'a> {
    fn from(err: nom::Err<VerboseError<&'a str>>) -> Self {
        Error::Nom(err)
    }
}

pub fn parse(input: &str) -> Result<Vec<Expr>, Error> {
    let (input, _) = parse_header(input)?;
    let (input, _) = spaces_and_endlines(input)?;
    let (input, exps) = many1(terminated(
        parse_stmt,
        tuple((tag(";"), spaces_and_endlines))))(input)?;
    let (rest, _) = spaces_and_endlines(input)?;

    if rest == "" {
        Ok(exps.concat())
    } else {
        Err(Error::Unexpected(rest))
    }
}

pub fn parse_header(input: &str) -> IResult<&str, (), VerboseError<&str>> {
    let (input, _) = tag("OPENQASM 2.0;")(input)?;
    let (input, _) = spaces_and_endlines(input)?;
    let (input, _) = opt(parse_include)(input)?;
    let (input, _) = spaces_and_endlines(input)?;
    Ok((input, ()))
}

fn parse_include(input: &str) -> IResult<&str, (), VerboseError<&str>> {
    let (input, _) = tag("include")(input)?;
    let (input, _) = spaces_and_endlines(input)?;
    let (input, _) = preceded(take_until(";"), tag(";"))(input)?; // TODO?
    let (input, _) = spaces_and_endlines(input)?;
    Ok((input, ()))
}

fn parse_stmt(input: &str) -> IResult<&str, Vec<Expr>, VerboseError<&str>> {
    let (input, e) = alt((
        map_res(parse_reg_decl, |inits| -> Result<_> {
            Ok(inits.into_iter().map(|e| Expr::from(e)).collect())
        }),
        map_res(parse_apply, |e| -> Result<_> { Ok(vec![Expr::from(e)]) }),
        map_res(parse_measure, |e| -> Result<_> { Ok(vec![Expr::from(e)]) }),
        map_res(parse_barrier, |e| -> Result<_> { Ok(vec![e]) })
    ))(input)?;

    Ok((input, e))
}

pub fn parse_reg_decl(input: &str) -> IResult<&str, Vec<InitExpr>, VerboseError<&str>> {
    let (input, reg_kind) = alt((tag("qreg"), tag("creg")))(input)?;
    let (input, (var, size)) = preceded(spaces_and_endlines, parse_indexed_array)(input)?;
    if reg_kind == "creg" {
        Ok((input, Vec::new()))
    } else {
        let size = size.parse::<u32>().unwrap();
        let initializes: Vec<_> = (0..size).map(|i| InitExpr { dst: var.clone() + &i.to_string() }).collect();
        Ok((input, initializes))
    }
}

pub fn parse_apply(input: &str) -> IResult<&str, ApplyExpr, VerboseError<&str>> {
    let (input, gate) = parse_gate(input)?;
    let (input, _) = spaces_and_endlines(input)?;
    let (input, args) = parse_arguments(input)?;
    Ok((input, ApplyExpr { gate, args }))
}

pub fn parse_gate(input: &str) -> IResult<&str, PrimitiveGate, VerboseError<&str>> {
    alt((parse_gate_cx, parse_gate_rz, parse_gate_u3, parse_other_gates))(input)
}

pub fn parse_gate_cx(input: &str) -> IResult<&str, PrimitiveGate, VerboseError<&str>> {
    let (input, _) = tag("cx")(input)?;
    Ok((input, PrimitiveGate::CX))
}

pub fn parse_gate_rz(input: &str) -> IResult<&str, PrimitiveGate, VerboseError<&str>> {
    let (input, _) = alt((tag("rz"), tag("u1")))(input)?;
    let (input, theta) = delimited(tag("("), double, tag(")"))(input)?;
    Ok((input, PrimitiveGate::Rz(theta)))
}

pub fn parse_gate_u3(input: &str) -> IResult<&str, PrimitiveGate, VerboseError<&str>> {
    let (input, params) = preceded(char('u'),
                              delimited(char('('),
                                        separated_list0(char(','), take_while(move |c: char| !",)".contains(c))), // TODO
                                        char(')')))(input)?;
    assert!(params.len() == 3);
    let gate = match (params[0], params[1], params[2]) {
        ("pi/2", "0", "pi") => PrimitiveGate::H,
        ("pi", "0", "pi") => PrimitiveGate::X,
        ("0", "0", "pi/4") => PrimitiveGate::T,
        ("0", "0", "-pi/4") => PrimitiveGate::Tdg,
        _ => unimplemented!() // TODO
    };
    Ok((input, gate))
}

fn parse_other_gates(input: &str) -> IResult<&str, PrimitiveGate, VerboseError<&str>> {
    let (input, name) = alt((
        tag("x"),
        tag("z"),
        tag("h"),
        tag("tdg"), // Be careful to the order of 'tdg' and 't'.
        tag("t"),
    ))(input)?;
    let gate = match name {
        "x" => PrimitiveGate::X,
        "z" => PrimitiveGate::Z,
        "h" => PrimitiveGate::H,
        "t" => PrimitiveGate::T,
        "tdg" => PrimitiveGate::Tdg,
        _ => unimplemented!()
    };
    Ok((input, gate))
}

pub fn parse_measure(input: &str) -> IResult<&str, MeasureExpr, VerboseError<&str>> {
    let (input, _) = tag("measure")(input)?;
    let (input, _) = spaces_and_endlines(input)?;
    let (input, args) = parse_arguments(input)?;
    let (input, _) = delimited(spaces_and_endlines, tag("->"), spaces_and_endlines)(input)?;
    let (input, dst) = parse_argument(input)?;
    Ok((input, MeasureExpr { kind: MeasureKind::Z, dst, args }))
}

fn parse_barrier(input: &str) -> IResult<&str, Expr, VerboseError<&str>> {
    let (input, _) = tag("barrier")(input)?;
    let (input, _) = spaces_and_endlines(input)?;
    let (input, args) = parse_arguments(input)?;

    Ok((input, Expr::Barrier(BarrierExpr { args })))
}

pub fn parse_argument(input: &str) -> IResult<&str, String, VerboseError<&str>> {
    let (input, arg) = alt((
        // Remark: in this order
        map_res(parse_indexed_array, |(var, idx)| -> Result<String> { Ok(var + &idx) }),
        parse_variable
    ))(input)?;
    Ok((input, arg))
}
pub fn parse_arguments(input: &str) -> IResult<&str, Vec<String>, VerboseError<&str>> {
    let (input, args) = separated_list1(char(','), parse_argument)(input)?;
    Ok((input, args))
}

pub fn parse_variable(input: &str) -> IResult<&str, String, VerboseError<&str>> {
    let (input, (pre, tail)) = tuple((alpha1, alphanumeric0))(input)?;
    Ok((input, String::from(pre) + tail))
}

pub fn parse_indexed_array(input: &str) -> IResult<&str, (String, String), VerboseError<&str>> {
    let (input, var) = parse_variable(input)?;
    let (input, idx) = delimited(char('['), alphanumeric0, char(']'))(input)?;
    Ok((input, (var, idx.to_string())))
}


pub fn spaces_and_endlines(input: &str) -> IResult<&str, (), VerboseError<&str>> {
    let skipped = " \t\r\n";
    let (input, _) = take_while(move |c: char| skipped.contains(c))(input)?;
    Ok((input, ()))
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::hir::PrimitiveGate;

    #[test]
    fn parse_indexed_array_test() {
        let input = "arr[0]";
        let (_, (var, idx)) = parse_indexed_array(input).unwrap();
        assert_eq!(var, "arr");
        assert_eq!(idx, "0");
    }

    #[test]
    fn parse_qreg_decl_test() {
        let input = "qreg q[2]";
        let (_, decls) = parse_reg_decl(input).unwrap();
        assert_eq!(decls.len(), 2);
        assert_eq!(decls[0].dst, "q0");
        assert_eq!(decls[1].dst, "q1");
    }

    macro_rules! check_gate {
        ($input:expr, $expected:expr) => {
            let (_, gate) = parse_gate($input).unwrap();
            assert_eq!(gate, $expected);
        }
    }

    #[test]
    fn parse_gates() {
        check_gate!("u(pi/2,0,pi)", PrimitiveGate::H);
        check_gate!("u(0,0,pi/4)", PrimitiveGate::T);
        check_gate!("u(0,0,-pi/4)", PrimitiveGate::Tdg);
        check_gate!("u(pi,0,pi)", PrimitiveGate::X);
        check_gate!("cx", PrimitiveGate::CX);
        check_gate!("tdg", PrimitiveGate::Tdg);
    }

    #[test]
    fn parse_rz_test() {
        let input = "rz(-0.15)";
        let (_, gate) = parse_gate(input).unwrap();
        match gate {
            PrimitiveGate::Rz(theta) => {
                assert!((theta - (-0.15)).abs() < 1e9);
            },
            _ => assert!(false)
        }
    }

    #[test]
    fn parse_measure_test() {
        let input = "measure q[0] -> c[0]";
        let (_, e) = parse_measure(input).unwrap();
        assert_eq!(e.args.len(), 1);
        assert_eq!(e.dst, "c0");
        assert_eq!(e.args[0], "q0");
    }

    #[test]
    fn parse_header_test() {
        let input = "OPENQASM 2.0;\ninclude \"qelib1.inc\";\n";
        let (input, _) = parse_header(input).unwrap();
        assert_eq!(input, "");
    }
}
