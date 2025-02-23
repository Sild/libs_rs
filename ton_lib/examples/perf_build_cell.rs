use ton_lib::cell_build_parse::cell_builder::TonCellBuilder;

fn main() -> anyhow::Result<()> {
    for _ in 0..10000000 {
        let mut builder1 = TonCellBuilder::new();
        builder1.write_bit(true)?;
        builder1.write_bytes([1, 2, 3])?;
        builder1.write_num(4, 4)?;

        let mut builder2 = TonCellBuilder::new();
        builder2.write_bytes([10, 20, 30])?;

        let mut builder3 = TonCellBuilder::new();
        builder3.write_bytes([100, 200, 255])?;

        let mut builder = TonCellBuilder::new();
        builder.write_ref(builder1.build()?)?;
        builder.write_ref(builder2.build()?)?;
        builder.write_ref(builder3.build()?)?;

        let cell = builder.build()?;
        // println!("{cell}");
    }
    Ok(())
}
