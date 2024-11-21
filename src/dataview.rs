use ariadne::{Color, Label, Report, ReportKind, Source};
use chumsky::prelude::*;
use std::{collections::HashMap, env, fs};

use crate::{ VaultIndex };

#[derive(Debug)]
pub enum QueryOutput {
    List(Vec<ListItem>),
    Table(Table),
    Err(Vec<String>),
}

#[derive(Debug)]
pub struct ListItem {
    pub note_name: Option<String>,
    pub additional_info: Option<String>
}

#[derive(Debug)]
pub struct Table {
    pub head: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

enum QueryStructType {
    List,
    Table,
}

struct QueryStruct {
    output_type: QueryStructType,
    matches: Option<Vec<String>>,
    additional_info: Vec<String>,
    as_statements: Vec<Option<String>>,
}
impl QueryStruct {
    pub fn new() -> Self {
        Self {
            output_type: QueryStructType::List,
            matches: None,
            additional_info: vec![],
            as_statements: vec![],
        }
    }
    pub fn build_output(&self) -> QueryOutput {
        match self.output_type {
            QueryStructType::List => {
                let mut out_vec: Vec<ListItem> = vec![];
                if let Some(matches) = &self.matches {
                    for note in matches {
                        out_vec.push(ListItem {
                            note_name: Some(note.to_string()),
                            additional_info: None,
                        });
                    };
                }
                
                
                return QueryOutput::List(out_vec);
            },
            QueryStructType::Table => {
                todo!("Tables aren't implemented yet!");
            },
        };
    }
}

#[derive(Debug)]
enum DataSource {
    Tag(String),
    Folder(String),
    File(String),
    InLink(String),
    OutLink(String),
}
impl DataSource {
    pub fn get_matches(&self, index: &VaultIndex) -> Option<Vec<String>> {
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
        out_vec.extend(x_list);
    }
    if let Some(y_list) = y {
        out_vec.extend(y_list);
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
                    out_vec.push(item);
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
    Negate(Box<Expr>),
}

fn parser() -> impl Parser<char, Expr, Error = Simple<char>> {
    let expr = recursive(|expr| {
        let tag_path = filter(|c: &char| (c.is_alphanumeric() || c == &'/'|| c == &'-'|| c == &'_')).repeated();

        let tag = just('#')
            .ignore_then(tag_path)
            .map(|c: Vec<char>| Expr::Source(DataSource::Tag(c.into_iter().collect())))
            .padded();

        let negate = just('!')
            .ignore_then(tag)
            .map(|tag| Expr::Negate(Box::new(tag)));

        let paren = expr.delimited_by(just('('), just(')'));

        let atom = tag
            .or(paren)
            .padded();

        let ope = atom.clone()
            
            .then(
                text::keyword("AND").to(Expr::And as fn(_, _) -> _)
                .or(text::keyword("and").to(Expr::And as fn(_, _) -> _))
                .or(text::keyword("OR").to(Expr::Or as fn(_, _) -> _))
                .or(text::keyword("or").to(Expr::Or as fn(_, _) -> _))
                .then(atom)
                .repeated()
            ).foldl(|lhs, (op, rhs)| op(Box::new(lhs), Box::new(rhs)));

        ope
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

fn eval<'a>(expr: &'a Expr, index: &'a VaultIndex, query_struct: &'a mut  QueryStruct) -> Result<Option<Vec<String>>, String> {

    match expr {
        Expr::List {from} => {
            query_struct.output_type = QueryStructType::List;
            return Ok(eval(from, &index, query_struct)?);
        },
        Expr::From(tag) => {
            let m = eval(tag, &index, query_struct)?;
            query_struct.matches = m.clone();
            return Ok(m)
        },
        Expr::Source(source) => Ok(source.get_matches(&index)),
        Expr::Or(x, y) => Ok(eval_or(eval(x, &index, query_struct)?, eval(y, &index, query_struct)?)),
        Expr::And(x, y) => Ok(eval_and(eval(x, &index, query_struct)?, eval(y, &index, query_struct)?)),
        
        _ => todo!("Stuff here!"),
    }
}

pub fn to_view(in_query: &str, index: &VaultIndex) -> QueryOutput {
    let mut query_struct = QueryStruct::new();
	match parser().parse_recovery_verbose(in_query) {
        (Some(ast), _err_vec) => match eval(&ast, &index, &mut query_struct) {
            Ok(output) => {
                return query_struct.build_output();
            },
            Err(eval_err) => QueryOutput::Err(vec![format!("{}", eval_err)]),
        },
        (None, err_vec) => {
            let mut out_vec = vec![];
            err_vec
                .into_iter()
                .for_each(|e| out_vec.push(format!("{}", e)));
            return QueryOutput::Err(out_vec);
        },
        
    }
}