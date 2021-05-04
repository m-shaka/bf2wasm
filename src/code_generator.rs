use super::bf_ops::{BfOp, BfOpKind};

struct LabelStack {
    index: u32,
    stack: Vec<u32>,
}

impl LabelStack {
    fn new() -> Self {
        LabelStack {
            index: 0,
            stack: vec![],
        }
    }

    fn push(&mut self) -> u32 {
        let current = self.index;
        self.stack.push(current);
        self.index += 1;
        current
    }

    fn pop(&mut self) -> u32 {
        self.stack.pop().unwrap()
    }
}

fn generate_main(ops: &[BfOp]) -> Vec<String> {
    let get_index = "global.get $index";
    let set_index = "global.set $index";
    let load = "i32.load8_u";
    let store = "i32.store8";
    let add = "i32.add";
    let sub = "i32.sub";

    let mut label_stack = LabelStack::new();
    let mut wat_insts: Vec<String> = vec!["(local $mvDataTemp i32)".to_string()];
    for op in ops {
        match op.kind {
            BfOpKind::IncData => {
                let mut insts: Vec<String> = vec![
                    get_index.to_string(),
                    get_index.to_string(),
                    load.to_string(),
                    format!("i32.const {}", op.argument),
                    add.to_string(),
                    store.to_string(),
                ];
                wat_insts.append(&mut insts);
            }
            BfOpKind::DecData => {
                let mut insts: Vec<String> = vec![
                    get_index.to_string(),
                    get_index.to_string(),
                    load.to_string(),
                    format!("i32.const {}", op.argument),
                    sub.to_string(),
                    store.to_string(),
                ];
                wat_insts.append(&mut insts);
            }
            BfOpKind::IncPtr => {
                let mut insts: Vec<String> = vec![
                    get_index.to_string(),
                    format!("i32.const {}", op.argument),
                    add.to_string(),
                    set_index.to_string(),
                ];
                wat_insts.append(&mut insts)
            }
            BfOpKind::DecPtr => {
                let mut insts: Vec<String> = vec![
                    get_index.to_string(),
                    format!("i32.const {}", op.argument),
                    sub.to_string(),
                    set_index.to_string(),
                ];
                wat_insts.append(&mut insts)
            }
            BfOpKind::JumpIfDataZero => {
                let current = label_stack.push();
                let mut insts = vec![
                    format!("loop $loop_{}", current),
                    get_index.to_string(),
                    load.to_string(),
                    "if".to_string(),
                ];
                wat_insts.append(&mut insts);
            }
            BfOpKind::JumpIfDataNotZero => {
                let current = label_stack.pop();
                let mut insts = vec![
                    get_index.to_string(),
                    load.to_string(),
                    "if".to_string(),
                    format!("br $loop_{}", current),
                    "end".to_string(),
                    "end".to_string(),
                    "end".to_string(),
                ];
                wat_insts.append(&mut insts);
            }
            BfOpKind::LoopSetToZero => {
                let mut insts = vec![
                    get_index.to_string(),
                    "i32.const 0".to_string(),
                    store.to_string(),
                ];
                wat_insts.append(&mut insts);
            }
            BfOpKind::LoopMovePtr => {
                let current = label_stack.push();
                let mut insts = vec![
                    format!("loop $loop_{}", current),
                    get_index.to_string(),
                    load.to_string(),
                    "if".to_string(),
                    get_index.to_string(),
                    format!("i32.const {}", op.argument),
                    add.to_string(),
                    set_index.to_string(),
                    format!("br $loop_{}", label_stack.pop()),
                    "end".to_string(),
                    "end".to_string(),
                ];
                wat_insts.append(&mut insts);
            }
            BfOpKind::LoopMoveData => {
                let mut insts = vec![
                    get_index.to_string(),
                    load.to_string(),
                    "if".to_string(),
                    get_index.to_string(),
                    format!("i32.const {}", op.argument),
                    add.to_string(),
                    "local.set $mvDataTemp".to_string(),
                    "local.get $mvDataTemp".to_string(),
                    get_index.to_string(),
                    load.to_string(),
                    "local.get $mvDataTemp".to_string(),
                    load.to_string(),
                    add.to_string(),
                    store.to_string(),
                    get_index.to_string(),
                    "i32.const 0".to_string(),
                    store.to_string(),
                    "end".to_string(),
                ];
                wat_insts.append(&mut insts);
            }
            BfOpKind::ReadStdin => {
                for _ in 0..op.argument {
                    wat_insts.push("call $read_one_byte".to_string())
                }
            }
            BfOpKind::WriteStdout => {
                for _ in 0..op.argument {
                    wat_insts.push("call $write_one_byte".to_string())
                }
            }
        }
    }
    wat_insts
}

pub fn generate_wat(ops: &[BfOp]) -> String {
    let main_body = generate_main(ops);
    let wat = r#"
(module
    (import "wasi_unstable" "fd_read" (func $fd_read (param i32 i32 i32 i32) (result i32)))

    (import "wasi_unstable" "fd_write" (func $fd_write (param i32 i32 i32 i32) (result i32)))

    (memory $mem 1)
    (export "memory" (memory $mem))
    (global $index (mut i32) (i32.const 8))

    (func $write_one_byte
        i32.const 0
        global.get $index
        i32.store8
        (i32.store8 (i32.const 4) (i32.const 1))
        (call $fd_write
            (i32.const 1)
            (i32.const 0)
            (i32.const 1)
            (i32.const 30012)
        )
        drop
    )

    (func $read_one_byte
        i32.const 0
        global.get $index
        i32.store8
        (i32.store8 (i32.const 4) (i32.const 1))
        (call $fd_read
            (i32.const 0)
            (i32.const 0)
            (i32.const 1)
            (i32.const 30012)
        )
        drop
    )

    (func $main (export "_start")
"#
    .to_string();
    let indent = " ".repeat(8);
    format!(
        "{}{}{}\n    )\n)\n",
        wat,
        indent,
        main_body.join("\n        ")
    )
}
