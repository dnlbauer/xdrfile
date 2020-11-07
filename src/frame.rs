use std::fmt;

/// A frame represents a single step in a trajectory.
#[derive(Clone)]
pub struct Frame {
    /// Number of atoms in the frame
    pub num_atoms: u32,

    /// Trajectory step
    pub step: u32,

    /// Time step (usually in picoseconds)
    pub time: f32,

    /// 3x3 box vector
    pub box_vector: [[f32; 3usize]; 3usize],

    /// 3D coordinates for N atoms where N is num_atoms
    pub coords: Vec<[f32; 3usize]>,
}

impl Default for Frame {
    fn default() -> Frame {
        Frame {
            num_atoms: 0,
            step: 0,
            time: 0.0,
            box_vector: [[0.0; 3]; 3],
            coords: Vec::with_capacity(0),
        }
    }
}

impl fmt::Debug for Frame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Frame {{ atoms: {}, step: {}, time: {}, \
            box: {:?}, coords: {:?} }}",
            self.num_atoms, self.step, self.time, self.box_vector, self.coords
        )
    }
}

impl Frame {
    /// Creates an empty frame with a capacity of 0
    pub fn new() -> Frame {
        Frame {
            ..Default::default()
        }
    }

    /// Creates a frame with the given capacity
    pub fn with_capacity(num_atoms: u32) -> Frame {
        Frame {
            num_atoms,
            coords: vec![[0.0, 0.0, 0.0]; num_atoms as usize],
            ..Default::default()
        }
    }

    /// Filters the frame by removing all atoms not matching the given indeces.
    pub fn filter_coords(self: &mut Frame, indeces: &[usize]) {
        self.coords = self
            .coords
            .iter()
            .map(|elem| elem.clone())
            .enumerate()
            .filter(|&(i, _)| indeces.contains(&i))
            .map(|(_, elem)| elem)
            .collect();
        self.num_atoms = self.coords.len() as u32;
    }

    /// Length of the frame (number of atoms)
    pub fn len(self: &Frame) -> usize {
        self.num_atoms as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_with_capacity() {
        let frame = Frame::with_capacity(10);
        println!("{:?}", frame.coords);
        assert_eq!(frame.coords.len(), 10);
    }

    #[test]
    fn test_frame_filter_atoms() {
        let mut frame = Frame::with_capacity(3);
        frame.coords[0] = [1.0, 2.0, 3.0];
        frame.coords[1] = [4.0, 5.0, 6.0];
        frame.coords[2] = [7.0, 8.0, 9.0];
        let filter: Vec<usize> = vec![1, 2];
        let mut frame_new = frame.clone();
        frame_new.filter_coords(&filter);
        assert!(frame_new.num_atoms as usize == filter.len());
        assert!(frame_new.coords[0] == frame.coords[1]);
        assert!(frame_new.coords[1] == frame.coords[2]);
    }

    #[test]
    fn test_frame_len() {
        let frame = Frame::with_capacity(10);
        assert_eq!(frame.len(), 10);
    }
}
