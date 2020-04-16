use crate::*;
use crate::c_abi::xdrfile::exdrENDOFFILE;
use failure::Error;
use std::rc::Rc;

impl IntoIterator for XTCTrajectory {
    type Item = Result<Rc<Frame>, Error>;
    type IntoIter = XTCTrajectoryIterator;

    fn into_iter(mut self) -> Self::IntoIter {
        // TODO this should be handled without the requirement of unwrap
        let num_atoms = self.get_num_atoms().unwrap();
        XTCTrajectoryIterator {
            trajectory: self,
            item: Rc::new(Frame::with_capacity(num_atoms)),
            has_error: false
        }
    }
}

/* 
Iterator for trajectories. This iterator yields a Result<Frame, Error>
for each frame in the trajectory file and stops with yielding None once the 
trajectory is finished. Also yields None after the first occurence of an error
*/
pub struct XTCTrajectoryIterator {
    trajectory: XTCTrajectory,
    item: Rc<Frame>,
    has_error: bool,
}

impl Iterator for XTCTrajectoryIterator {
    type Item = Result<Rc<Frame>, Error>;

    fn next(&mut self) -> Option<Self::Item> { // Reuse old frame
        if self.has_error {
            return None;
        }

        let item: &mut Frame = match Rc::get_mut(&mut self.item) {
            Some(item) => item,
            None => {  // caller kept frame. Create new one
                self.item = Rc::new(Frame::with_capacity(self.item.num_atoms));
                Rc::get_mut(&mut self.item).unwrap()
            }

        };
        match self.trajectory.read(item) {
            Ok(()) => Some(Ok(Rc::clone(&self.item))),
            Err(msg) => {
                if msg.to_string().contains(&exdrENDOFFILE.to_string()) {
                    None
                } else {
                    self.has_error = true;
                    Some(Err(msg))
                }
            }
        } 
    }
}

impl IntoIterator for TRRTrajectory {
    type Item = Result<Rc<Frame>, Error>;
    type IntoIter = TRRTrajectoryIterator;

    fn into_iter(mut self) -> Self::IntoIter {
        // TODO this should be handled without the requirement of unwrap
        let num_atoms = self.get_num_atoms().unwrap();
        TRRTrajectoryIterator {
            trajectory: self,
            item: Rc::new(Frame::with_capacity(num_atoms)),
            has_error: false
        }
    }
}

/* 
Iterator for trajectories. This iterator yields a Result<Frame, Error>
for each frame in the trajectory file and stops with yielding None once the 
trajectory is finished. Also yields None after the first occurence of an error
*/
pub struct TRRTrajectoryIterator {
    trajectory: TRRTrajectory,
    item: Rc<Frame>,
    has_error: bool,
}

impl Iterator for TRRTrajectoryIterator {
    type Item = Result<Rc<Frame>, Error>;

    fn next(&mut self) -> Option<Self::Item> { // Reuse old frame
        if self.has_error {
            return None;
        }

        let item: &mut Frame = match Rc::get_mut(&mut self.item) {
            Some(item) => item,
            None => {  // caller kept frame. Create new one
                self.item = Rc::new(Frame::with_capacity(self.item.num_atoms));
                Rc::get_mut(&mut self.item).unwrap()
            }

        };
        match self.trajectory.read(item) {
            Ok(()) => Some(Ok(Rc::clone(&self.item))),
            Err(msg) => {
                if msg.to_string().contains(&exdrENDOFFILE.to_string()) {
                    None
                } else {
                    self.has_error = true;
                    Some(Err(msg))
                }
            }
        } 
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_xtc_trajectory_iterator() {
        let traj = XTCTrajectory::open(Path::new("tests/1l2y.xtc"), FileMode::Read).unwrap();
        let frames: Vec<Rc<Frame>> = traj.into_iter().filter_map(Result::ok).collect();
        assert!(frames.len() == 38);
        assert!(frames[0].step == 1, frames[0].step);
        assert!(frames[37].step == 38);
    }

    #[test]
    pub fn test_trr_trajectory_iterator() {
        let traj = TRRTrajectory::open(Path::new("tests/1l2y.trr"), FileMode::Read).unwrap();
        let frames: Vec<Rc<Frame>> = traj.into_iter().filter_map(Result::ok).collect();
        assert!(frames.len() == 38);
        assert!(frames[0].step == 1, frames[0].step);
        assert!(frames[37].step == 38);
    }
}