use std::mem;

use crate::parser::ast;
use cranelift::codegen::ir::{UserFuncName, FuncRef, DataFlowGraph};
use cranelift::codegen::packed_option::ReservedValue;
use cranelift::frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift::codegen::{
    ir::{types::I64, AbiParam, Function, Signature},
    isa::CallConv,
};
use cranelift::prelude::{InstBuilder, types, ExtFuncData, MemFlags, EntityRef, Configurable};

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



    let mut module = JITModule::new(builder);
    let mut ctx = module.make_context();

    let mut builder_ctx  = FunctionBuilderContext::new();
    let mut data_ctx = DataContext::new();
    let mut l = DataFlowGraph::new();
  
    /*let mut sig2 = Signature::new(CallConv::SystemV);
    sig2.params.push(AbiParam::new(types::I64));

    sig2.returns.push(AbiParam::new(I64));*/

    ctx.func.signature.params.push(AbiParam::new(I64));

    ctx.func.signature.returns.push(AbiParam::new(I64));
   


    //let mut func2 = Function::with_name_signature(UserFuncName::default(), sig2);
  //  
    let mut builder2 = FunctionBuilder::new(&mut ctx.func, &mut builder_ctx);
//    let pointer_type2 = isa.pointer_type();
    let block2 = builder2.create_block();

    builder2.append_block_params_for_function_params(block2);
    builder2.seal_block(block2);
    builder2.switch_to_block(block2);

    let arg = builder2.block_params(block2)[0];

    let plus_one2 = builder2.ins().iadd_imm(arg, 1);

    let inst = builder2.ins().return_(&[plus_one2]);
    l.append_result(inst, I64);

    builder2.finalize();

    let id = module.declare_function("t", Linkage::Export, &ctx.func.signature).unwrap();

    let m = module.declare_func_in_func(id, &mut ctx.func);
    module.define_function(id, &mut ctx).unwrap();
    module.finalize_definitions().unwrap();

    let code_ad = module.get_finalized_function(id);

    

    println!("d");


    let mut sig = Signature::new(CallConv::SystemV);

  //  ctx.func.signature = Signature::new(CallConv::SystemV);
    sig.params.push(AbiParam::new(I64));
   // ctx.func.signature.params.push(AbiParam::new(I64));

    

    sig.returns.push(AbiParam::new(I64));
    
    
    let mut func = Function::with_name_signature(UserFuncName::default(), sig);
   // let mut func_ctx = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut func, &mut builder_ctx);
    let block = builder.create_block();
    builder.seal_block(block);
    
    builder.append_block_params_for_function_params(block);
    builder.switch_to_block(block);
    let def_sg = builder.import_signature(ctx.func.signature.clone());
    let def_adr = builder.ins().iconst(pointer_type, code_ad as i64);

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
   
    
    println!("d2");
    
   /* let plus_four = builder.ins().call(m, &[plus_three]);
    let pl4 = l.inst_results(plus_four);
    println!("d3");*/
   
    builder.ins().call_indirect(write_sig,write_address,&[plus_three,plus_three,plus_three,plus_three]);
    builder.ins().call_indirect(def_sg,def_adr,&[plus_three]);

    

   // builder.ins().store(mem_flags, cell_value, plus_three, 0);

    builder.ins().return_(&[plus_three]);
    
    
    builder.finalize();
    
    

   /*  let id = module
    .declare_function(&"t1", Linkage::Export, &ctx.func.signature)
    .map_err(|e| e.to_string()).unwrap();
println!("{}", ctx.func);  


    module.define_function(id, &mut ctx).unwrap();


    module.finalize_definitions().unwrap();



    let code = module.get_finalized_function(id);*/
    println!("d4");



    println!("{}", func);  


    

    
 

   /*  let mut buffer = memmap2::MmapOptions::new()
        .len(code.len())
        .map_anon()
        .unwrap();

    buffer.copy_from_slice(code);*/

    //let buffer = buffer.make_exec().unwrap();

   /*  let x = unsafe {
        let code_fn: unsafe extern "sysv64" fn(usize) -> usize =  mem::transmute::<_, unsafe extern "sysv64" fn(usize) -> usize>(code);

        code_fn(55)
    };

    println!("out: {}", x);*/

    
}

struct JIT{
    builder: settings::Builder,


}
impl JIT{
    pub fn new()->Self{
        todo!()
    }
}