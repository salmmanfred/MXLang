use std::mem;

use crate::parser::ast;
use cranelift::codegen::ir::UserFuncName;
use cranelift::frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift::codegen::{
    ir::{types::I64, AbiParam, Function, Signature},
    isa::CallConv,
};
use cranelift::prelude::{InstBuilder, types, ExtFuncData};

use cranelift::codegen::{isa, settings, Context};
use cranelift_jit::JITModule;
use target_lexicon::Triple;
use cranelift_module::{DataContext, Linkage, Module};


fn pr(t: i64){
    println!("{t}");
}

pub fn test_compile(){
    let builder = settings::builder();
    let flags = settings::Flags::new(builder);

    let isa = match isa::lookup(Triple::host()) {
        Err(err) => panic!("Error looking up target: {}", err),
        Ok(isa_builder) => isa_builder.finish(flags).unwrap(),
    };

    let modu = JITModule::new(builder);
    modu
    .declare_function(&"pr".to_string(), Linkage::Export, &self.ctx.func.signature)
    .map_err(|e| e.to_string())?;


    let mut sig = Signature::new(CallConv::SystemV);
    sig.params.push(AbiParam::new(I64));
    sig.params.push(AbiParam::new(I64));

    sig.returns.push(AbiParam::new(I64));
    
    let mut func = Function::with_name_signature(UserFuncName::default(), sig);
    let mut func_ctx = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut func, &mut func_ctx);
    

    let block = builder.create_block();
    builder.seal_block(block);
    
    builder.append_block_params_for_function_params(block);
    builder.switch_to_block(block);
    
    
    

    
    let arg = builder.block_params(block)[0];
    let arg2 = builder.block_params(block)[1];

    let plus_one = builder.ins().iadd_imm(arg, 1);
    let plus_two = builder.ins().iadd_imm(arg, 2);
    
    let plus_three = builder.ins().iadd(plus_one,plus_two);

    builder.ins().return_(&[plus_three]);
    
    builder.finalize();
    println!("{}", func.display());  


   

    
    let mut ctx = Context::for_function(func);
    let code = ctx.compile(&*isa).unwrap();

    let mut buffer = memmap2::MmapOptions::new()
        .len(26)
        .map_anon()
        .unwrap();

    buffer.copy_from_slice(code.code_buffer());

    let buffer = buffer.make_exec().unwrap();

    let x = unsafe {
        let code_fn: unsafe extern "sysv64" fn(usize) -> usize =
            std::mem::transmute(buffer.as_ptr());

        code_fn(0)
    };

    println!("out: {}", x);

    
}