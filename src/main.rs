use std::f64::consts::PI;

use futuredsp::windows;
use futuresdr::blocks::{FirBuilder, Apply, VectorSource, FileSink, ApplyIntoIter, Throttle};
use futuresdr::num_complex::Complex32;
use futuresdr::runtime::{Flowgraph, Runtime};
use futuresdr::macros::connect;
use futuresdr::anyhow::Result;
use futuresdr::blocks::seify::SinkBuilder;

use crate::intobits::{TypeIntoBitIteratorWrapper, RepeatNWrapper};

mod vco;
mod intobits;

const SAMPLE_RATE: f64 = 8e6;


fn main() -> Result<()> {
    let mut fg = Flowgraph::new();
    
    let src = VectorSource::<u8>::new(vec!(0x55_u8, 0x55, 0x55, 0x55, 0x55, 0x55));
    
    let expand_bytes_b = ApplyIntoIter::new(|b: &u8| {
        TypeIntoBitIteratorWrapper::<u8>::new(*b)
    });

    let data_repeat_b = ApplyIntoIter::new(|b: &u8| {
        RepeatNWrapper::<u8>::new(*b, 556)
    });

    let transform_fsk = Apply::<_, u8, f32>::new(|i: &u8| (*i as f32) * 2.0 - 1.0);

    let vco_fsk = vco::build_complex_vco(1.0, 0.0, 45e3_f64 * 2.0_f64 * PI / SAMPLE_RATE);

    let snk_b = SinkBuilder::new()
        .frequency(868.4e6)
        .sample_rate(SAMPLE_RATE)
        .gain(0.0);

    let roofing_filter_taps = futuredsp::firdes::kaiser::lowpass::<f32>(140e3_f64 / SAMPLE_RATE, 50e3_f64 / SAMPLE_RATE, 0.01);
    let roofing_filter = FirBuilder::new::<Complex32, Complex32, f32, Vec<f32>>(roofing_filter_taps);

    let debug_block = FileSink::<Complex32>::new("debug.filtered.c32");
    let throttle_block = Throttle::<Complex32>::new(SAMPLE_RATE);

    let debug_block_bits = FileSink::<u8>::new("debug.bits.u8");
    let debug_block_repeats = FileSink::<u8>::new("debug.bits.repeat.u8");
    let debug_block_fsk = FileSink::<f32>::new("debug.bytes.fsk.f32");
    let debug_block_fsk_cplx = FileSink::<Complex32>::new("debug.cplx.fsk.c32");

    connect!(fg, src > expand_bytes_b > data_repeat_b > transform_fsk > vco_fsk > roofing_filter);

    connect!(fg, expand_bytes_b > debug_block_bits);
    connect!(fg, data_repeat_b > debug_block_repeats);
    connect!(fg, transform_fsk > debug_block_fsk);
    connect!(fg, vco_fsk > debug_block_fsk_cplx);

    match snk_b.build() {
        Ok(snk) => {
            connect!(fg, roofing_filter > snk);
        },
        Err(e) => {
            println!("Error instantiating sink device: {}", e);
            connect!(fg, roofing_filter > throttle_block > debug_block);
        }
    };
    
    Runtime::new().run(fg)?;
    Ok(())
}
