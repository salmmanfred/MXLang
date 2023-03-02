//! Do to some weird reason you have to use LF line endings idono why
extern crate pest;



use std::collections::HashMap;

use crate::parser;
use crate::parser::ast::{self,InternalFunctions, Node};


/*
Simple language implemented with small amounts of packages
*/

/*
Variable struct this will contain all the variables in a hashmap
This will make its easy and fast to get all the variables
*/
#[derive(Clone, Debug)]
pub struct Vars {
    var: HashMap<String, Box<Node>>,
    functions: HashMap<String, Box<Node>>,
    pub draw_stack: Vec<(String,Box<Vec<Node>>)>,
}
impl Vars {
    pub fn new() -> Self {
        Vars {
            var: HashMap::new(),
            functions: HashMap::new(),
            draw_stack: Vec::new(),
        }
    }
    
    pub fn push(&mut self, str: String, val: Box<Node>) {
        match *val.clone() {
            Node::Function {
                args: _,
                insides: _,
            } => {
                let val2 = *val.clone();
                self.var.insert(str.clone(), Box::new(val2));
                self.functions.insert(str, val.clone());
            }

            _ => {
                self.var.insert(str, val);
            }
        }
    }
    pub fn get(&mut self, str: String) -> &Box<Node> {
        // println!("var: {}",str);
        
        let Some(a) = self.var.get(&str)else{

            panic!("No variable {:#?}",str);
        };
        a

    }

    /*pub fn clone_func(&mut self) -> Vars {
        Vars {
            var: self.functions.clone(),
            functions: self.functions.clone(),
        }
    }*/
}

pub fn execute_code() {
    //let args: Vec<String> = env::args().collect();
    /*
    initiates the program by generating the AST(Abstract syntax tree)
    */

    let nodes = parser::ast::gen("./test.MXLA");
    //println!("{:#?}",nodes);
    // Crates the variables for later use
    let mut var = Vars::new();
    // runs the entire program
    run(nodes, &mut var);
    
}
//#[decurse::decurse_unsound]
pub fn run(nodes: Vec<Node>, var: &mut Vars) {
    // This interprets everything in the vector

    for x in nodes {

        //Only some nodes can be run so this makes sure it is only them who come through
        match x {
            // This function assigns to variables
            Node::Varasgn { asgn, op, name } => {
                match op {
                    // var += number like in most languages
                    parser::ast::Op::Plus => {
                        /*
                        gets the correct variable then makes sure its an int by doing .to_int(var)
                        it then gets the int and increments it by one
                        then pushes the new value to the hashmap
                        */
                        let varl = var.get(name.clone()).to_owned();

                        match *varl {
                            Node::Array(a) => {
                                let mut a = a;
                                let asgn = asgn.unwrap_var(var);
                                a.push(*asgn);

                                var.push(name, Box::new(Node::Array(a)))
                            }
                            Node::String(a)=>{
                                let mut a = ast::clean_string(a);
                                let asgn = asgn.unwrap_var(var);
                                a.push_str(&asgn.to_print_clean());
                                
                                var.push(name,Box::new(Node::String(a)))
                            }

                            _ => {
                                let mut varl = varl.to_int(var);
                                varl += asgn.to_int(var);
                                var.push(name, Box::new(Node::Int(varl)))
                            }
                        }
                    }
                    // -= it works the same as the plus but negative
                    parser::ast::Op::Minus => {
                        let varl = var.get(name.clone()).to_owned();

                        match *varl {
                            Node::Array(a) => {
                                let mut a = a;
                                a.remove(asgn.to_int(var) as usize);

                                var.push(name, Box::new(Node::Array(a)))
                            }

                            _ => {
                                let mut varl = varl.to_int(var);
                                varl -= asgn.to_int(var);
                                var.push(name, Box::new(Node::Int(varl)))
                            }
                        }
                    }
                    // only assigns what it wants nothing special here
                    parser::ast::Op::Equall => {
                        let asgn = asgn.unwrap_var(var);
                        var.push(name, asgn)
                    }
                    _ => panic!("not a legal assign operator"),
                }
            }
            // Internal functions like print!() or input!()
            // You can know if the function is internal because internal functions have !
            Node::InternalFunction { typ, args } => {
                match typ {
                    InternalFunctions::Print(a) => {
                        // creates a string from the first argument and if there is more it will
                        // add the in the for loop later
                        let mut string = args[0].to_print_var_clean(var);
                        // This gets all the variables and then converts them to &str by using .to_print_var(var)
                        // it then pushes them to the string
                        for x in 1..args.len() {
                            string.push_str(&args[x].to_print_var_clean(var))
                        }
                        // If its true it prints it with a new line or without if it is false
                        match a {
                            true => println!("{string}"),
                            false => print!("{string}")
                        };
                    }
                    InternalFunctions::LenArray => {
                        let len = args[0].unwrap_var(var).unwrap_array().len();
                        panic!("len {len}");
                    }
                    InternalFunctions::Import =>{
                        // function to import all files and run them
                        // import!("test.sm","std.sm",etc..)
                        // import it on top of the current variables to make sure its like an extention of the
                        // current working file

                        for x in args{
                            let arg = x.to_print_var_clean(var);
                            // most of the work is offloaded to another module
                             crate::lib::load_library(arg.as_str(), var)
                        }

                    }
                    
                }
            }
            Node::RunFunction { name, args } => {
                run_function(Node::RunFunction { name, args }, var);
            }
            // If statement
            Node::Ifs {
                arg1,
                arg2,
                if_op,
                insides,
            } => {
                match if_op {
                    // gets the correct type of if statement
                    parser::ast::Op::EquallEquall => {
                        /*
                        if node == node
                        Here it gets the arguments makes them strings and then compares them
                        */

                        if arg1.unwrap_var(var).unwrap_var(var).to_print_var(var) == arg2.unwrap_var(var).to_print_var(var) {
                            // it makes a new instance of run to run what is inside of the if
                            // it then sends all the variables to it (sicne its still the same scope)
                            let _ = Box::new(run(insides, var));
                        }
                    }
                    parser::ast::Op::GreatEquall => {
                        /*
                        if node >= node
                        Here it gets the arguments makes them ints and then compares them
                        */

                        if arg1.unwrap_var(var).to_int(var) >= arg2.unwrap_var(var).to_int(var) {
                            // works like in the ==
                            let _ = Box::new(run(insides, var));
                        }
                        //  println!("{:#?}",if_op)
                    }
                    parser::ast::Op::LessEquall => {
                        // same as in the previous but <=
                        if arg1.unwrap_var(var).to_int(var) <= arg2.unwrap_var(var).to_int(var) {
                            // works like in the ==

                            let _ = Box::new(run(insides, var));
                        }
                    }
                    parser::ast::Op::NotEquall => {
                        // println!("x {}, {}",arg1.to_print_var(&mut var),arg2.to_print_var(&mut var));

                        // works like in the string version just !=
                        if arg1.unwrap_var(var).to_print_var(var) != arg2.unwrap_var(var).to_print_var(var) {
                            let _ = Box::new(run(insides, var));
                        }
                    }
                    _ => todo!(),
                }
            }
            Node::Nop =>{}
            _ => panic!("not an operation"),
        }
    }
}

pub fn run_function(n: Node, var: &mut Vars) -> Vars {
    let (name, args) = match n {
        Node::RunFunction { name, args } => (name, args),
        _ => {
            panic!("not a function")
        }
    };

    /*
    Creates a new variable holder
    then gets the vectors from the function
    these vectors contain the arguments that are needed and the lines of code inside of the function
    it then iterates through the arguments provided by the Node::RunFunction
    the iteration is also enumerated so it can get the corresponding arg from fun_arg
    it then gets the name so that it can crate the new variable for var2
    it then crates a box that will run everything
    */

    let mut var2 = var.clone();

    let (fun_arg, fun_in) = var.get(name).unwrap_fun();
    /*println!("a,{}",args.len());
    println!("b{}",fun_arg.len());
    println!("{:#?}",fun_arg[0].check_var_empt());
    println!("{:#?}",args);*/

    if !fun_arg[0].check_var_empt() {
        for (a, x) in args.iter().enumerate() {
            let v = match x.to_owned() {
                Node::Var(a) => var.get(a).to_owned(),
                a => Box::new(a),
            };

            var2.push(fun_arg[a].var_get_name(), v)
        }
    }

    run(fun_in, &mut var2);
    if var2.draw_stack.len() != 0{
       
        var.draw_stack.extend(var2.draw_stack.clone());
    }
    var2
}

