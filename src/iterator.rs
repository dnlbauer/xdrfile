use crate::*;
use std::rc::Rc;

fn into_iter_inner<T: Trajectory>(mut traj: T) -> TrajectoryIterator<T> {
    let num_atoms = traj.get_num_atoms();
    let frame = match &num_atoms {
        Ok(num_atoms) => Frame::with_capacity(*num_atoms),
        Err(_) => Frame::new(),
    };
    TrajectoryIterator {
        trajectory: traj,
        item: Rc::new(frame),
        has_error: false,
    }
}

impl IntoIterator for XTCTrajectory {
    type Item = Result<Rc<Frame>>;
    type IntoIter = TrajectoryIterator<XTCTrajectory>;

    fn into_iter(self) -> Self::IntoIter {
        into_iter_inner(self)
    }
}

impl IntoIterator for TRRTrajectory {
    type Item = Result<Rc<Frame>>;
    type IntoIter = TrajectoryIterator<TRRTrajectory>;

    fn into_iter(self) -> Self::IntoIter {
        into_iter_inner(self)
    }
}

/*
Iterator for trajectories. This iterator yields a Result<Frame, Error>
for each frame in the trajectory file and stops with yielding None once the
trajectory is finished. Also yields None after the first occurence of an error
*/
pub struct TrajectoryIterator<T> {
    trajectory: T,
    item: Rc<Frame>,
    has_error: bool,
}

impl<T: Trajectory> TrajectoryIterator<T> {
    /// Inner function for `next()`  to seperate error handling from iteration logic
    fn next_inner(&mut self) -> <Self as Iterator>::Item {
        // If we couldn't read the number of frames when we called into_iter, return that error now
        // It's OK to do this every frame because the result is cached by Trajectory
        let num_atoms = match &self.trajectory.get_num_atoms() {
            &Ok(n) => n,
            Err(e) => Err(Error::CouldNotCheckNAtoms(Box::new(e.clone())))?,
        };

        // Reuse old frame
        let item: &mut Frame = match Rc::get_mut(&mut self.item) {
            Some(item) => item,
            None => {
                // caller kept frame. Create new one
                self.item = Rc::new(Frame::with_capacity(num_atoms));
                Rc::get_mut(&mut self.item).expect("Could not get mutable access to new Rc")
            }
        };

        self.trajectory.read(item)?;
        Ok(Rc::clone(&self.item))
    }
}

impl<T> Iterator for TrajectoryIterator<T>
where
    T: Trajectory,
{
    type Item = Result<Rc<Frame>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.has_error {
            return None;
        }

        match self.next_inner() {
            Ok(item) => Some(Ok(item)),
            Err(e) if e.is_eof() => None,
            Err(e) => {
                self.has_error = true;
                Some(Err(e))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_xtc_trajectory_iterator() -> Result<()> {
        let traj = XTCTrajectory::open_read("tests/1l2y.xtc")?;
        let frames: Result<Vec<Rc<Frame>>> = traj.into_iter().collect();
        let frames = frames?;
        assert!(frames.len() == 38);
        assert!(frames[0].step == 1, frames[0].step);
        assert!(frames[37].step == 38);
        Ok(())
    }

    #[test]
    pub fn test_trr_trajectory_iterator() -> Result<()> {
        let traj = TRRTrajectory::open_read("tests/1l2y.trr")?;
        let frames: Result<Vec<Rc<Frame>>> = traj.into_iter().collect();
        let frames = frames?;
        assert!(frames.len() == 38);
        assert!(frames[0].step == 1, frames[0].step);
        assert!(frames[37].step == 38);
        Ok(())
    }
}
