use ancvm_runtime::opcode::Opcode;

fn main() {
    assert_eq!(1, Opcode::drop as u16);
}
