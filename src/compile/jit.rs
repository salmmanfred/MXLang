use std::mem;

use crate::parser::ast;
use cranelift::codegen::ir::{UserFuncName, FuncRef, DataFlowGraph};
use cranelift::codegen::packed_option::ReservedValue;
use cranelift::frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift::codegen::{
    ir::{types::I64, AbiParam, Function, Signature},
    isa::CallConv,
};
use cranelift::prelude::{InstBuilder, types, ExtFuncData, MemFlags, EntityRef, Configurable, Type};

use cranelift::codegen::{isa, settings, Context};
use cranelift_jit::{JITModule, JITBuilder};
use target_lexicon::Triple;
use cranelift_module::{DataContext, Linkage, Module, FuncId};
use core::fmt::write;

extern "sysv64" fn pr(t: i64,d:i64){
    
        println!("{t} {d}");
    //println!("{:?}",*t);
    
}
extern "sysv64" fn pr2(t: i64,d:i64){
    
    println!("22:::: {t} {d}");
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
    println!("d0");


    let mut module = JITModule::new(builder);
    let mut ctx = module.make_context();

    let mut builder_ctx  = FunctionBuilderContext::new();
    let mut data_ctx = DataContext::new();
    
  
    let mut sig2 = Signature::new(CallConv::SystemV);
    sig2.params.push(AbiParam::new(types::I64));
    sig2.params.push(AbiParam::new(types::I64));


    sig2.returns.push(AbiParam::new(I64));
    

    
    let id2 = module.declare_function("1:1", Linkage::Export, &sig2).unwrap();
    
    ctx.func.signature = sig2;

    let mut func2 = Function::with_name_signature(UserFuncName::user(1,1), ctx.func.signature.clone());
  //  


    
    let mut builder2 = FunctionBuilder::new(&mut func2, &mut builder_ctx);
//    let pointer_type2 = isa.pointer_type();
    let block2 = builder2.create_block();
    builder2.append_block_params_for_function_params(block2);

    builder2.switch_to_block(block2);
    builder2.seal_block(block2);


    let (write_sig, write_address) = {
        let mut write_sig = Signature::new(CallConv::SystemV);
        write_sig.params.push(AbiParam::new(types::I64));
        write_sig.params.push(AbiParam::new(types::I64));



       // write_sig.returns.push(AbiParam::new(pointer_type));
        let write_sig = builder2.import_signature(write_sig);
    
        let write_address = pr2 as *const () as i64;
        let write_address = builder2.ins().iconst(pointer_type, write_address);
        (write_sig, write_address)
    };

    let arg = builder2.block_params(block2)[0];

    let plus_one2 = builder2.ins().iadd_imm(arg, 1);
    builder2.ins().call_indirect(write_sig,write_address,&[plus_one2,plus_one2]);

    builder2.ins().return_(&[plus_one2]);
    
    builder2.finalize();

    println!("{}", func2);  
    ctx.func = func2.clone();
    module.define_function(id2, &mut ctx).unwrap();
    


    
   
    println!("d0.5");

    //module.define_function(id, &mut ctx).unwrap();

    //module.finalize_definitions().unwrap();

    //let code_ad = module.get_finalized_function(id);

    

    println!("d");


    let mut sig = Signature::new(CallConv::SystemV);

  //  ctx.func.signature = Signature::new(CallConv::SystemV);
    sig.params.push(AbiParam::new(I64));
    ctx.func.signature.params.push(AbiParam::new(I64));

    

    sig.returns.push(AbiParam::new(I64));
    
    let id = module.declare_function("d", Linkage::Export, &sig).unwrap();
    ctx.func.signature = sig;
    let mut func = Function::with_name_signature(UserFuncName::default(), ctx.func.signature.clone());
   // let mut func_ctx = FunctionBuilderContext::new();
    let func2ref = module.declare_func_in_func(id2, &mut func);
   
    let mut builder = FunctionBuilder::new(&mut func, &mut builder_ctx);
    let block = builder.create_block();
    builder.seal_block(block);
    
    builder.append_block_params_for_function_params(block);
    builder.switch_to_block(block);
    //281389207782307
   /* let def_sg = builder.import_signature(ctx.func.signature.clone());
    let def_adr = builder.ins().iconst(pointer_type, code_ad as i64);*/

    let (write_sig, write_address) = {
        let mut write_sig = Signature::new(CallConv::SystemV);
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
    builder.ins().call_indirect(write_sig,write_address,&[plus_three,arg]);

    let a = builder.ins().call(func2ref, &[plus_three,plus_two]);
    func2.dfg.append_result(a, I64);
    func2.dfg.append_result(a, I64);


    println!("{:#?}",func2.dfg.has_results(a));

    let returna = func2.dfg.inst_results(a);


    builder.ins().call_indirect(write_sig,write_address,&[returna[0],returna[1]]);

    println!("d3");

   // builder.ins().call_indirect(def_sg,def_adr,&[plus_three]);

    

   // builder.ins().store(mem_flags, cell_value, plus_three, 0);

    builder.ins().return_(&[plus_three]);
    
    
    builder.finalize();
    println!("{}", func2);  
    
    println!("{}", func);  
    
    
    println!("d3.5");
    ctx.func = func;
    
    module.define_function(id, &mut ctx).unwrap();
    

   


    module.finalize_definitions().unwrap();

    

    let code = module.get_finalized_function(id);
    println!("d4");

    unsafe {
        let code_fn:unsafe extern "sysv64" fn(i64) ->i64 =  mem::transmute::<_, unsafe extern "sysv64" fn(i64)->i64>(code);
        // unsafe extern "sysv64" fn(usize) ->usize 
        let f = code_fn(55);
        println!("{f}");
    };



    

    
 

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

pub fn test_compile2(){
    let mut l = JIT::new();
    let func = l.create_function(&[I64], &[I64]);
    //l.module.declare_func_in_func(func_id, func)


}

struct JIT{
    module: JITModule,
    fn_refs: Vec<FuncRef>,
    pub builder_ctx: FunctionBuilderContext,
    fn_ids: Vec<FuncId>,
    sigs: Vec<Signature>,
    pointer_type: Type,
    refid: u32,
   
}
impl JIT{
    pub fn new()->Self{
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
        println!("d0");


        let mut module = JITModule::new(builder);
        Self{
            module,
            fn_refs: Vec::new(),
            builder_ctx: FunctionBuilderContext::new(),
            fn_ids: Vec::new(),
            sigs: Vec::new(),
            pointer_type,
            refid: 0,
           
            
        }
    }
    fn refid(&mut self)->u32{
        self.refid += 1;
        self.refid

    }

    pub fn create_function(&mut self, params: &[Type], returns: &[Type] )-> (FuncId, Function){
        let mut sig = Signature::new(CallConv::SystemV);



        for x in params{
            sig.params.push(AbiParam::new(x.to_owned()));
        }
        for x in returns{
            sig.returns.push(AbiParam::new(x.to_owned()));
        }
        let name = self.refid().to_string();
        let id = self.module.declare_function(&name, Linkage::Export, &sig).unwrap();

        let func = Function::with_name_signature(UserFuncName::user(0,self.refid), sig);
        
        



        (id,func)
    }   


}