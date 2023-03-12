mod parser;

mod traits;
#[allow(special_module_name)]
mod lib;
mod compile;

extern crate pest_derive;








fn main() {


    //parser::execute::execute_code();
    compile::jit::test_compile();
    compile::jit::test_compile2(parser::ast::gen("./test.MXLA"))

    //TODO: libraries import!("file.file") (done)
    //TODO: standard library (WUP)

    //TODO: function(function()) (not a priority)
    //TODO: var = intfun!() (done)
    //TODO: APPEND strings so string += string creates string (done)
    //TODO: ADD push with array += value (done)
    //TODO: REMOVE push with array -= index (done)
    //TODO: ADD len(array) (done)
    //TODO: Add return for functions (done)

    


    


   
}
