// this file is responsible for the standard library and everything it includes

pub enum Standard{
    Lib(String),
    None
}

pub fn fetch_standard_lib(str: &str)->Standard{
    match str{
        "std" => Standard::Lib(include_str!("./std/std.MXLA").to_string()),
        _=>{
            Standard::None
        }
    }
}