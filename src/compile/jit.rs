use std::alloc::{Layout, alloc};
use std::collections::HashMap;
use std::mem;

use crate::parser::ast::{self, Node};
use cranelift::codegen::ir::{ FuncRef, UserFuncName};
use cranelift::codegen::{
    ir::{types::I64, AbiParam, Function, Signature},
    isa::CallConv,
};
use cranelift::frontend::{FunctionBuilder, FunctionBuilderContext};

use cranelift::prelude::{
    types, Configurable, EntityRef, InstBuilder, Type, Value,
    Variable,
};


use cranelift::codegen::{isa, settings, Context};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{FuncId, Linkage, Module};
use target_lexicon::Triple;



//TODO: do it in a better way
unsafe extern "sysv64" fn malloc(size: i64)->i64{
    let layout = Layout::from_size_align(8*size as usize, 2).unwrap();

    let ptr_raw = alloc(layout);
    if ptr_raw.is_null(){
        panic!("Failed to allocate memory size: {size}");
    }
    let p = ptr_raw as i64+8;
    


    println!("ptrraw {:#?}, P: {}",ptr_raw, p );
    p
}
unsafe extern "sysv64" fn poke(addr: i64, data: i64){
    *(addr as *mut i64) = data;
}
unsafe extern "sysv64" fn peek(addr: i64)->i64{
    *(addr as *mut i64)
}

fn compile_code(
    code: Vec<Node>,
    module: &mut JIT,
    builder: &mut FunctionBuilder,
    variables: &mut HashMap<String, Variable>,
    funcs: &mut HashMap<String, FuncRef>,
    arrays: &mut HashMap<String, i64>,
) {

    //TODO: This also in a better way
    let (mallocsig, malloc_addr) = {
        let mut malloc_sig = Signature::new(CallConv::SystemV);
        malloc_sig.params.push(AbiParam::new(types::I64));
        

        malloc_sig.returns.push(AbiParam::new(I64));
        let write_sig = builder.import_signature(malloc_sig);

        let malloc_addr = malloc as *const () as i64;
        let malloc_addr = builder.ins().iconst(module.pointer_type, malloc_addr);
        (write_sig, malloc_addr)
    };
    let (poke_sig, poke_addr) = {
        // ! when chaning these dont forget to change the function
        let mut malloc_sig = Signature::new(CallConv::SystemV);
        malloc_sig.params.push(AbiParam::new(types::I64));
        malloc_sig.params.push(AbiParam::new(types::I64));


       
        let write_sig = builder.import_signature(malloc_sig);

        let malloc_addr = poke as *const () as i64;
        let malloc_addr = builder.ins().iconst(module.pointer_type, malloc_addr);
        (write_sig, malloc_addr)
    };
    let (peek_sig, peek_addr) = {
        // ! when chaning these dont forget to change the function
        let mut malloc_sig = Signature::new(CallConv::SystemV);
        malloc_sig.params.push(AbiParam::new(types::I64));
       


        malloc_sig.returns.push(AbiParam::new(I64));
        let write_sig = builder.import_signature(malloc_sig);

        let malloc_addr = peek as *const () as i64;
        let malloc_addr = builder.ins().iconst(module.pointer_type, malloc_addr);
        (write_sig, malloc_addr)
    };

    for x in code {
        match x {
            Node::Varasgn { op, name, asgn } => {
                let name_alt = name.clone(); //TODO: needs to be done in a better way

                match op {
                    ast::Op::Equall => {
                        
                        
                            match *asgn {
                                //TODO: better way of doing this?
                                Node::Int(a) => {
                                    let var = Variable::new(module.varid());

                                    builder.declare_var(var, module.pointer_type);

                                    let num = builder.ins().iconst(I64, a);
                                    builder.def_var(var, num);
                                    variables.insert(name, var);
                                }
                                Node::Array(a)=>{

                                    
                                    //This is so that variables created at runtime can interact with it. 
                                    /*unsafe{
                                        let len = a.len()+1;
                                        let layout = Layout::from_size_align(8*len, 2).unwrap();
                                        let ptr_raw = alloc(layout);
                                        if ptr_raw.is_null(){
                                            panic!("Failed to allocate memory for array: {name}");
                                        }
                                        let p = ptr_raw as i64+8;
                                            
                                        //|len|obj1|obj2|...
                                        //     ^^^ this is wha the pointer points to (trust me)
                                        // this might be a handicap later but dont worry about it 

                                        println!("{p}");

                                        // setting the lenght of the array it does not include the lenght in the lenght
                                        *((p-8) as *mut i64) = len as i64 -1;

                                        // this looks like someones cat was given a computer
                                        for x in 0..(len-1){
                                            *((p+8*(x as i64)) as *mut i64) = a[x].unwrap_int();
                                        }

                                        let var = Variable::new(module.varid());

                                        builder.declare_var(var, module.pointer_type);
                                        //let ptr1  =  p as *mut i64;

                                        let num = builder.ins().iconst(I64, p);
                                        builder.def_var(var, num);
                                        variables.insert(name.clone(), var);
                                        arrays.insert(name.clone(), p);


                                    }*/


                                    // An idea is to add before len an address to a continuation of the array 
                                    //|addr|len|obj1|obj2|..|objn|
                                    //          ^^^ p still points here but when i becomes bigger than len(-8) or equall it looks if addr (-16) 
                                    // points anywhere if it does it jumps there and continues this then has the same layout 
                                    // benifits: no need to realloc the entire thing if the current position in memory cannot support it
                                    // negatives: slower

                                    //|len|obj1|obj2|...
                                    //     ^^^ this is wha the pointer points to (trust me)
                                    // this might be a handicap later but dont worry about it 

                                    let varval = builder.ins().iconst(I64, a.len() as i64 +8);
                                    println!("{}",a.len() as i64 as usize);
                                    let num = builder.ins().call_indirect(mallocsig, malloc_addr, &[varval]);
                                    let num = builder.func.dfg.inst_results(num)[0];
                                    // Store the size 
                                    let addr_size = builder.ins().iadd_imm(num, -8);
                                    builder.ins().call_indirect(poke_sig, poke_addr, &[addr_size, varval]);
                                    let mut addr = num;
                                    //TODO: this looks stupid in code any better way of doing it? 
                                    for x in 0..a.len(){                                    
                                        let varval = builder.ins().iconst(I64, a[x].unwrap_int());
                                        //TODO                                                            ^^^ change to an internal unwrap_value
                                        builder.ins().call_indirect(poke_sig, poke_addr, &[addr, varval]);
                                        addr = builder.ins().iadd_imm(addr, 8);
                                        println!("e");

                                    }
                                    


                                    let var = Variable::new(module.varid());

                                    builder.declare_var(var, module.pointer_type);
                                    

                                    
                                    builder.def_var(var, num);
                                    variables.insert(name.clone(), var);




                                }
                                
                                Node::GetArray(a,b ) =>{
                                    /*unsafe{
                                        let addr = arrays.get(&a).unwrap();
                                        let len = *((addr-8) as *mut i64);

                                        if b.unwrap_int() >= len{
                                            panic!("index out of bounds");
                                        }
                                        let value = *((addr+8*b.unwrap_int()) as *mut i64);
                                        let var = Variable::new(module.varid());
                                        builder.declare_var(var, module.pointer_type);
                                        let varval = builder.ins().iconst(I64, value);
                                        builder.def_var(var, varval);
                                        variables.insert(name, var);
                                        
                                    }*/

                                    let var = variables.get(&a).unwrap();
                                    let varval = builder.use_var(*var);
                                    let addr  =builder.ins().iadd_imm(varval, b.unwrap_int()*8);
                                    //TODO                                                  ^^^ change to an internal unwrap_value

                                    let num = builder.ins().call_indirect(peek_sig, peek_addr, &[addr]);
                                    let num = builder.func.dfg.inst_results(num)[0];

                                    let var = Variable::new(module.varid());

                                    builder.declare_var(var, module.pointer_type);
                                    

                                    
                                    builder.def_var(var, num);
                                    variables.insert(name.clone(), var);

                                  

                                }
                                Node::Var(a) => {
                                    let var = variables.get(&a).unwrap();
                                    let varval = builder.use_var(*var);

                                    let var = Variable::new(module.varid());

                                    builder.declare_var(var, module.pointer_type);
                                    builder.def_var(var, varval);
                                    variables.insert(name, var);
                                }
                                Node::Function { args, insides } => {
                                    // TODO: add return somehow
                                    let mut arg_typ: Vec<Type> = Vec::new();
                                    println!("aa{:#?}", args);
                                    for x in args.clone() {
                                        match x {
                                            Node::Int(_) => arg_typ.push(I64),
                                            Node::Var(_) => {
                                                arg_typ.push(I64);
                                            }
                                            _ => {}
                                        }
                                    }
                                    let (funcid2, mut func2ctx) =
                                        module.create_function(&arg_typ, &[I64]);

                                    function_compiler_handler(
                                        args,
                                        insides,
                                        module,
                                        &mut func2ctx,
                                        funcid2,
                                        true,
                                    );

                                    let funcref =
                                        module.module.declare_func_in_func(funcid2, builder.func);
                                    funcs.insert(name, funcref);
                                }
                                Node::RunFunction { name, args } => {
                                    let func = *funcs.get(&name).unwrap();
                                    let var = Variable::new(module.varid());

                                    builder.declare_var(var, module.pointer_type);
                                    //TODO: add args

                                    let mut argsval: Vec<Value> = Vec::new();
                                    println!("{:?}", args);
                                    for arg in args {
                                        match arg {
                                            Node::Int(a) => {
                                                argsval.push(builder.ins().iconst(I64, a));
                                            }
                                            Node::Var(a) => {
                                                if a == "" {
                                                } else {
                                                    todo!()
                                                }
                                            }
                                            _ => {
                                                todo!()
                                            }
                                        }
                                    }

                                    let ins = builder.ins().call(func, &argsval);
                                    let num = builder.func.dfg.inst_results(ins)[0];

                                    builder.def_var(var, num);
                                    variables.insert(name_alt, var);
                                }
                                _ => {
                                    todo!()
                                }
                            }
                        
                    }
                    ast::Op::Plus => {
                        if variables.contains_key(&name) {

                            //TODO maybe rewrite it as its done in the interpreter
                            // panic!("not a var {}, {:#?}",name, variables);
                            match *asgn {
                                Node::Int(a) => {
                                    let v = builder.use_var(*variables.get(&name).unwrap());
                                    let num_plus_v = builder.ins().iadd_imm(v, a);
                                    builder.def_var(*variables.get(&name).unwrap(), num_plus_v);
                                }
                                Node::Var(a) => {
                                    let v = builder.use_var(*variables.get(&name).unwrap());
                                    let v2 = builder.use_var(*variables.get(&a).unwrap());

                                    let num_plus_v = builder.ins().iadd(v, v2);
                                    builder.def_var(*variables.get(&name).unwrap(), num_plus_v);
                                }
                                _ => {
                                    todo!()
                                }
                            }
                        }
                    }
                    _ => {
                        todo!()
                    }
                }
            }
            Node::Ifs {
                arg1,
                if_op,
                arg2,
                insides,
            } => {
                
                let arg1val: Value = arg1.get_var_if_jit(variables, builder);
                
                let arg2val: Value = arg2.get_var_if_jit(variables, builder);
               
                match if_op {
                    ast::Op::EquallEquall => {
                        let c = builder.ins().isub(arg1val, arg2val);
                        let then_block = builder.create_block();
                        let else_block = builder.create_block();
                        let merge_block = builder.create_block();

                        /*  builder.seal_block(then_block);
                        builder.seal_block(else_block);*/

                        // else is the new then
                        builder.ins().brif(c, else_block, &[], then_block, &[]);

                        builder.switch_to_block(then_block);
                        builder.seal_block(then_block);

                        compile_code(insides, module, builder, variables, funcs,arrays);

                        builder.ins().jump(merge_block, &[]);

                        builder.switch_to_block(else_block);
                        builder.seal_block(else_block);
                        builder.ins().jump(merge_block, &[]);

                        builder.switch_to_block(merge_block);
                        builder.seal_block(merge_block);
                    }

                    _ => {
                        todo!()
                    }
                }
            }
            _ => {
                todo!()
            }
        }
    }
}

fn function_compiler_handler(
    args: Vec<Node>,
    code: Vec<Node>,
    module: &mut JIT,
    ctx: &mut Context,
    funcid: FuncId,
    TEMPARG: bool,
) {
    let mut buildctx = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut ctx.func, &mut buildctx);
    let entryblock = builder.create_block();
    builder.seal_block(entryblock);

    builder.append_block_params_for_function_params(entryblock);
    builder.switch_to_block(entryblock);

    let mut variables: HashMap<String, Variable> = HashMap::new();
    let mut funcs: HashMap<String, FuncRef> = HashMap::new();
    let mut arrays: HashMap<String, i64> = HashMap::new();

    for (i, arg) in args.iter().enumerate() {
        if let Node::Var(name) = arg {
            let var = Variable::new(module.varid());
            let argin = builder.block_params(entryblock)[i];
            builder.declare_var(var, module.pointer_type);

            let num = argin;
            builder.def_var(var, num);
            variables.insert(name.to_string(), var);
        }
    }

    compile_code(code, module, &mut builder, &mut variables, &mut funcs, &mut arrays);
    //println!("vars: {:#?}",variables);
    if TEMPARG {
        let v = builder.use_var(*variables.get("y").unwrap());
        let v2 = builder.use_var(*variables.get("arr").unwrap());


        builder.ins().return_(&[v2,v]);
    } else {
        builder.ins().return_(&[]);
    }

    builder.finalize();

    println!("{}", ctx.func);
    //println!("{:#?}",ctx.func.layout);

    module.module.define_function(funcid, ctx).unwrap();

    println!("d4");
}

pub fn test_compile2(code_snip: Vec<ast::Node>) {
    let mut l = JIT::new();
    let (_mainfuncid, mut ctx) = l.create_function(&[], &[I64,I64]);
    //l.module.declare_func_in_func(func_id, func)

    function_compiler_handler(Vec::new(), code_snip, &mut l, &mut ctx, _mainfuncid, true);

    l.module.finalize_definitions().unwrap();

    let code = l.module.get_finalized_function(_mainfuncid);

    unsafe {
        let code_fn: unsafe extern "sysv64" fn() -> (i64,i64) =
            mem::transmute::<_, unsafe extern "sysv64" fn() -> (i64,i64)>(code);
        // unsafe extern "sysv64" fn(usize) ->usize
        let (f,a) = code_fn();
       
       // let ptraddr1: *mut  i64 =  f;
        //let ptr3 = std::ptr::read(f);
        
        let ptr2 = f as *const i64;
        let ptr3 = std::ptr::read(ptr2);
        for x in 0..5{
            let ptr2 = (f+8*x) as *const i64;
            let ptr33 = std::ptr::read(ptr2);
            println!("index {x}: {ptr33}");
        }
        let ptr2 = (f+8) as *const i64;
        let ptr33 = std::ptr::read(ptr2);
        println!(" const i64: {:#?}, ret: {:#?}, data: {:#?}, data+8: {}",ptr2,f,ptr3, ptr33);

        println!("{a}");
    };
}

pub struct JIT {
    module: JITModule,
    fn_refs: Vec<FuncRef>,
    pub builder_ctx: FunctionBuilderContext,
    fn_ids: Vec<FuncId>,
    sigs: Vec<Signature>,
    pointer_type: Type,
    refid: u32,
    varid: usize,
}
impl JIT {
    pub fn new() -> Self {
        let mut flag_builder = settings::builder();
        flag_builder.set("use_colocated_libcalls", "false").unwrap();
        flag_builder.set("is_pic", "false").unwrap();
        let flags = settings::Flags::new(flag_builder);

        let isa = match isa::lookup(Triple::host()) {
            Err(err) => panic!("Error looking up target: {}", err),
            Ok(isa_builder) => isa_builder.finish(flags).unwrap(),
        };
        let pointer_type = isa.pointer_type();

        let builder = JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());

        let module = JITModule::new(builder);
        Self {
            module,
            fn_refs: Vec::new(),
            builder_ctx: FunctionBuilderContext::new(),
            fn_ids: Vec::new(),
            sigs: Vec::new(),
            pointer_type,
            refid: 0,
            varid: 0,
        }
    }
    fn refid(&mut self) -> u32 {
        self.refid += 1;
        self.refid
    }
    pub fn varid(&mut self) -> usize {
        self.varid += 1;
        self.varid
    }

    pub fn create_function(&mut self, params: &[Type], returns: &[Type]) -> (FuncId, Context) {
        let mut sig = Signature::new(CallConv::SystemV);

        for x in params {
            sig.params.push(AbiParam::new(x.to_owned()));
        }
        for x in returns {
            sig.returns.push(AbiParam::new(x.to_owned()));
        }
        let name = self.refid().to_string();
        let id = self
            .module
            .declare_function(&name, Linkage::Export, &sig)
            .unwrap();
        let mut ctx = Context::new();
        ctx.func.signature = sig.clone();

        let func = Function::with_name_signature(UserFuncName::user(0, self.refid), sig);

        ctx.func = func;

        (id, ctx)
    }
}
