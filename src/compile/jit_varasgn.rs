use std::collections::HashMap;

use cranelift::{
    codegen::ir::FuncRef,
    prelude::{types::I64, FunctionBuilder, Variable},
};

use crate::parser::ast::{self, Node};

use cranelift::prelude::{EntityRef, InstBuilder, Type, Value};

use cranelift_module::Module;

pub fn op_asgn(
    op: ast::Op,
    name: String,
    asgn: Box<Node>,
    variables: &mut HashMap<String, Variable>,
    funcs: &mut HashMap<String, FuncRef>,
    builder: &mut FunctionBuilder,
    module: &mut super::jit::JIT,
    memfun: &super::jit::IntFunction,
) {
    match op {
        ast::Op::Equall => {
            op_asgn_eq(name, asgn, variables, funcs, builder, module, memfun);
        }
        ast::Op::Plus => {
            if variables.contains_key(&name) {
                println!(" asg {:#?}",asgn);
                match *asgn{
                    Node::Var(_) | Node::Int(_) =>{
                        let v = builder.use_var(*variables.get(&name).unwrap());

                        let v2 = asgn.unwrap_value(variables, builder);
                        let num_plus_v = builder.ins().iadd(v, v2);
                        builder.def_var(*variables.get(&name).unwrap(), num_plus_v);
                    }
                    _=>{
                        panic!("Add plus for arrays")
                    }
                }
               

            }
        }
        _ => {
            todo!()
        }
    }
}

pub fn op_asgn_eq(
    name: String,
    asgn: Box<Node>,
    variables: &mut HashMap<String, Variable>,
    funcs: &mut HashMap<String, FuncRef>,
    builder: &mut FunctionBuilder,
    module: &mut super::jit::JIT,
    memfun: &super::jit::IntFunction,
) {
    let name_alt = name.clone();

    match *asgn {
        //TODO: better way of doing this?
        Node::Int(a) => {
            let var = Variable::new(module.varid());

            builder.declare_var(var, module.pointer_type);

            let num = builder.ins().iconst(I64, a);
            builder.def_var(var, num);
            variables.insert(name, var);
        }
        Node::Array(a) => {
            // An idea is to add before len an address to a continuation of the array
            //|addr|len|obj1|obj2|..|objn|
            //          ^^^ p still points here but when i becomes bigger than len(-8) or equall it looks if addr (-16)
            // points anywhere if it does it jumps there and continues this then has the same layout
            // benifits: no need to realloc the entire thing if the current position in memory cannot support it
            // negatives: slower

            //|len|obj1|obj2|...
            //     ^^^ this is wha the pointer points to (trust me)
            // this might be a handicap later but dont worry about it

            let varval = builder.ins().iconst(I64, a.len() as i64 + 8);
            println!("{}", a.len() as i64 as usize);
            let num = builder
                .ins()
                .call_indirect(memfun.malloc_sig, memfun.malloc_addr, &[varval]);
            let num = builder.func.dfg.inst_results(num)[0];
            // Store the size
            let addr_size = builder.ins().iadd_imm(num, -8);
            builder
                .ins()
                .call_indirect(memfun.poke_sig, memfun.poke_addr, &[addr_size, varval]);
            let mut addr = num;
            //TODO: this looks stupid in code any better way of doing it?
            for x in 0..a.len() {
                let varval = a[x].unwrap_value(variables, builder);

                builder
                    .ins()
                    .call_indirect(memfun.poke_sig, memfun.poke_addr, &[addr, varval]);
                addr = builder.ins().iadd_imm(addr, 8);
                println!("e");
            }

            let var = Variable::new(module.varid());

            builder.declare_var(var, module.pointer_type);

            builder.def_var(var, num);
            variables.insert(name.clone(), var);
        }

        Node::GetArray(a, b) => {
            //TODO: Add checking if the memory pointer is outside of the array lenght

            let var = variables.get(&a).unwrap();
            let varval = builder.use_var(*var);
            let val = b.unwrap_value(variables, builder);
            let val = builder.ins().imul_imm(val, 8);
            let addr = builder.ins().iadd(varval, val);

            let num = builder
                .ins()
                .call_indirect(memfun.peek_sig, memfun.peek_addr, &[addr]);
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
                    Node::Int(_) | Node::Var(_) => arg_typ.push(I64),
                    _ => {}
                }
            }
            let (funcid2, mut func2ctx) = module.create_function(&arg_typ, &[I64]);

            super::jit::function_compiler_handler(
                args,
                insides,
                module,
                &mut func2ctx,
                funcid2,
                true,
            );

            let funcref = module.module.declare_func_in_func(funcid2, builder.func);
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
