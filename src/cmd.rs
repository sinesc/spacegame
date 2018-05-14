use prelude::*;
use unicode_segmentation::UnicodeSegmentation as UCS;
use error::Error;

#[derive(Clone, Debug)]
pub enum Type {
    Str,
    Int,
    Float,
    Bool,
}

#[derive(Clone, Debug)]
pub enum Param {
    Str(String),
    Int(i32),
    Float(f32),
    Bool(bool),
}

impl Param {
    pub fn to_string(self: &Self) -> String {
        if let &Param::Str(ref ret) = self { ret.to_string() } else { "".to_string() }
    }
}

#[derive(Debug)]
pub enum CmdError {
    UnknownCommand,
    UnknownOverload(u32, Vec<u32>),
    InvalidArgument(u32, Type),
}

impl fmt::Display for CmdError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.description(), match self {
            CmdError::UnknownOverload(got, expected) => format!(". Got {}, expected {:?}", got, expected),
            CmdError::InvalidArgument(index, ty) => format!(". Expected {:?} as argument {}", ty, index),
            _ => "".to_string(),
        })
    }
}

impl error::Error for CmdError {
    fn description(&self) -> &str {
        match self {
            CmdError::UnknownCommand => "Unknown command",
            CmdError::UnknownOverload(_, _) => "Invalid number of arguments for given command",
            CmdError::InvalidArgument(_, _) => "Invalid argument type",
        }
    }
}

pub type Handler<T> = Box<Fn(&Cmd<T>, &[Param])>;

pub struct Cmd<T> {
    commands: HashMap<String, HashMap<usize, (Vec<Type>, Handler<T>)>>,
    context: RefCell<T>,
}

impl<T> Cmd<T> {

    /**
     * creates a new command parser instance
     */
    pub fn new(context: T) -> Self {
        Cmd {
            commands: HashMap::new(),
            context: RefCell::new(context),
        }
    }

    /**
     * returns a reference to the given context
     */
    pub fn context(self: &Self) -> ::std::cell::Ref<T> {
        self.context.borrow()
    }

    /**
     * returns a mutable reference to the given context
     */
    pub fn context_mut(self: &Self) -> ::std::cell::RefMut<T> {
        self.context.borrow_mut()
    }

    /**
     * registers a command+signature with the command processor
     */
    pub fn register(self: &mut Self, name: &str, signature: &[Type], handler: Handler<T>) {
        let overloads = self.commands.entry(name.to_string()).or_insert(HashMap::new());
        overloads.insert(signature.len(), (signature.to_vec(), handler));
    }

    /**
     * attempts to execute the console commands in the given string
     */
    pub fn exec(self: &Self, input: &str) {

        let lines = Self::tokenize(input);

        for tokens in lines.iter() {
            if tokens.len() > 0 {
                match self.commands.get(tokens[0]) {
                    Some(overloads) => {
                        if let Some(command) = overloads.get(&(tokens.len() - 1)) {
                            match Self::parse(&tokens[1..tokens.len()], &command.0) {
                                Ok(params) => command.1(self, &params),
                                Err(error) => println!("{}.", error),
                            }
                        } else {
                            println!("Command \"{}\" expects one of the following number of arguments: {:?}.", tokens[0], overloads.keys());
                        }
                    }
                    None => println!("Unknown command \"{}\".", tokens[0])
                }
            }
        }
    }

    /**
     * execute the given (single) console command using typed parameters
     */
    pub fn call(self: &Self, command: &str, params: &[Param]) {

        match self.commands.get(command) {
            Some(overloads) => {
                if let Some(command) = overloads.get(&params.len()) {
                    command.1(self, params);
                } else {
                    panic!("Command \"{}\" expects one of the following number of arguments: {:?}.", command, overloads.keys());
                }
            }
            None => panic!("Unknown command \"{}\".", command)
        }
    }

    /**
     * parses list of parameter strings into list of typed values
     */
    fn parse(raw_params: &[&str], signature: &[Type]) -> Result<Vec<Param>, CmdError> {

        let mut result = Vec::new();

        for (index, ptype) in signature.iter().enumerate() {
            result.push(match *ptype {
                // TODO: ugly
                Type::Str => {
                    let param = &raw_params[index];
                    if &param[0..1] == "\"" {
                        Param::Str(param[1..param.len()-1].to_string())
                    } else {
                        Param::Str(param.to_string())
                    }
                },
                Type::Int => {
                    if let Ok(result) = raw_params[index].parse() {
                        Param::Int(result)
                    } else {
                        return Err(CmdError::InvalidArgument(index as u32 + 1, Type::Int))
                    }
                },
                Type::Float => {
                    if let Ok(result) = raw_params[index].parse() {
                        Param::Float(result)
                    } else {
                        return Err(CmdError::InvalidArgument(index as u32 + 1, Type::Float))
                    }
                },
                Type::Bool => {
                    if raw_params[index] == "true" {
                        Param::Bool(true)
                    } else if raw_params[index] == "false" {
                        Param::Bool(false)
                    } else {
                        return Err(CmdError::InvalidArgument(index as u32 + 1, Type::Bool))
                    }
                }
            });
        }

        Ok(result)
    }

    /**
     * splits the input string into commands and tokens for each command
     */
    fn tokenize(input: &str) -> Vec<Vec<&str>> {

        let input = input.trim();
        let mut start = 0;
        let mut within_string = false;
        let mut commands = Vec::new();
        let mut tokens = Vec::new();

        // * is required since start==pos right after a string ends (can't look ahead and skip the space)
        // TODO: handle escaped "

        for (pos, ref grapheme) in UCS::grapheme_indices(input, true) {
            if within_string {
                if *grapheme == "\"" {
                    tokens.push(&input[start..pos + grapheme.len()]);
                    start = pos + grapheme.len();
                    within_string = false;
                }
            } else {
                if *grapheme == "\"" {
                    start = pos;
                    within_string = true;
                } else if *grapheme == " " {
                    if start < pos { // *
                        tokens.push(&input[start..pos]);
                    }
                    start = pos + grapheme.len();
                } else if *grapheme == ";" {
                    if start < pos { // *
                        tokens.push(&input[start..pos]);
                    }
                    commands.push(::std::mem::replace(&mut tokens, Vec::new()));
                    start = pos + grapheme.len();
                }
            }
        }

        if start < input.len() { // *
            tokens.push(&input[start..input.len()]);
        }

        if tokens.len() > 0 {
            commands.push(tokens);
        }

        commands
    }
}