#[cfg(test)]
mod integration {

    use std::rc::Rc;
    use xdrfile::*;

    #[test]
    fn test_use_library() -> Result<()> {
        let mut trj = XTCTrajectory::open_read("tests/1l2y.xtc")?;
        let num_atoms = trj.get_num_atoms()?;
        let mut frame = Frame::with_len(num_atoms as usize);

        trj.read(&mut frame)?;
        trj.read(&mut frame)?;
        assert_eq!(frame.step, 2);
        Ok(())
    }

    #[test]
    fn test_use_library_iterator() -> Result<()> {
        let trj = XTCTrajectory::open_read("tests/1l2y.xtc")?;
        let frames: Result<Vec<Rc<Frame>>> = trj.into_iter().collect();
        for (idx, frame) in frames?.iter().enumerate() {
            assert_eq!(frame.step as usize, idx + 1);
        }
        Ok(())
    }
}
