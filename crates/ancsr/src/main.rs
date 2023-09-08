fn main() {
    println!("Hello");
}

#[cfg(test)]
mod tests {
    use ancvm_types::opcode::Opcode;

    #[test]
    fn test_test() {
        assert_eq!(Opcode::drop as u16, 0x101);
    }
}
