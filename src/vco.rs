use std::iter;

use futuresdr::{macros::async_trait, num_complex::Complex32};
use futuresdr::runtime::{Block, BlockMetaBuilder, StreamIoBuilder, MessageIoBuilder, Kernel, WorkIo, StreamIo, MessageIo, BlockMeta};
use futuresdr::blocks::{FixedPointPhase, signal_source::NCO};
use futuresdr::anyhow::Result;

pub struct VariableSignalSource<F>
where
    F: FnMut(FixedPointPhase) -> Complex32 + Send + 'static,
{
    nco: NCO,
    phase_to_amplitude: F,
    amplitude: f32,
    offset: f32,
    coeff: f64,
}

impl<F> VariableSignalSource<F>
where
    F: FnMut(FixedPointPhase) -> Complex32 + Send + 'static,
{
    /// Create VariableSignalSource block
    pub fn new(phase_to_amplitude: F, nco: NCO, amplitude: f32, offset: f32, coeff: f64) -> Block {
        Block::new(
            BlockMetaBuilder::new("VariableSignalSource").build(),
            StreamIoBuilder::new().add_output::<Complex32>("out").add_input::<f32>("in").build(),
            MessageIoBuilder::<Self>::new().build(),
            VariableSignalSource {
                nco,
                phase_to_amplitude,
                amplitude,
                offset,
                coeff
            },
        )
    }
}

#[doc(hidden)]
#[async_trait]
impl<F> Kernel for VariableSignalSource<F>
where
    F: FnMut(FixedPointPhase) -> Complex32 + Send + 'static,
{
    async fn work(
        &mut self,
        io: &mut WorkIo,
        sio: &mut StreamIo,
        _mio: &mut MessageIo<Self>,
        _meta: &mut BlockMeta,
    ) -> Result<()> {
        let o = sio.output(0).slice::<Complex32>();
        let i = sio.input(0).slice::<f32>();

        let mut consumed: usize = 0;

        for (o_v, v_in) in iter::zip(o.iter_mut(), i.iter()) {
            let a = (self.phase_to_amplitude)(self.nco.phase);
            let a = a * self.amplitude;
            let a = a + self.offset;
            *o_v = a;
            self.nco.set_freq((*v_in as f64 * self.coeff) as f32);
            self.nco.step();
            consumed += 1;
        }

        sio.output(0).produce(consumed);
        sio.input(0).consume(consumed);

        if sio.input(0).finished() && consumed == i.len() && consumed < o.len() {
            io.finished = true;
        }

        Ok(())
    }
}

pub fn build_complex_vco(amplitude: f32, offset: f32, coeff: f64) -> Block {
    let cplx_phase_to_ampl = |phase: FixedPointPhase| Complex32::new(phase.cos(), phase.sin());
    let nco = NCO::new(
        0.0,
        0.0,
    );
    VariableSignalSource::new(cplx_phase_to_ampl, nco, amplitude, offset, coeff)
}