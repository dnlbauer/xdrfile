#[cfg(test)]
mod integration {

    use std::path::Path;
    use std::rc::Rc;
    use xdrfile::*;

    #[test]
    fn test_use_library() {
        let mut trj = XTCTrajectory::open("tests/1l2y.xtc", FileMode::Read).unwrap();
        let num_atoms = trj.get_num_atoms().unwrap();
        let mut frame = Frame::with_capacity(num_atoms);

        trj.read(&mut frame).unwrap();
        trj.read(&mut frame).unwrap();
        assert_eq!(frame.step, 2);
    }

    #[test]
    fn test_use_library_iterator() {
        let path = Path::new("tests/1l2y.xtc");

        let trj = XTCTrajectory::open(path, FileMode::Read).unwrap();
        let frames: Vec<Rc<Frame>> = trj.into_iter().filter_map(Result::ok).collect();
        for (idx, frame) in frames.iter().enumerate() {
            assert_eq!(frame.step as usize, idx + 1);
        }
    }
}
