use super::xdrfile::*;

extern "C" {
    pub fn read_xtc_natoms(
        fn_: *const ::std::os::raw::c_char,
        natoms: *const ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn read_xtc_nframes(
        fn_: *const ::std::os::raw::c_char,
        nframes: *const ::std::os::raw::c_ulong,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn read_xtc(
        xd: *mut XDRFILE,
        natoms: ::std::os::raw::c_int,
        step: *mut ::std::os::raw::c_int,
        time: *mut ::std::os::raw::c_float,
        box_vec: *mut Matrix,
        x: *mut Rvec,
        prec: *mut ::std::os::raw::c_float,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn write_xtc(
        xd: *mut XDRFILE,
        natoms: ::std::os::raw::c_int,
        step: ::std::os::raw::c_int,
        time: ::std::os::raw::c_float,
        box_vec: *mut Matrix,
        x: *mut Rvec,
        prec: ::std::os::raw::c_float,
    ) -> ::std::os::raw::c_int;
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::ffi::CString;
    use tempfile::NamedTempFile;

    #[test]
    fn test_read_xtc_natoms() -> Result<(), Box<dyn std::error::Error>> {
        let path = CString::new("tests/1l2y.xtc")?;

        let mut natoms = 0;
        unsafe {
            read_xtc_natoms(path.as_ptr() as *mut i8, &mut natoms);
        }
        assert!(natoms == 304);
        Ok(())
    }

    #[test]
    fn test_read_xtc_nframes() -> Result<(), Box<dyn std::error::Error>> {
        let path = CString::new("tests/1l2y.xtc")?;
        let mut nframes: u64 = 0;

        unsafe {
            let code = read_xtc_nframes(path.as_ptr(), &mut nframes);
            assert!(code as u32 == exdrOK);
        }
        assert!(nframes == 38, "{:?}", nframes);
        Ok(())
    }

    #[test]
    fn test_read_write_xtc() -> Result<(), Box<dyn std::error::Error>> {
        let tempfile = NamedTempFile::new()?;
        let tmp_path = CString::new(
            tempfile
                .path()
                .to_str()
                .expect("Could not convert path to str"),
        )?;

        // write atoms to tempfile
        let natoms: i32 = 2;
        let time: f32 = 2.0;
        let step: i32 = 5;
        let box_vec: Matrix = [[1.0, 2.0, 3.0], [2.0, 1.0, 3.0], [3.0, 2.0, 1.0]];
        let x: Vec<Rvec> = vec![[1.0, 1.0, 1.0], [1.0, 1.0, 1.0]];

        unsafe {
            let mode = CString::new("w")?;
            let xdr = xdrfile_open(tmp_path.as_ptr(), mode.as_ptr());
            let write_code = write_xtc(
                xdr,
                natoms,
                step,
                time,
                box_vec.as_ptr() as *mut Matrix,
                x.as_ptr() as *mut Rvec,
                1000.0,
            );
            assert!(write_code as u32 == exdrOK);
            xdrfile_close(xdr);
        }

        // read atoms from tempfile
        let mut time2: f32 = 0.0;
        let mut step2: i32 = 0;
        let box_vec2: Matrix = [[0.0, 0.0, 0.0]; 3];
        let x2: Vec<Rvec> = vec![[0.0, 0.0, 0.0]; 2];
        let mut prec: f32 = 0.0;

        unsafe {
            let mode = CString::new("r")?;
            let xdr = xdrfile_open(tmp_path.as_ptr(), mode.as_ptr());
            let read_code = read_xtc(
                xdr,
                natoms,
                &mut step2,
                &mut time2,
                box_vec2.as_ptr() as *mut Matrix,
                x2.as_ptr() as *mut Rvec,
                &mut prec,
            );
            assert!(read_code as u32 == exdrOK);
            xdrfile_close(xdr);
        }

        // make sure everything is still the same
        assert!(step2 == step);
        assert!(time2 == time);
        assert!(box_vec2 == box_vec);
        assert!(x2 == x);
        Ok(())
    }
}
