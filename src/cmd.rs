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

pub type Handler<T> = Box<Fn(&mut T, &[Param])>;
pub type Signature = Vec<Type>;

pub struct Cmd<T> {
    commands: HashMap<String, HashMap<usize, (Signature, Handler<T>)>>,
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
    pub fn register(self: &mut Self, name: &str, signature: Signature, handler: Handler<T>) {
        let overloads = self.commands.entry(name.to_string()).or_insert(HashMap::new());
        overloads.insert(signature.len(), (signature, handler));
    }

    /**
     * attempts to execute the given console commands
     */
    pub fn exec(self: &Self, input: &str) {

        let lines = Self::tokenize(input);

        for tokens in lines.iter() {
            if tokens.len() > 0 {
                match self.commands.get(tokens[0]) {
                    Some(overloads) => {
                        if let Some(command) = overloads.get(&(tokens.len() - 1)) {
                            let params = Self::parse(&tokens[1..tokens.len()], &command.0);
                            let mut context = self.context.borrow_mut();
                            command.1(&mut context, &params);
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
     * parses list of parameter strings into list of typed values
     */
    fn parse(raw_params: &[&str], signature: &Signature) -> Vec<Param> {

        let mut result = Vec::new();

        for (index, ptype) in signature.iter().enumerate() {
            result.push(match *ptype {
                // TODO: all kinds of checks
                Type::Str => {
                    let param = &raw_params[index];
                    if &param[0..1] == "\"" {
                        Param::Str(param[1..param.len()-1].to_string())
                    } else {
                        Param::Str(param.to_string())
                    }
                },
                Type::Int => Param::Int(raw_params[index].parse().unwrap()),
                Type::Float => Param::Float(raw_params[index].parse().unwrap()),
                Type::Bool => Param::Bool(raw_params[index] == "true"),
            })
        }
        
        result
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