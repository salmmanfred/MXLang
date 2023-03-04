use std::mem;

use crate::parser::ast;
use cranelift::codegen::ir::{UserFuncName, FuncRef};
use cranelift::frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift::codegen::{
    ir::{types::I64, AbiParam, Function, Signature},
    isa::CallConv,
};
use cranelift::prelude::{InstBuilder, types, ExtFuncData, MemFlags, EntityRef};

use cranelift::codegen::{isa, settings, Context};
use cranelift_jit::{JITModule, JITBuilder};
use target_lexicon::Triple;
use cranelift_module::{DataContext, Linkage, Module};
use core::fmt::write;

extern "sysv64" fn pr(t: i64,b:i64,c:i64,d:i64){
    
        println!("{t},{},{},{}",b,c,d);
    //println!("{:?}",*t);
    
}

pub fn test_compile(){
    
    let mut builder = settings::builder();
    let flags = settings::Flags::new(builder);

    let isa = match isa::lookup(Triple::host()) {
        Err(err) => panic!("Error looking up target: {}", err),
        Ok(isa_builder) => isa_builder.finish(flags).unwrap(),
    };
    


    let mut sig = Signature::new(CallConv::SystemV);
    sig.params.push(AbiParam::new(I64));
    

    sig.returns.push(AbiParam::new(I64));
    
    
    let mut func = Function::with_name_signature(UserFuncName::default(), sig);
    let mut func_ctx = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut func, &mut func_ctx);
    let pointer_type = isa.pointer_type();
    let block = builder.create_block();
    builder.seal_block(block);
    
    builder.append_block_params_for_function_params(block);
    builder.switch_to_block(block);
    

    let (write_sig, write_address) = {
        let mut write_sig = Signature::new(CallConv::SystemV);
        write_sig.params.push(AbiParam::new(types::I64));
        write_sig.params.push(AbiParam::new(types::I64));
        write_sig.params.push(AbiParam::new(types::I64));
        write_sig.params.push(AbiParam::new(types::I64));


       // write_sig.returns.push(AbiParam::new(pointer_type));
        let write_sig = builder.import_signature(write_sig);
    
        let write_address = pr as *const () as i64;
        let write_address = builder.ins().iconst(pointer_type, write_address);
        (write_sig, write_address)
    };

   
    
    

    

    
    let arg = builder.block_params(block)[0];
    

    let plus_one = builder.ins().iadd_imm(arg, 1);
    let plus_two = builder.ins().iadd_imm(arg, 2);
    
    
    let plus_three = builder.ins().iadd(plus_one,plus_two);
   

    
    
   
    builder.ins().call_indirect(write_sig,write_address,&[plus_three,plus_three,plus_three,plus_three]);
   // builder.ins().store(mem_flags, cell_value, plus_three, 0);

    builder.ins().return_(&[plus_three]);
    
    builder.finalize();
    println!("{}", func.display());  


   

    
    let mut ctx = Context::for_function(func);
    let code = ctx.compile(&*isa).unwrap();

    let mut buffer = memmap2::MmapOptions::new()
        .len(code.buffer.data().len())
        .map_anon()
        .unwrap();

    buffer.copy_from_slice(code.code_buffer());

    let buffer = buffer.make_exec().unwrap();

    let x = unsafe {
        let code_fn: unsafe extern "sysv64" fn(usize) -> usize =
            std::mem::transmute(buffer.as_ptr());

        code_fn(55)
    };

    println!("out: {}", x);

    
}