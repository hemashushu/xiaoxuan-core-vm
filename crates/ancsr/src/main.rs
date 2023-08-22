fn main() {
    println!("Hello");
}

#[cfg(test)]
mod tests {
    use ancvm_types::opcode::Opcode;

    #[test]
    fn test_test() {
        assert_eq!(1, Opcode::drop as u16);
    }
}
