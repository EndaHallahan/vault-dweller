use ariadne::{Color, Label, Report, ReportKind, Source};
use chumsky::prelude::*;
use std::{collections::HashMap, env, fs};

use crate::{ VaultIndex };

#[derive(Debug)]
enum DataSource {
    Tag(String),
    Folder(String),
    File(String),
    InLink(String),
    OutLink(String),
}
impl DataSource {
    pub fn get_matches<'a>(&self, index: &'a VaultIndex) -> Option<Vec<String>> {
        if let Some(v) = match self {
            DataSource::Tag(tag_name) => index.tags.get(tag_name),
            _ => todo!("Other sources aren't implemented yet!"),
        } {
            return Some(v.clone());
        } else {
            return None;
        }
    }
}

fn eval_or(x: Option<Vec<String>>, y: Option<Vec<String>>) -> Option<Vec<String>> {
    let mut out_vec: Vec<String> = vec![];
    if let Some(x_list) = x {
        out_vec.extend(x_list.clone());
    }
    if let Some(y_list) = y {
        out_vec.extend(y_list.clone());
    }

    out_vec.sort();
    out_vec.dedup();

    if out_vec.is_empty() {
        return None;
    } else {
        return Some(out_vec);
    }
}

fn eval_and(x: Option<Vec<String>>, y: Option<Vec<String>>) -> Option<Vec<String>> {
    let mut out_vec: Vec<String> = vec![];

    if let Some(x_list) = x {
        if let Some(y_list) = y {
            for item in x_list {
                if y_list.contains(&item) {
                    out_vec.push(item.clone());
                }
            }
        }
    }

    if out_vec.is_empty() {
        return None;
    } else {
        return Some(out_vec);
    }
}



#[derive(Debug)]
enum Expr {
	Invalid, 
	Source(DataSource),
	From(Box<Expr>),
	List {
        from: Box<Expr>
    },
    Or(Box<Expr>, Box<Expr>),
    And(Box<Expr>, Box<Expr>),
}

fn parser() -> impl Parser<char, Expr, Error = Simple<char>> {

	

    let expr = recursive(|expr| {
        let tag_path = filter(|c: &char| !c.is_ascii_whitespace()).repeated();

        let tag = just('#')
            .ignore_then(tag_path)
            .map(|c: Vec<char>| Expr::Source(DataSource::Tag(c.into_iter().collect())))
            .padded();

        let op = tag.clone()
            .then(
                text::keyword("and").to(Expr::And as fn(_, _) -> _)
                .or(text::keyword("or").to(Expr::Or as fn(_, _) -> _))
                .then(tag)
                .repeated()
            ).foldl(|lhs, (op, rhs)| op(Box::new(lhs), Box::new(rhs)));


        

        op
    });

    let from = text::keyword("FROM")
            .ignore_then(expr)
            .map(|tag| Expr::From(Box::new(tag)))
            .padded();


    let decl = recursive(|decl| {
        let r#list = text::keyword("LIST")
            .ignore_then(from)
            .map(|from| Expr::List {
                from: Box::new(from)
            });

        r#list
            // Must be later in the chain than `r#let` to avoid ambiguity
            .padded()
    });

    decl
}

fn eval<'a>(expr: &'a Expr, index: &'a VaultIndex) -> Result<Option<Vec<String>>, String> {
    match expr {
        Expr::List {from} => Ok(eval(from, &index)?),
        Expr::From(tag) => Ok(eval(tag, &index)?),
        Expr::Source(source) => Ok(source.get_matches(&index)),
        Expr::Or(x, y) => Ok(eval_or(eval(x, &index)?, eval(y, &index)?)),
        Expr::And(x, y) => Ok(eval_and(eval(x, &index)?, eval(y, &index)?)),
        
        _ => todo!("Stuff here!"),
    }
}

pub fn to_view(in_query: &str, index: &VaultIndex ) {
	match parser().parse(in_query) {
        Ok(ast) => match eval(&ast, &index) {
            Ok(output) => println!("Matched Notes: {:?}", output),
            Err(eval_err) => println!("Evaluation error: {}", eval_err),
        },
        Err(parse_errs) => parse_errs
            .into_iter()
            .for_each(|e| println!("Parse error: {}", e)),
    }
}