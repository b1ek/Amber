use std::fmt::Display;

use heraclitus_compiler::prelude::*;
use itertools::Itertools;
use crate::utils::ParserMetadata;

#[derive(Debug, Clone, Eq, Default)]
pub enum Type {
    #[default] Null,
    Text,
    Bool,
    Num,
    Union(Vec<Box<Type>>),
    Array(Box<Type>),
    Failable(Box<Type>),
    Generic
}

impl Type {
    pub fn is_union(&self) -> bool {
        match self {
            Type::Union(_) => true,
            _ => false
        }
    }

    fn eq_union_normal(one: &Vec<Box<Type>>, other: &Type) -> bool {
        one.iter().find(|x| (***x).to_string() == other.to_string()).is_some()
    }

    fn eq_unions(one: &Vec<Box<Type>>, other: &Vec<Box<Type>>) -> bool {
        one.iter().find(|x| {
            Self::eq_union_normal(other, x)
        }).is_some()
    }
}

impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        if let Type::Union(union) = self {
            if let Type::Union(other) = other {
                return Type::eq_unions(union, other);
            } else {
                return Type::eq_union_normal(union, other);
            }
        }
        
        if let Type::Union(other) = other {
            Type::eq_union_normal(other, self)
        } else {
            self.to_string() == other.to_string()
        }
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Text => write!(f, "Text"),
            Type::Bool => write!(f, "Bool"),
            Type::Num => write!(f, "Num"),
            Type::Null => write!(f, "Null"),
            Type::Union(types) => write!(f, "{}", types.iter().map(|x| format!("{x}")).join(" | ")),
            Type::Array(t) => write!(f, "[{}]", t),
            Type::Failable(t) => write!(f, "{}?", t),
            Type::Generic => write!(f, "Generic")
        }
    }
}

pub trait Typed {
    fn get_type(&self) -> Type;
}

// Tries to parse the type - if it fails, it fails loudly
pub fn parse_type(meta: &mut ParserMetadata) -> Result<Type, Failure> {
    let tok = meta.get_current_token();
    try_parse_type(meta)
        .map_err(|_| Failure::Loud(Message::new_err_at_token(meta, tok).message("Expected a data type")))
}

fn parse_type_tok(meta: &mut ParserMetadata, tok: Option<Token>) -> Result<Type, Failure> {
    match tok.clone() {
        Some(matched_token) => {
            match matched_token.word.as_ref() {
                "Text" => {
                    meta.increment_index();
                    Ok(Type::Text)
                },
                "Bool" => {
                    meta.increment_index();
                    Ok(Type::Bool)
                },
                "Num" => {
                    meta.increment_index();
                    Ok(Type::Num)
                },
                "Null" => {
                    meta.increment_index();
                    Ok(Type::Null)
                },
                "[" => {
                    let index = meta.get_index();
                    meta.increment_index();
                    match try_parse_type(meta) {
                        Ok(Type::Array(_)) => error!(meta, tok, "Arrays cannot be nested due to the Bash limitations"),
                        Ok(result_type) => {
                            token(meta, "]")?;
                            Ok(Type::Array(Box::new(result_type)))
                        },
                        Err(_) => {
                            meta.set_index(index);
                            Err(Failure::Quiet(PositionInfo::at_eof(meta)))
                        }
                    }
                },
                // Error messages to help users of other languages understand the syntax
                text @ ("String" | "Char") => {
                    error!(meta, tok, format!("'{text}' is not a valid data type. Did you mean 'Text'?"))
                },
                number @ ("Number" | "Int" | "Float" | "Double") => {
                    error!(meta, tok, format!("'{number}' is not a valid data type. Did you mean 'Num'?"))
                },
                "Boolean" => {
                    error!(meta, tok, "'Boolean' is not a valid data type. Did you mean 'Bool'?")
                },
                array @ ("List" | "Array") => {
                    error!(meta, tok => {
                        message: format!("'{array}'<T> is not a valid data type. Did you mean '[T]'?"),
                        comment: "Where 'T' is the type of the array elements"
                    })
                },
                // The quiet error
                _ => Err(Failure::Quiet(PositionInfo::at_eof(meta)))
            }
        },
        None => {
            Err(Failure::Quiet(PositionInfo::at_eof(meta)))
        }
    }
}

fn parse_one_type(meta: &mut ParserMetadata, tok: Option<Token>) -> Result<Type, Failure> {
    let res = parse_type_tok(meta, tok)?;
    if token(meta, "?").is_ok() {
        return Ok(Type::Failable(Box::new(res)))
    }
    Ok(res)
}

// Tries to parse the type - if it fails, it fails quietly
pub fn try_parse_type(meta: &mut ParserMetadata) -> Result<Type, Failure> {
    let tok = meta.get_current_token();
    let res = parse_one_type(meta, tok);

    if token(meta, "|").is_ok() {
        // is union type
        let mut unioned = vec![ Box::new(res?) ];
        loop {
            match parse_one_type(meta, meta.get_current_token()) {
                Err(err) => return Err(err),
                Ok(t) => unioned.push(Box::new(t))
            };
            if token(meta, "|").is_err() {
                break;
            }
        }
        return Ok(Type::Union(unioned))
    }

    res
}
