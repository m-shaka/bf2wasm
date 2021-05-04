use std::fs::File;
use std::io::{BufRead, BufReader};

const TOKENS: &str = "><+-.,[]";

fn parse_to_char(reader: &mut BufReader<File>) -> Vec<char> {
    let mut res: Vec<char> = vec![];

    for line in reader.lines() {
        for c in line.unwrap().chars() {
            if TOKENS.contains(c) {
                res.push(c)
            }
        }
    }
    res
}

#[derive(Debug, PartialEq)]
pub enum BfOpKind {
    IncPtr,
    DecPtr,
    IncData,
    DecData,
    ReadStdin,
    WriteStdout,
    LoopSetToZero,
    LoopMovePtr,
    LoopMoveData,
    JumpIfDataZero,
    JumpIfDataNotZero,
}

#[derive(Debug)]
pub struct BfOp {
    pub kind: BfOpKind,
    pub argument: i32,
}

fn translate(insts: &[char]) -> Vec<BfOp> {
    let mut res: Vec<BfOp> = vec![];
    let mut loop_stack: Vec<usize> = vec![];
    let mut pc: usize = 0;
    let program_size = insts.len();
    while pc < program_size {
        let inst = insts[pc];
        match inst {
            '[' => {
                loop_stack.push(res.len());
                res.push(BfOp {
                    kind: BfOpKind::JumpIfDataZero,
                    argument: 0,
                });
                pc += 1;
            }
            ']' => {
                let offset = loop_stack
                    .pop()
                    .expect(&format!("unmatched closing ']' at pc={}", pc));
                let optimized_ops = optimize_loop(&res, offset);
                if optimized_ops.len() == 0 {
                    res[offset].argument = res.len() as i32;
                    res.push(BfOp {
                        kind: BfOpKind::JumpIfDataNotZero,
                        argument: offset as i32,
                    })
                } else {
                    res.splice(offset.., optimized_ops);
                }
                pc += 1;
            }
            _ => {
                let num_repeats = insts[pc..insts.len()]
                    .iter()
                    .take_while(|&&c| c == inst)
                    .count();
                pc += num_repeats;
                let kind = match inst {
                    '>' => BfOpKind::IncPtr,
                    '<' => BfOpKind::DecPtr,
                    '+' => BfOpKind::IncData,
                    '-' => BfOpKind::DecData,
                    ',' => BfOpKind::ReadStdin,
                    '.' => BfOpKind::WriteStdout,
                    _ => panic!("Invalid token"),
                };
                res.push(BfOp {
                    kind,
                    argument: num_repeats as i32,
                })
            }
        }
    }
    res
}

fn optimize_loop(ops: &[BfOp], loop_start: usize) -> Vec<BfOp> {
    let mut res: Vec<BfOp> = vec![];
    let loop_size = ops.len() - loop_start;
    match loop_size {
        2 => {
            let repeated_op = &ops[loop_start + 1];
            match repeated_op.kind {
                BfOpKind::IncData | BfOpKind::DecData => res.push(BfOp {
                    kind: BfOpKind::LoopSetToZero,
                    argument: 0,
                }),
                BfOpKind::IncPtr => res.push(BfOp {
                    kind: BfOpKind::LoopMovePtr,
                    argument: repeated_op.argument,
                }),
                BfOpKind::DecPtr => res.push(BfOp {
                    kind: BfOpKind::LoopMovePtr,
                    argument: -repeated_op.argument,
                }),

                _ => {}
            }
        }
        5 => {
            if ops[loop_start + 1].kind == BfOpKind::DecData
                && ops[loop_start + 3].kind == BfOpKind::IncData
                && ops[loop_start + 1].argument == 1
                && ops[loop_start + 3].argument == 1
            {
                match (&ops[loop_start + 2], &ops[loop_start + 4]) {
                    (
                        BfOp {
                            kind: BfOpKind::IncPtr,
                            argument: a1,
                        },
                        BfOp {
                            kind: BfOpKind::DecPtr,
                            argument: a2,
                        },
                    ) if a1 == a2 => res.push(BfOp {
                        kind: BfOpKind::LoopMoveData,
                        argument: *a1,
                    }),
                    (
                        BfOp {
                            kind: BfOpKind::DecPtr,
                            argument: a1,
                        },
                        BfOp {
                            kind: BfOpKind::IncPtr,
                            argument: a2,
                        },
                    ) if a1 == a2 => res.push(BfOp {
                        kind: BfOpKind::LoopMoveData,
                        argument: -*a1,
                    }),
                    _ => {}
                }
            }
        }
        _ => {}
    }
    res
}

pub fn parse(reader: &mut BufReader<File>) -> Vec<BfOp> {
    let insts = parse_to_char(reader);
    translate(&insts)
}
