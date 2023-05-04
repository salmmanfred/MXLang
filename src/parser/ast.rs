use std::collections::HashMap;

use crate::{parser::{self, execute}, compile::jit::Types};
use cranelift::{
    codegen::ir::VariableArgs,
    prelude::{isa::Builder, types::I64, FunctionBuilder, InstBuilder, Value, Variable},
};
use openfile;
use pest::Parser;
use pest_derive::Parser;
#[derive(Parser)]
#[grammar = "./parser/gram.pest"]
pub struct Pars;

#[derive(Debug, PartialEq, Clone)]
pub enum Op {
    Plus,
    Minus,
    Equall,
    EquallEquall,
    LessEquall,
    GreatEquall,
    NotEquall,
}
impl Op {
    pub fn to_op(str: &str) -> Op {
        match str {
            "=" => Op::Equall,
            "+=" => Op::Plus,
            "-=" => Op::Minus,
            a => panic!("{a} not an operation"),
        }
    }
    pub fn to_ifop(str: &str) -> Op {
        match str {
            "==" => Op::EquallEquall,
            ">=" => Op::GreatEquall,
            "<=" => Op::LessEquall,
            "!=" => Op::NotEquall,
            _ => panic!("not an .operation"),
        }
    }
}
#[derive(Debug, PartialEq, Clone)]
pub enum InternalFunctions {
    Print(bool),
    LenArray,
    Import,
}
impl InternalFunctions {
    pub fn to_intfun(str: &str) -> InternalFunctions {
        match str {
            "print" => InternalFunctions::Print(false),
            "println" => InternalFunctions::Print(true),
            "len" => InternalFunctions::LenArray,
            "import" => InternalFunctions::Import,

            _ => panic!("not internal"),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Node {
    //TODO: write wtf all this shit is I forgo
    Int(i64),
    String(String),
    Varasgn {
        op: Op,
        name: String,
        asgn: Box<Node>,
    },
    Var(String),

    InternalFunction {
        typ: InternalFunctions,
        args: Vec<Node>,
    },
    Array(Vec<Node>),
    GetArray(String, Box<Node>),
    Ifs {
        arg1: Box<Node>,
        if_op: Op,
        arg2: Box<Node>,
        insides: Vec<Node>,
    },
    Function {
        args: Vec<Node>,
        insides: Vec<Node>,
    },
    RunFunction {
        name: String,
        args: Vec<Node>,
    },
    Nop,
}

//TODO? possibly change all the use of vars to just a hashmap but that hashmap can then dynamically change to
//TODO? <String, Variables> for jit and <String, Node> for execute
impl Node {
    pub fn unwrap_fun(&self) -> (Vec<Node>, Vec<Node>) {
        match self {
            Node::Function { args, insides } => (args.to_vec(), insides.to_vec()),
            _ => panic!("Unwrap function fail"),
        }
    }
    pub fn unwrap_array(&self) -> Vec<Node> {
        match self {
            Node::Array(a) => return a.to_owned(),

            a => {
                panic!("Not an array {:?}", a);
            }
        }
    }

    fn unwrap_backend(&self,
        variables: &mut HashMap<String, (Variable,Types)>,
        builder: &mut FunctionBuilder,) ->(Value,Types)
    {
        match self {
            Node::Var(a) => {
                let var = *variables.get(a).unwrap();
                (builder.use_var(var.0),var.1)
            }
            Node::Int(a) => (builder.ins().iconst(I64, *a),Types::Int),
            Node::GetArray(a, b) => {
                todo!()
            }
            Node::String(a) => {
                panic!("String not impl");
            }
            _ => {
                todo!()
            }
        }
    }
    pub fn unwrap_value(
        &self,
        variables: &mut HashMap<String, (Variable,Types)>,
        builder: &mut FunctionBuilder,
    ) -> Value {
       return self.unwrap_backend(variables, builder).0
    }
    pub fn unwrap_value_int(
        &self,
        variables: &mut HashMap<String, (Variable,Types)>,
        builder: &mut FunctionBuilder,
    ) ->Value {
        let vt = self.unwrap_backend(variables, builder);
        if let Types::Int = vt.1{
            return vt.0
        }
        panic!("not an int")
    }
   

    pub fn unwrap_var(&self, vars: &mut parser::execute::Vars) -> Box<Node> {
        match self {
            Node::Var(a) => {
                let a = vars.get(a.to_string()).to_owned();

                a //panic!("{:#?}",a)
            }
            Node::GetArray(a, b) => {
                //array = ["test","test2","test3"]
                //t = array[0]
                let a = vars.get(a.to_string()).to_owned();
                let a = a.unwrap_array();
                Box::new(a[b.to_int(vars) as usize].clone())
            }
            Node::RunFunction { name: _, args: _ } => {
                //panic!("is function");
                let node = self.clone();
                let mut var = vars.clone();

                let mut var = execute::run_function(node, &mut var);

                //execute::run(vec![node],&mut var);
                // println!("{:#?}",var);
                let a = var.get("return".to_string()).to_owned();

                let returnstm = a.unwrap_var(&mut var);

                returnstm
            }
            Node::InternalFunction { typ, args } => match typ {
                InternalFunctions::LenArray => {
                    let len = args[0].unwrap_var(vars).unwrap_array().len();
                    return Box::new(Node::Int(len as i64));
                }
                a => {
                    panic!("No return statement for {:#?}", a)
                }
            },
            a => {
                // println!("here");
                Box::new(a.clone())
            }
        }
    }
    pub fn var_get_name(&self) -> String {
        match self {
            Node::Var(a) => return a.to_string(),
            _ => {
                panic!("no not that kind of var")
            }
        }
    }
    pub fn to_print(&self) -> String {
        match self {
            Node::String(a) => a.to_string(),
            Node::Int(a) => a.to_string(),
            a => {
                println!("{:#?}", a);
                panic!("cant go to print")
            }
        }
    }

    pub fn to_print_var_clean(&self, vars: &mut parser::execute::Vars) -> String {
        let v = self.unwrap_var(vars);
        match *v {
            Node::String(a) => {
                let a = a;
                if a == "" {
                    return "".to_string();
                }

                let mut b: Vec<char> = a.chars().collect();
                if b.len() <= 1 {
                    return "".to_string();
                }
                if b[0] == '"' {
                    b.remove(0);
                }
                if b[b.len() - 1] == '"' {
                    b.remove(b.len() - 1);
                }
                b.iter().collect()
            }
            Node::Int(a) => a.to_string(),
            Node::Var(a) => vars.get(a.to_string()).to_print_clean(),
            Node::GetArray(a, b) => {
                let a = vars.get(a.to_string()).to_owned();
                let a = a.unwrap_array();
                let a = clean_string(a[b.to_int(vars) as usize].to_print());
                a
            }
            a => {
                println!("{:#?}", a);
                panic!("cant go to print")
            }
        }
    }
    pub fn to_print_clean(&self) -> String {
        match self {
            Node::String(a) => clean_string(a.to_string()),
            Node::Int(a) => a.to_string(),
            a => {
                println!("{:#?}", a);
                panic!("cant go to print")
            }
        }
    }
    pub fn to_print_var(&self, vars: &mut parser::execute::Vars) -> String {
        match self {
            Node::String(a) => a.to_string(),
            Node::Int(a) => a.to_string(),
            Node::Var(a) => vars.get(a.to_string()).to_print(),
            a => {
                println!("{:#?}", a);
                panic!("cant go to print")
            }
        }
    }
    fn to_int_sub(&self) -> i64 {
        match self {
            Node::Int(a) => a.to_string().parse::<i64>().unwrap(),
            a => {
                println!("{:#?}", a);
                panic!("cant go to print int")
            }
        }
    }
    #[allow(dead_code)]
    pub fn unwrap_int(&self) -> i64 {
        match self {
            Node::Int(a) => a.to_string().parse::<i64>().unwrap(),

            a => {
                println!("{:#?}", a);
                panic!("cant go to print int")
            }
        }
    }
    pub fn to_int(&self, vars: &mut parser::execute::Vars) -> i64 {
        match self {
            Node::Int(a) => a.to_string().parse::<i64>().unwrap(),
            Node::Var(a) => vars.get(a.to_string()).to_int_sub(),
            a => {
                println!("{:#?}", a);
                panic!("cant go to print int")
            }
        }
    }

    pub fn check_var_empt(&self) -> bool {
        match self {
            Node::Var(a) => {
                if a == &"".to_string() {
                    return true;
                }
                return false;
            }
            _ => return false,
        }
    }
    pub fn get_var_if_jit(
        &self,
        variables: &mut HashMap<String, (Variable, crate::compile::jit::Types)>,
        builder: &mut FunctionBuilder,
    ) -> Value {
        match self {
            Node::Int(a) => builder.ins().iconst(I64, *a),
            Node::Var(a) => {
                let v = *variables.get(a).unwrap();
                if let Types::Int = v.1{
                return builder.use_var(v.0)

                }
                panic!("Wrong type {:#?}", v.1)
            },
            _ => {
                todo!()
            }
        }
    }
}
pub fn gen(file: &str) -> Vec<Node> {
    let x = openfile::read_file(file).unwrap();
    /*let x: String = match code_ok_err{
        Ok(a) =>{
            return a.to_string()
        }
        Err(a) =>{
            let err_mes =  a.to_string().as_str();

            return "println!(\"there was an error loading the file\")".to_string()

        }

    };
    println!("{x}");

    */
    let mut ast = vec![];
    let pairs = Pars::parse(Rule::program, &x).unwrap();
    //println!("{:#?}", pairs);

    for pair in pairs {
        if let Rule::EOI = pair.as_rule() {
        } else {
            ast.push(build_ast(pair));
        }
    }
    ast
}
pub fn gen_access(x: &str) -> Vec<Node> {
    let mut ast = vec![];
    let pairs = Pars::parse(Rule::program, &x).unwrap();
    //println!("{:#?}", pairs);

    for pair in pairs {
        if let Rule::EOI = pair.as_rule() {
        } else {
            ast.push(build_ast(pair));
        }
    }
    ast
}
pub fn build_ast(pair: pest::iterators::Pair<Rule>) -> Node {
    match pair.as_rule() {
        Rule::var => {
            let inner = pair.into_inner().next();
            return build_ast_from_var(inner.unwrap());
        }
        Rule::runfun => {
            let mut intpair: Vec<Node> = Vec::new();
            let mut inner_expr = pair.clone().into_inner();
            inner_expr.next().unwrap();

            for x in inner_expr {
                // println!("in, {x}");

                intpair.push(build_ast_from_expr(x.into_inner().next().unwrap()));
            }

            let inner = pair.into_inner().next();
            Node::RunFunction {
                name: inner.unwrap().as_str().to_string(),
                args: intpair,
            }
        }
        Rule::intfun => {
            let mut intpair: Vec<Node> = Vec::new();
            let mut inner_expr = pair.clone().into_inner();
            inner_expr.next().unwrap();

            for x in inner_expr {
                //println!("in, {x}");

                intpair.push(build_ast_from_expr(x.into_inner().next().unwrap()));
            }

            let inner = pair.into_inner().next();
            Node::InternalFunction {
                typ: InternalFunctions::to_intfun(inner.unwrap().as_str()),
                args: intpair,
            }
        }
        Rule::ifs => {
            let mut inside: Vec<Node> = Vec::new();
            let mut inner_expr = pair.clone().into_inner();

            // inner_expr.next().unwrap();
            let expr1 =
                build_ast_from_expr(inner_expr.next().unwrap().into_inner().next().unwrap());
            let if_op = Op::to_ifop(inner_expr.next().unwrap().as_str());
            let expr2 =
                build_ast_from_expr(inner_expr.next().unwrap().into_inner().next().unwrap());

            for x in inner_expr {
                //  println!("l, {x}");

                inside.push(build_ast(x));
            }
            Node::Ifs {
                arg1: Box::new(expr1),
                if_op: if_op,
                arg2: Box::new(expr2),
                insides: inside,
            }
        }
        Rule::string => return Node::Nop,

        a => {
            panic!("{:#?}", a);
        }
    }
}
pub fn build_ast_from_var(pair: pest::iterators::Pair<Rule>) -> Node {
    match pair.as_rule() {
        Rule::asgn => {
            let mut inner = pair.clone().into_inner();

            let name = inner.next().unwrap().as_str();

            let ops = inner.next().unwrap().as_str();

            let mut x = inner.next().unwrap().into_inner();
            Node::Varasgn {
                op: Op::to_op(ops),
                name: name.to_string(),
                asgn: Box::new(build_ast_from_expr(x.next().unwrap())),
            }
        }
        _ => {
            panic!("")
        }
    }
}
pub fn build_ast_from_expr(pair: pest::iterators::Pair<Rule>) -> Node {
    match pair.as_rule() {
        Rule::string => {
            //let mut inner = pair.clone().into_inner();
            Node::String(pair.as_str().to_string())
        }
        Rule::name => Node::Var(pair.as_str().to_string()),
        Rule::int => Node::Int(pair.as_str().parse().unwrap()),
        Rule::fun => {
            let mut inside: Vec<Node> = Vec::new();
            let mut args: Vec<Node> = Vec::new();

            let mut inner_expr = pair.clone().into_inner();
            // panic!("i {:#?}",inner_expr);
            for x in inner_expr.next().unwrap().into_inner() {
                //    println!("a, {x}");

                args.push(build_ast_from_expr(x.into_inner().next().unwrap()));
            }
            for x in inner_expr {
                // println!("l, {x}");

                inside.push(build_ast(x));
            }

            Node::Function {
                args: args,
                insides: inside,
            }
        }
        Rule::array => {
            let mut args: Vec<Node> = Vec::new();

            let mut inner_expr = pair.clone().into_inner();
            // panic!("i {:#?}",inner_expr);
            for x in inner_expr.next().unwrap().into_inner() {
                //    println!("a, {x}");

                args.push(build_ast_from_expr(x.into_inner().next().unwrap()));
            }

            // panic!("args {:#?}",name);
            Node::Array(args)
        }
        Rule::getarr => {
            let mut inner_expr = pair.clone().into_inner();
            let inner_exprn = build_ast_from_expr(inner_expr.next().unwrap());

            let inner_expr2 =
                build_ast_from_expr(inner_expr.next().unwrap().into_inner().next().unwrap());
            //panic!("inner {:#?}",inner_exprn.var_get_name());

            Node::GetArray(inner_exprn.var_get_name(), Box::new(inner_expr2))
        }
        Rule::runfun | Rule::intfun => {
            let pair = pair.clone();
            let funstmt = build_ast(pair);
            funstmt
        }

        a => {
            panic!("{:#?}", a)
        }
    }
}

pub fn clean_string(a: String) -> String {
    if a == "" {
        return "".to_string();
    }

    let mut b: Vec<char> = a.chars().collect();

    if b.len() <= 1 {
        return "".to_string();
    }
    if b[0] == '"' {
        b.remove(0);
    }
    if b[b.len() - 1] == '"' {
        b.remove(b.len() - 1);
    }

    b.iter().collect()
}
