use ancvm_runtime::instruction::Opcode;

fn main() {
    assert_eq!(1, Opcode::drop as u16);
}
