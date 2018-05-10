use prelude::*;
use unicode_segmentation::UnicodeSegmentation as UCS;

#[derive(Debug)]
pub enum Type {
    Str,
    Int,
    Float,
    Bool,
}

#[derive(Debug)]
pub enum Param {
    Str(String),
    Int(i32),
    Float(f32),
    Bool(bool),
}

pub type Handler = Box<Fn(&[Param])>;
pub type Signature = Box<[Type]>;

pub struct Cmd {
    commands: HashMap<String, (Signature, Handler)>  // TODO: signature length in key to support overloading
}

impl Cmd {
    pub fn new() -> Self{
        Cmd {
            commands: HashMap::new(),
        }
    }

    pub fn register(self: &mut Self, name: &str, signature: Signature, handler: Handler) {
        self.commands.insert(name.to_string(), (signature, handler));
    }

    pub fn exec(self: &Self, input: &str) {

        let tokens = Self::tokenize(input);

        if tokens.len() > 0 {
            match self.commands.get(tokens[0]) {
                Some(command) => {
                    let params = Self::parse(&tokens[1..tokens.len()], &command.0);
                    command.1(&params);
                }
                None => println!("Unknown command \"{}\".", tokens[0])
            }
        }
    }

    fn parse(raw_params: &[&str], signature: &Signature) -> Vec<Param> {

        let mut result = Vec::new();

        for (index, ptype) in signature.iter().enumerate() {
            result.push(match *ptype {
                // TODO: all kinds of checks
                Type::Str => Param::Str(raw_params[index][1..raw_params[index].len()-1].to_string()),
                Type::Int => Param::Int(raw_params[index].parse().unwrap()),
                Type::Float => Param::Float(raw_params[index].parse().unwrap()),
                Type::Bool => Param::Bool(raw_params[index] == "true"),
            })
        }
        
        result
    }

    fn tokenize(input: &str) -> Vec<&str> {

        let input = input.trim();
        let mut start = 0;
        let mut within_string = false;
        let mut result = Vec::new();

        for (pos, ref grapheme) in UCS::grapheme_indices(input, true) {
            if within_string {
                if *grapheme == "\"" {
                    result.push(&input[start..pos + grapheme.len()]);
                    start = pos + grapheme.len();
                    within_string = false;
                }
            } else {
                if *grapheme == "\"" {
                    start = pos;
                    within_string = true;
                } else if *grapheme == " " {
                    if start < pos {
                        // start==pos right after a string ends
                        result.push(&input[start..pos]);
                    }
                    start = pos + grapheme.len();
                }
            }
        }

        if start < input.len() {
            // start==pos right after a string ends
            result.push(&input[start..input.len()]);
        }
        
        result
    }
}