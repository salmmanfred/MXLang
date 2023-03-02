pub mod standard;
use crate::parser::execute::Vars;
use crate::parser;
use crate::lib::standard::Standard;

pub fn load_library(str: &str, vars: &mut Vars){

    match standard::fetch_standard_lib(str){
        Standard::Lib(a)=>{
            let x = a.as_str();
            let nodes = parser::ast::gen_access(x);
            parser::execute::run(nodes, vars);
        }
        Standard::None =>{
            let nodes = parser::ast::gen(str);
            parser::execute::run(nodes, vars);
        }
    }





}
