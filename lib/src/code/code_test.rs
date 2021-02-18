use super::*;

#[test]
fn test_make() {
    let tests: Vec<(Opcode, Vec<usize>, Vec<u8>)> = vec![(
        Opcode::Constant,
        vec![65534],
        vec![Opcode::Constant as u8, 255, 254],
    )];

    for test in tests {
        let instruction = make(test.0, test.1);
        assert_eq!(instruction.len(), test.2.len());

        for (expected, actual) in test.2.iter().zip(instruction) {
            assert_eq!(expected, &actual);
        }
    }
}

#[test]
fn test_display() {
    let instructions: Vec<Instructions> = vec![
        make(Opcode::Constant, vec![1]),
        make(Opcode::Constant, vec![2]),
        make(Opcode::Constant, vec![65535]),
    ];

    let expected = "0000 OpConstant 1
0003 OpConstant 2
0006 OpConstant 65535
";

    let concatted = instructions.concat();

    assert_eq!(format!("{}", pretty(concatted)), expected);
}
