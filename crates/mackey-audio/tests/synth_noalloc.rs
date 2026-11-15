//! NF-8.1: `Synth::apply` and `Synth::render` never allocate.
//!
//! This binary installs `assert_no_alloc`'s allocator (which aborts the process when memory is
//! allocated inside `assert_no_alloc`), so it has its own file and does not affect other tests.
//! The guard is active in debug builds only (`cargo test`); `make props` runs release builds,
//! where the test still exercises the same calls without the guard.

use assert_no_alloc::{AllocDisabler, assert_no_alloc};
use mackey_audio::dsp::{PlaybackPlan, Synth, SynthCommand};
use mackey_theory::ReferencePitch;

#[cfg(debug_assertions)]
#[global_allocator]
static ALLOCATOR: AllocDisabler = AllocDisabler;

#[test]
fn nf_8_1_render_and_apply_do_not_allocate() {
    // Everything is allocated here, outside the guard.
    let mut synth = Synth::new(48_000, 16);
    let mut block = vec![0.0_f32; 512];
    let plan = PlaybackPlan::strum(&[40, 45, 50, 55, 59, 64]).unwrap();
    let commands = plan.commands(48_000, ReferencePitch::default(), 0);

    assert_no_alloc(|| {
        for c in &commands {
            synth.apply(*c);
        }
        // Steal voices, reference tone re-plucks, release and volume changes: every path.
        synth.apply(SynthCommand::ReferenceToneOn {
            id: 99,
            frequency_hz: 110.0,
        });
        for i in 0..40_u32 {
            synth.apply(SynthCommand::NoteOn {
                id: 1000 + i,
                frequency_hz: 82.41 + f32::from(u16::try_from(i).unwrap()) * 11.0,
                velocity: 1.0,
                delay_samples: i * 97,
            });
            synth.render(&mut block);
            if i == 10 {
                synth.apply(SynthCommand::NoteOff { id: 1005 });
                synth.apply(SynthCommand::SetVolume(0.5));
                synth.apply(SynthCommand::SetMute(true));
                synth.apply(SynthCommand::SetMute(false));
            }
        }
        // 4 s of audio to cross the 1.5 s re-pluck boundaries.
        for _ in 0..(4 * 48_000 / 512) {
            synth.render(&mut block);
        }
        synth.apply(SynthCommand::ReferenceToneOff);
        synth.apply(SynthCommand::StopAll);
        synth.render(&mut block);
    });
    assert!(block.iter().all(|v| v.is_finite()));
}
