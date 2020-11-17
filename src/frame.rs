use std::ops::{Index, IndexMut};

/// A frame represents a single step in a trajectory.
#[derive(Clone, Debug)]
pub struct Frame {
    /// Trajectory step
    pub step: usize,

    /// Time step (usually in picoseconds)
    pub time: f32,

    /// 3x3 box vector
    pub box_vector: [[f32; 3]; 3],

    /// 3D coordinates for N atoms where N is num_atoms
    pub coords: Vec<[f32; 3]>,
}

impl Default for Frame {
    fn default() -> Frame {
        Frame {
            step: 0,
            time: 0.0,
            box_vector: [[0.0; 3]; 3],
            coords: Vec::with_capacity(0),
        }
    }
}

impl Frame {
    /// Creates an empty frame with a capacity of 0
    pub fn new() -> Frame {
        Default::default()
    }

    /// Creates a frame with the given capacity
    pub fn with_len(num_atoms: usize) -> Frame {
        Frame {
            coords: vec![[0.0, 0.0, 0.0]; num_atoms],
            ..Default::default()
        }
    }

    /// Filters the frame by removing all atoms not matching the given indeces.
    pub fn filter_coords(self: &mut Frame, indices: &[usize]) {
        self.coords = self
            .coords
            .iter()
            .enumerate()
            .filter(|(i, _)| indices.contains(i))
            .map(|(_, elem)| *elem)
            .collect();
    }

    /// Length of the frame (number of atoms)
    pub fn len(self: &Frame) -> usize {
        self.num_atoms()
    }

    /// The number of atoms in the frame
    pub fn num_atoms(self: &Frame) -> usize {
        self.coords.len()
    }

    /// Resize the frame to have exactly `num_atoms` atoms, filling coords with zeros if necessary
    pub fn resize(&mut self, num_atoms: usize) {
        self.coords.resize(num_atoms, [0.0; 3])
    }
}

impl Index<usize> for Frame {
    type Output = [f32; 3];

    fn index(&self, index: usize) -> &Self::Output {
        &self.coords[index]
    }
}

impl IndexMut<usize> for Frame {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output { {
        &mut self.coords[index]
    }}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_with_capacity() {
        let frame = Frame::with_len(10);
        println!("{:?}", frame.coords);
        assert_eq!(frame.coords.len(), 10);
    }

    #[test]
    fn test_frame_filter_atoms() {
        let mut frame = Frame::with_len(3);
        frame[0] = [1.0, 2.0, 3.0];
        frame[1] = [4.0, 5.0, 6.0];
        frame[2] = [7.0, 8.0, 9.0];
        let filter: Vec<usize> = vec![1, 2];
        let mut frame_new = frame.clone();
        frame_new.filter_coords(&filter);
        assert!(frame_new.len() == filter.len());
        assert!(frame_new.coords[0] == frame[1]);
        assert!(frame_new.coords[1] == frame[2]);
    }

    #[test]
    fn test_frame_len() {
        let frame = Frame::with_len(10);
        assert_eq!(frame.len(), 10);
    }

    #[test]
    fn test_filter_coords() {
        let mut frame = Frame {
            step: 0,
            time: 0.0,
            box_vector: [[0.0; 3]; 3],
            coords: vec![[0.0; 3], [1.0; 3], [2.0; 3]]
        };

        frame.filter_coords(&[1]);
        for i in 0..3 {
            assert_approx_eq!(frame[0][i], 1.0);
        }
    }

    #[test]
    #[allow(unused_mut)]
    fn test_index() {
        // test Index
        let frame = Frame {
            step: 0,
            time: 0.0,
            box_vector: [[0.0; 3]; 3],
            coords: vec![[0.0; 3], [1.0; 3], [2.0; 3]]
        };
        for i in 0..frame.len() {
            for j in 0..3 {
                assert_approx_eq!(frame[i][j], frame[i][j]);
            }
        }

        // test IndexMut
        let mut frame = Frame {
            step: 0,
            time: 0.0,
            box_vector: [[0.0; 3]; 3],
            coords: vec![[0.0; 3], [1.0; 3], [2.0; 3]]
        };
        for i in 0..frame.len() {
            for j in 0..3 {
                assert_approx_eq!(frame[i][j], frame.coords[i][j]);
            }
        }

        frame[0] = [123.0; 3];
        for i in 0..frame.len() {
            for j in 0..3 {
                assert_approx_eq!(frame[i][j], frame.coords[i][j]);
                if i == 0 {
                    assert_approx_eq!(frame[i][j], 123.0);    
                }
            }
        }

    }
}
