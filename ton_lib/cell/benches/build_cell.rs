use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ton_lib_cell::build_parse::builder::TonCellBuilder;
use tonlib_core::cell::CellBuilder;

const ITERATIONS_COUNT: usize = 100;

fn cell_build_ton_lib_empty_cell() {
    for _ in 0..ITERATIONS_COUNT {
        let mut builder = TonCellBuilder::new();
        builder.write_ref(TonCellBuilder::new().build().unwrap().into_ref()).unwrap();
        let cell = builder.build().unwrap();
        black_box(cell);
    }
}

fn cell_build_tonlib_core_empty_cell() {
    for _ in 0..ITERATIONS_COUNT {
        let mut builder = CellBuilder::new();
        builder.store_child(CellBuilder::new().build().unwrap()).unwrap();
        let cell = builder.build().unwrap();
        black_box(cell);
    }
}

fn cell_build_ton_lib_data_cell() {
    for _ in 0..ITERATIONS_COUNT {
        let mut builder1 = TonCellBuilder::new();
        builder1.write_bytes([1, 2, 3]).unwrap();

        let mut builder2 = TonCellBuilder::new();
        builder2.write_bytes([10, 20, 30]).unwrap();

        let mut builder3 = TonCellBuilder::new();
        builder3.write_bytes([100, 200, 255]).unwrap();

        let mut builder = TonCellBuilder::new();
        builder.write_ref(builder1.build().unwrap().into_ref()).unwrap();
        builder.write_ref(builder2.build().unwrap().into_ref()).unwrap();
        builder.write_ref(builder3.build().unwrap().into_ref()).unwrap();

        let cell = builder.build().unwrap();
        black_box(cell);
    }
}

fn cell_build_tonlib_core_data_cell() {
    for _ in 0..ITERATIONS_COUNT {
        let mut builder1 = CellBuilder::new();
        builder1.store_slice(&[1, 2, 3]).unwrap();

        let mut builder2 = CellBuilder::new();
        builder2.store_slice(&[10, 20, 30]).unwrap();

        let mut builder3 = CellBuilder::new();
        builder3.store_slice(&[100, 200, 255]).unwrap();

        let mut builder = CellBuilder::new();
        builder.store_child(builder1.build().unwrap()).unwrap();
        builder.store_child(builder2.build().unwrap()).unwrap();
        builder.store_child(builder3.build().unwrap()).unwrap();

        let cell = builder.build().unwrap();
        black_box(cell);
    }
}

fn benchmark_functions(c: &mut Criterion) {
    c.bench_function("cell_build_ton_lib_empty_cell", |b| b.iter(cell_build_ton_lib_empty_cell));
    c.bench_function("cell_build_tonlib_core_empty_cell", |b| b.iter(cell_build_tonlib_core_empty_cell));

    c.bench_function("cell_build_ton_lib_data_cell", |b| b.iter(cell_build_ton_lib_data_cell));
    c.bench_function("cell_build_tonlib_core_data_cell", |b| b.iter(cell_build_tonlib_core_data_cell));
}

criterion_group!(benches, benchmark_functions);
criterion_main!(benches);
