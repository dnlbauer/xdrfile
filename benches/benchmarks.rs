use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::rc::Rc;
use std::time::Duration;
use tempfile::NamedTempFile;
use xdrfile::*;

/// generate a temporary test trajectory of given length
fn gen_test_traj(num_atoms: usize, num_frames: usize) -> Result<NamedTempFile> {
    let tempfile = NamedTempFile::new().expect("Could not create temporary file");
    let tmp_path = tempfile.path().to_path_buf();
    let mut f = XtcTrajectory::open_write(&tmp_path)?;

    let mut frame = Frame {
        step: 1,
        time: 1.0,
        box_vector: [[1.0, 2.0, 3.0], [2.0, 1.0, 3.0], [3.0, 2.0, 1.0]],
        coords: vec![[1.0, 1.1, 1.2]; num_atoms],
    };

    for _ in 0..num_frames {
        for j in 0..num_atoms {
            frame.coords[j][0] += 1.0;
            frame.coords[j][1] += 1.0;
            frame.coords[j][2] += 1.0;
            frame.time += 1.0;
            frame.step += 1;
        }
        f.write(&frame)?;
    }
    f.flush()?;

    Ok(tempfile)
}

// Iterate over a trajectory and do some stuff on it
fn iterate_traj(file: &NamedTempFile, num_atoms: usize) -> Result<()> {
    let path = file.path();
    let traj = XtcTrajectory::open_read(path)?;
    for frame in traj.into_iter() {
        assert_eq!(frame?.len(), num_atoms);
    }
    Ok(())
}

// Iterate over a trajectory and keep_frames its frames
// This benchmarks iteration speed when frames are not reused
fn iterate_traj_keep_frames(file: &NamedTempFile, num_frames: usize) -> Result<()> {
    let path = file.path();
    let traj = XtcTrajectory::open_read(path)?;
    let frames = traj
        .into_iter()
        .filter_map(Result::ok)
        .collect::<Vec<Rc<Frame>>>();
    assert_eq!(num_frames, frames.len());
    Ok(())
}

fn bench_iterate_traj(c: &mut Criterion) {
    let num_atoms = 100;
    let num_frames = 1000;
    let tempfile = gen_test_traj(num_atoms, num_frames).unwrap();

    let mut group = c.benchmark_group("iterate_traj");
    group
        .significance_level(0.05)
        .warm_up_time(Duration::from_secs(10))
        .sample_size(2500)
        .noise_threshold(0.05); // high noise thresholds because of disk i/o
    group.bench_function("iterate_traj", |b| {
        b.iter(|| iterate_traj(black_box(&tempfile), black_box(num_atoms)).unwrap())
    });
    group.bench_function("iterate_traj_keep_frames", |b| {
        b.iter(|| iterate_traj_keep_frames(black_box(&tempfile), black_box(num_frames)).unwrap())
    });
}

criterion_group!(benches, bench_iterate_traj);
criterion_main!(benches);
