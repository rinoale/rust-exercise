fn main() -> Result<(), Box<dyn std::error::Error>> {
    let result = metal_asm::add_u32(&[1, 2, 3, 4], &[10, 20, 30, 40])?;
    assert_eq!(result, [11, 22, 33, 44]);
    println!("Rust -> ARM64 assembly -> Metal compute: {result:?}");
    Ok(())
}
