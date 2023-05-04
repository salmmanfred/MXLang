use std::alloc::{alloc, Layout};
use std::collections::HashMap;
use std::mem;

use crate::parser::ast::{self, Node};
use cranelift::codegen::ir::{FuncRef, SigRef, UserFuncName};
use cranelift::codegen::{
    ir::{types::I64, AbiParam, Function, Signature},
    isa::CallConv,
};
use cranelift::frontend::{FunctionBuilder, FunctionBuilderContext};

use cranelift::prelude::{types, Configurable, EntityRef, InstBuilder, Type, Value, Variable};

use super::jit_varasgn;
use cranelift::codegen::{isa, settings, Context};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{FuncId, Linkage, Module};
use target_lexicon::Triple;


#[derive(Debug,Copy,Clone)]
pub enum Types{
    Int,
    Array,
}
pub struct IntFunction {
    pub malloc_sig: SigRef,
    pub malloc_addr: Value,

    pub poke_sig: SigRef,
    pub poke_addr: Value,

    pub peek_sig: SigRef,
    pub peek_addr: Value,
}
impl IntFunction {
    pub fn new(builder: &mut FunctionBuilder, pointer: Type) -> Self {
        //TODO: This also in a better way
        let (malloc_sig, malloc_addr) = {
            let mut malloc_sig = Signature::new(CallConv::SystemV);
            malloc_sig.params.push(AbiParam::new(types::I64));

            malloc_sig.returns.push(AbiParam::new(I64));
            let write_sig = builder.import_signature(malloc_sig);

            let malloc_addr = malloc as *const () as i64;
            let malloc_addr = builder.ins().iconst(pointer, malloc_addr);
            (write_sig, malloc_addr)
        };
        let (poke_sig, poke_addr) = {
            // ! when chaning these dont forget to change the function
            let mut malloc_sig = Signature::new(CallConv::SystemV);
            malloc_sig.params.push(AbiParam::new(types::I64));
            malloc_sig.params.push(AbiParam::new(types::I64));

            let write_sig = builder.import_signature(malloc_sig);

            let malloc_addr = poke as *const () as i64;
            let malloc_addr = builder.ins().iconst(pointer, malloc_addr);
            (write_sig, malloc_addr)
        };
        let (peek_sig, peek_addr) = {
            // ! when chaning these dont forget to change the function
            let mut malloc_sig = Signature::new(CallConv::SystemV);
            malloc_sig.params.push(AbiParam::new(types::I64));

            malloc_sig.returns.push(AbiParam::new(I64));
            let write_sig = builder.import_signature(malloc_sig);

            let malloc_addr = peek as *const () as i64;
            let malloc_addr = builder.ins().iconst(pointer, malloc_addr);
            (write_sig, malloc_addr)
        };
        Self {
            malloc_sig,
            malloc_addr,
            poke_sig,
            poke_addr,
            peek_sig,
            peek_addr,
        }
    }
}

//TODO: do it in a better way
unsafe extern "sysv64" fn malloc(size: i64) -> i64 {
    let layout = Layout::from_size_align(8 * size as usize, 2).unwrap();

    let ptr_raw = alloc(layout);
    if ptr_raw.is_null() {
        panic!("Failed to allocate memory size: {size}");
    }
    let p = ptr_raw as i64 + 8;

    println!("ptrraw {:#?}, P: {}", ptr_raw, p);
    p
}
unsafe extern "sysv64" fn poke(addr: i64, data: i64) {
    *(addr as *mut i64) = data;
}
unsafe extern "sysv64" fn peek(addr: i64) -> i64 {
    *(addr as *mut i64)
}

fn compile_code(
    code: Vec<Node>,
    module: &mut JIT,
    builder: &mut FunctionBuilder,
    variables: &mut HashMap<String, (Variable,Types)>,
    funcs: &mut HashMap<String, FuncRef>,
    arrays: &mut HashMap<String, i64>,
) {
    let memfun = IntFunction::new(builder, module.pointer_type);

    for x in code {
        match x.clone() {
            // !IMPORANT TODO
            //TODO: Send type information with the variables hashmap

            Node::Varasgn { op, name, asgn } => {
                
                jit_varasgn::op_asgn(op, name, asgn, variables, funcs, builder, module, &memfun)
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

                        compile_code(insides, module, builder, variables, funcs, arrays);

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

pub fn function_compiler_handler(
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

    let mut variables: HashMap<String, (Variable,Types)> = HashMap::new();
    let mut funcs: HashMap<String, FuncRef> = HashMap::new();
    let mut arrays: HashMap<String, i64> = HashMap::new();

    for (i, arg) in args.iter().enumerate() {
        if let Node::Var(name) = arg {
            let var = Variable::new(module.varid());
            let argin = builder.block_params(entryblock)[i];
            builder.declare_var(var, module.pointer_type);

            let num = argin;
            builder.def_var(var, num);
            //TODO: determen type better
            variables.insert(name.to_string(), (var,Types::Int));
        }
    }

    compile_code(
        code,
        module,
        &mut builder,
        &mut variables,
        &mut funcs,
        &mut arrays,
    );
    //println!("vars: {:#?}",variables);
    if TEMPARG {
        let v = builder.use_var(variables.get("y").unwrap().0);
        let v2 = builder.use_var(variables.get("arr").unwrap().0);

        builder.ins().return_(&[v2, v]);
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
    let (_mainfuncid, mut ctx) = l.create_function(&[], &[I64, I64]);
    //l.module.declare_func_in_func(func_id, func)

    function_compiler_handler(Vec::new(), code_snip, &mut l, &mut ctx, _mainfuncid, true);

    l.module.finalize_definitions().unwrap();

    let code = l.module.get_finalized_function(_mainfuncid);

    unsafe {
        let code_fn: unsafe extern "sysv64" fn() -> (i64, i64) =
            mem::transmute::<_, unsafe extern "sysv64" fn() -> (i64, i64)>(code);
        // unsafe extern "sysv64" fn(usize) ->usize
        let (f, a) = code_fn();
        drop(code_fn);
        // let ptraddr1: *mut  i64 =  f;
        //let ptr3 = std::ptr::read(f);

        let ptr2 = f as *const i64;
        let ptr3 = std::ptr::read(ptr2);
        for x in 0..5 {
            let ptr2 = (f + 8 * x) as *const i64;
            let ptr33 = std::ptr::read(ptr2);
            println!("index {x}: {ptr33}");
        }
        let ptr2 = (f + 8) as *const i64;
        let ptr33 = std::ptr::read(ptr2);
        println!(
            " const i64: {:#?}, ret: {:#?}, data: {:#?}, data+8: {}",
            ptr2, f, ptr3, ptr33
        );

        println!("{a}");
    };
}

pub struct JIT {
    pub module: JITModule,
    _fn_refs: Vec<FuncRef>,
    pub builder_ctx: FunctionBuilderContext,
    _fn_ids: Vec<FuncId>,
    _sigs: Vec<Signature>,
    pub pointer_type: Type,
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
            _fn_refs: Vec::new(),
            builder_ctx: FunctionBuilderContext::new(),
            _fn_ids: Vec::new(),
            _sigs: Vec::new(),
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

