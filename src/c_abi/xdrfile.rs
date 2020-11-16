pub const DIM: u32 = 3;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct XDRFILE {
    _unused: [u8; 0],
}

pub const exdrOK: i32 = 0;
pub const exdrHEADER: i32 = 1;
pub const exdrSTRING: i32 = 2;
pub const exdrDOUBLE: i32 = 3;
pub const exdrINT: i32 = 4;
pub const exdrFLOAT: i32 = 5;
pub const exdrUINT: i32 = 6;
pub const exdr3DX: i32 = 7;
pub const exdrCLOSE: i32 = 8;
pub const exdrMAGIC: i32 = 9;
pub const exdrNOMEM: i32 = 10;
pub const exdrENDOFFILE: i32 = 11;
pub const exdrFILENOTFOUND: i32 = 12;
pub const exdrNR: i32 = 13;

extern "C" {
    pub static mut exdr_message: [*mut ::std::os::raw::c_char; 13usize];
}

pub type Matrix = [[::std::os::raw::c_float; 3usize]; 3usize];
pub type Rvec = [::std::os::raw::c_float; 3usize];
pub type Mybool = ::std::os::raw::c_int;

extern "C" {
    #[doc = " \\brief Open a portable binary file, just like fopen()"]
    #[doc = ""]
    #[doc = "  Use this routine much like calls to the standard library function"]
    #[doc = "  fopen(). The only difference is that the returned pointer should only"]
    #[doc = "  be used with routines defined in this header."]
    #[doc = ""]
    #[doc = "  \\param path  Full or relative path (including name) of the file"]
    #[doc = "  \\param mode  \"r\" for reading, \"w\" for writing, \"a\" for append."]
    #[doc = ""]
    #[doc = "  \\return Pointer to abstract xdr file datatype, or NULL if an error occurs."]
    #[doc = ""]
    pub fn xdrfile_open(
        path: *const ::std::os::raw::c_char,
        mode: *const ::std::os::raw::c_char,
    ) -> *mut XDRFILE;
}
extern "C" {
    #[doc = " \\brief Close a previously opened portable binary file, just like fclose()"]
    #[doc = ""]
    #[doc = "  Use this routine much like calls to the standard library function"]
    #[doc = "  fopen(). The only difference is that it is used for an XDRFILE handle"]
    #[doc = "  instead of a FILE handle."]
    #[doc = ""]
    #[doc = "  \\param xfp  Pointer to an abstract XDRFILE datatype"]
    #[doc = ""]
    #[doc = "  \\return     0 on success, non-zero on error."]
    pub fn xdrfile_close(xfp: *mut XDRFILE) -> ::std::os::raw::c_int;
}
extern "C" {
    #[doc = " \\brief Read one or more \\a char type variable(s)"]
    #[doc = ""]
    #[doc = "  \\param ptr    Pointer to memory where data should be written"]
    #[doc = "  \\param ndata  Number of characters to read"]
    #[doc = "  \\param xfp    Handle to portable binary file, created with xdrfile_open()"]
    #[doc = ""]
    #[doc = "  \\return       Number of characters read"]
    pub fn xdrfile_read_char(
        ptr: *mut ::std::os::raw::c_char,
        ndata: ::std::os::raw::c_int,
        xfp: *mut XDRFILE,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    #[doc = " \\brief Write one or more \\a characters type variable(s)"]
    #[doc = ""]
    #[doc = "  \\param ptr    Pointer to memory where data should be read"]
    #[doc = "  \\param ndata  Number of characters to write."]
    #[doc = "  \\param xfp    Handle to portable binary file, created with xdrfile_open()"]
    #[doc = ""]
    #[doc = "  \\return       Number of characters written"]
    pub fn xdrfile_write_char(
        ptr: *mut ::std::os::raw::c_char,
        ndata: ::std::os::raw::c_int,
        xfp: *mut XDRFILE,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    #[doc = " \\brief Read one or more \\a unsigned \\a char type variable(s)"]
    #[doc = ""]
    #[doc = "  \\param ptr    Pointer to memory where data should be written"]
    #[doc = "  \\param ndata  Number of unsigned characters to read"]
    #[doc = "  \\param xfp    Handle to portable binary file, created with xdrfile_open()"]
    #[doc = ""]
    #[doc = "  \\return       Number of unsigned characters read"]
    pub fn xdrfile_read_uchar(
        ptr: *mut ::std::os::raw::c_uchar,
        ndata: ::std::os::raw::c_int,
        xfp: *mut XDRFILE,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    #[doc = " \\brief Write one or more \\a unsigned \\a characters type variable(s)"]
    #[doc = ""]
    #[doc = "  \\param ptr    Pointer to memory where data should be read"]
    #[doc = "  \\param ndata  Number of unsigned characters to write."]
    #[doc = "  \\param xfp    Handle to portable binary file, created with xdrfile_open()"]
    #[doc = ""]
    #[doc = "  \\return       Number of unsigned characters written"]
    pub fn xdrfile_write_uchar(
        ptr: *mut ::std::os::raw::c_uchar,
        ndata: ::std::os::raw::c_int,
        xfp: *mut XDRFILE,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    #[doc = " \\brief Read one or more \\a short type variable(s)"]
    #[doc = ""]
    #[doc = "  \\param ptr    Pointer to memory where data should be written"]
    #[doc = "  \\param ndata  Number of shorts to read"]
    #[doc = "  \\param xfp    Handle to portable binary file, created with xdrfile_open()"]
    #[doc = ""]
    #[doc = "  \\return       Number of shorts read"]
    pub fn xdrfile_read_short(
        ptr: *mut ::std::os::raw::c_short,
        ndata: ::std::os::raw::c_int,
        xfp: *mut XDRFILE,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    #[doc = " \\brief Write one or more \\a short type variable(s)"]
    #[doc = ""]
    #[doc = "  \\param ptr    Pointer to memory where data should be read"]
    #[doc = "  \\param ndata  Number of shorts to write."]
    #[doc = "  \\param xfp    Handle to portable binary file, created with xdrfile_open()"]
    #[doc = ""]
    #[doc = "  \\return       Number of shorts written"]
    pub fn xdrfile_write_short(
        ptr: *mut ::std::os::raw::c_short,
        ndata: ::std::os::raw::c_int,
        xfp: *mut XDRFILE,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    #[doc = " \\brief Read one or more \\a unsigned \\a short type variable(s)"]
    #[doc = ""]
    #[doc = "  \\param ptr    Pointer to memory where data should be written"]
    #[doc = "  \\param ndata  Number of unsigned shorts to read"]
    #[doc = "  \\param xfp    Handle to portable binary file, created with xdrfile_open()"]
    #[doc = ""]
    #[doc = "  \\return       Number of unsigned shorts read"]
    pub fn xdrfile_read_ushort(
        ptr: *mut ::std::os::raw::c_ushort,
        ndata: ::std::os::raw::c_int,
        xfp: *mut XDRFILE,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    #[doc = " \\brief Write one or more \\a unsigned \\a short type variable(s)"]
    #[doc = ""]
    #[doc = "  \\param ptr    Pointer to memory where data should be read"]
    #[doc = "  \\param ndata  Number of unsigned shorts to write."]
    #[doc = "  \\param xfp    Handle to portable binary file, created with xdrfile_open()"]
    #[doc = ""]
    #[doc = "  \\return       Number of unsigned shorts written"]
    pub fn xdrfile_write_ushort(
        ptr: *mut ::std::os::raw::c_ushort,
        ndata: ::std::os::raw::c_int,
        xfp: *mut XDRFILE,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    #[doc = " \\brief Read one or more \\a integer type variable(s)"]
    #[doc = ""]
    #[doc = "  \\param ptr    Pointer to memory where data should be written"]
    #[doc = "  \\param ndata  Number of integers to read"]
    #[doc = "  \\param xfp    Handle to portable binary file, created with xdrfile_open()"]
    #[doc = ""]
    #[doc = "  \\return       Number of integers read"]
    #[doc = ""]
    #[doc = "  The integer data type is assumed to be less than or equal to 32 bits."]
    #[doc = ""]
    #[doc = "  We do not provide any routines for reading/writing 64-bit integers, since"]
    #[doc = "  - Not all XDR implementations support it"]
    #[doc = "  - Not all machines have 64-bit integers"]
    #[doc = ""]
    #[doc = "  Split your 64-bit data into two 32-bit integers for portability!"]
    pub fn xdrfile_read_int(
        ptr: *mut ::std::os::raw::c_int,
        ndata: ::std::os::raw::c_int,
        xfp: *mut XDRFILE,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    #[doc = " \\brief Write one or more \\a integer type variable(s)"]
    #[doc = ""]
    #[doc = "  \\param ptr    Pointer to memory where data should be read"]
    #[doc = "  \\param ndata  Number of integers to write."]
    #[doc = "  \\param xfp    Handle to portable binary file, created with xdrfile_open()"]
    #[doc = ""]
    #[doc = "  \\return       Number of integers written"]
    #[doc = ""]
    #[doc = "  The integer data type is assumed to be less than or equal to 32 bits."]
    #[doc = ""]
    #[doc = "  We do not provide any routines for reading/writing 64-bit integers, since"]
    #[doc = "  - Not all XDR implementations support it"]
    #[doc = "  - Not all machines have 64-bit integers"]
    #[doc = ""]
    #[doc = "  Split your 64-bit data into two 32-bit integers for portability!"]
    pub fn xdrfile_write_int(
        ptr: *mut ::std::os::raw::c_int,
        ndata: ::std::os::raw::c_int,
        xfp: *mut XDRFILE,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    #[doc = " \\brief Read one or more \\a unsigned \\a integers type variable(s)"]
    #[doc = ""]
    #[doc = "  \\param ptr    Pointer to memory where data should be written"]
    #[doc = "  \\param ndata  Number of unsigned integers to read"]
    #[doc = "  \\param xfp    Handle to portable binary file, created with xdrfile_open()"]
    #[doc = ""]
    #[doc = "  \\return       Number of unsigned integers read"]
    #[doc = ""]
    #[doc = "  The integer data type is assumed to be less than or equal to 32 bits."]
    #[doc = ""]
    #[doc = "  We do not provide any routines for reading/writing 64-bit integers, since"]
    #[doc = "  - Not all XDR implementations support it"]
    #[doc = "  - Not all machines have 64-bit integers"]
    #[doc = ""]
    #[doc = "  Split your 64-bit data into two 32-bit integers for portability!"]
    pub fn xdrfile_read_uint(
        ptr: *mut ::std::os::raw::c_uint,
        ndata: ::std::os::raw::c_int,
        xfp: *mut XDRFILE,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    #[doc = " \\brief Write one or more \\a unsigned \\a integer type variable(s)"]
    #[doc = ""]
    #[doc = "  \\param ptr    Pointer to memory where data should be read"]
    #[doc = "  \\param ndata  Number of unsigned integers to write."]
    #[doc = "  \\param xfp    Handle to portable binary file, created with xdrfile_open()"]
    #[doc = ""]
    #[doc = "  \\return       Number of unsigned integers written"]
    #[doc = ""]
    #[doc = "  The integer data type is assumed to be less than or equal to 32 bits."]
    #[doc = ""]
    #[doc = "  We do not provide any routines for reading/writing 64-bit integers, since"]
    #[doc = "  - Not all XDR implementations support it"]
    #[doc = "  - Not all machines have 64-bit integers"]
    #[doc = ""]
    #[doc = "  Split your 64-bit data into two 32-bit integers for portability!"]
    pub fn xdrfile_write_uint(
        ptr: *mut ::std::os::raw::c_uint,
        ndata: ::std::os::raw::c_int,
        xfp: *mut XDRFILE,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    #[doc = " \\brief Read one or more \\a float type variable(s)"]
    #[doc = ""]
    #[doc = "  \\param ptr    Pointer to memory where data should be written"]
    #[doc = "  \\param ndata  Number of floats to read"]
    #[doc = "  \\param xfp    Handle to portable binary file, created with xdrfile_open()"]
    #[doc = ""]
    #[doc = "  \\return       Number of floats read"]
    pub fn xdrfile_read_float(
        ptr: *mut ::std::os::raw::c_float,
        ndata: ::std::os::raw::c_int,
        xfp: *mut XDRFILE,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    #[doc = " \\brief Write one or more \\a float type variable(s)"]
    #[doc = ""]
    #[doc = "  \\param ptr    Pointer to memory where data should be read"]
    #[doc = "  \\param ndata  Number of floats to write."]
    #[doc = "  \\param xfp    Handle to portable binary file, created with xdrfile_open()"]
    #[doc = ""]
    #[doc = "  \\return       Number of floats written"]
    pub fn xdrfile_write_float(
        ptr: *mut ::std::os::raw::c_float,
        ndata: ::std::os::raw::c_int,
        xfp: *mut XDRFILE,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    #[doc = " \\brief Read one or more \\a double type variable(s)"]
    #[doc = ""]
    #[doc = "  \\param ptr    Pointer to memory where data should be written"]
    #[doc = "  \\param ndata  Number of doubles to read"]
    #[doc = "  \\param xfp    Handle to portable binary file, created with xdrfile_open()"]
    #[doc = ""]
    #[doc = "  \\return       Number of doubles read"]
    pub fn xdrfile_read_double(
        ptr: *mut ::std::os::raw::c_double,
        ndata: ::std::os::raw::c_int,
        xfp: *mut XDRFILE,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    #[doc = " \\brief Write one or more \\a double type variable(s)"]
    #[doc = ""]
    #[doc = "  \\param ptr    Pointer to memory where data should be read"]
    #[doc = "  \\param ndata  Number of double to write."]
    #[doc = "  \\param xfp    Handle to portable binary file, created with xdrfile_open()"]
    #[doc = ""]
    #[doc = "  \\return       Number of doubles written"]
    pub fn xdrfile_write_double(
        ptr: *mut ::std::os::raw::c_double,
        ndata: ::std::os::raw::c_int,
        xfp: *mut XDRFILE,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    #[doc = " \\brief Read a string (array of characters)"]
    #[doc = ""]
    #[doc = "  \\param ptr     Pointer to memory where data should be written"]
    #[doc = "  \\param maxlen  Maximum length of string. If no end-of-string is encountered,"]
    #[doc = "                 one byte less than this is read and end-of-string appended."]
    #[doc = "  \\param xfp     Handle to portable binary file, created with xdrfile_open()"]
    #[doc = ""]
    #[doc = "  \\return        Number of characters read, including end-of-string"]
    pub fn xdrfile_read_string(
        ptr: *mut ::std::os::raw::c_char,
        maxlen: ::std::os::raw::c_int,
        xfp: *mut XDRFILE,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    #[doc = " \\brief Write a string (array of characters)"]
    #[doc = ""]
    #[doc = "  \\param ptr     Pointer to memory where data should be read"]
    #[doc = "  \\param xfp     Handle to portable binary file, created with xdrfile_open()"]
    #[doc = ""]
    #[doc = "  \\return        Number of characters written, including end-of-string"]
    pub fn xdrfile_write_string(
        ptr: *mut ::std::os::raw::c_char,
        xfp: *mut XDRFILE,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    #[doc = " \\brief Read raw bytes from file (unknown datatype)"]
    #[doc = ""]
    #[doc = "  \\param ptr     Pointer to memory where data should be written"]
    #[doc = "  \\param nbytes  Number of bytes to read. No conversion whatsoever is done."]
    #[doc = "  \\param xfp     Handle to portable binary file, created with xdrfile_open()"]
    #[doc = ""]
    #[doc = "  \\return        Number of bytes read from file"]
    pub fn xdrfile_read_opaque(
        ptr: *mut ::std::os::raw::c_char,
        nbytes: ::std::os::raw::c_int,
        xfp: *mut XDRFILE,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    #[doc = " \\brief Write raw bytes to file (unknown datatype)"]
    #[doc = ""]
    #[doc = "  \\param ptr     Pointer to memory where data should be read"]
    #[doc = "  \\param nbytes  Number of bytes to write. No conversion whatsoever is done."]
    #[doc = "  \\param xfp     Handle to portable binary file, created with xdrfile_open()"]
    #[doc = ""]
    #[doc = "  \\return        Number of bytes written to file"]
    pub fn xdrfile_write_opaque(
        ptr: *mut ::std::os::raw::c_char,
        nbytes: ::std::os::raw::c_int,
        xfp: *mut XDRFILE,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    #[doc = " \\brief Compress coordiates in a float array to XDR file"]
    #[doc = ""]
    #[doc = "  This routine will perform \\a lossy compression on the three-dimensional"]
    #[doc = "  coordinate data data specified and store it in the XDR file."]
    #[doc = ""]
    #[doc = "  The lossy part of the compression consists of multiplying each"]
    #[doc = "  coordinate with the precision argument and then rounding to integers."]
    #[doc = "  We suggest a default value of 1000.0, which means you are guaranteed"]
    #[doc = "  three decimals of accuracy. The only limitation is that scaled coordinates"]
    #[doc = "  must still fit in an integer variable, so if the precision is 1000.0 the"]
    #[doc = "  coordinate magnitudes must be less than +-2e6."]
    #[doc = ""]
    #[doc = "  \\param ptr        Pointer to coordinates to compress (length 3*ncoord)"]
    #[doc = "  \\param ncoord     Number of coordinate triplets in data"]
    #[doc = "  \\param precision  Scaling factor for lossy compression. If it is <=0,"]
    #[doc = "                    the default value of 1000.0 is used."]
    #[doc = "  \\param xfp        Handle to portably binary file"]
    #[doc = ""]
    #[doc = "  \\return           Number of coordinate triplets written."]
    #[doc = "                    IMPORTANT: Check that this is equal to ncoord - if it is"]
    #[doc = "                    negative, an error occured. This should not happen with"]
    #[doc = "\t   \t              normal data, but if your coordinates are NaN or very"]
    #[doc = "                    large (>1e6) it is not possible to use the compression."]
    #[doc = ""]
    #[doc = "  \\warning          The compression algorithm is not part of the XDR standard,"]
    #[doc = "                    and very complicated, so you will need this xdrfile module"]
    #[doc = "                    to read it later."]
    pub fn xdrfile_compress_coord_float(
        ptr: *mut ::std::os::raw::c_float,
        ncoord: ::std::os::raw::c_int,
        precision: ::std::os::raw::c_float,
        xfp: *mut XDRFILE,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    #[doc = " \\brief Decompress coordiates from XDR file to array of floats"]
    #[doc = ""]
    #[doc = "  This routine will decompress three-dimensional coordinate data previously"]
    #[doc = "  stored in an XDR file and store it in the specified array of floats."]
    #[doc = ""]
    #[doc = "  The precision used during the earlier compression is read from the file"]
    #[doc = "  and returned - you cannot adjust the accuracy at this stage."]
    #[doc = ""]
    #[doc = "  \\param ptr        Pointer to coordinates to compress (length>= 3*ncoord)"]
    #[doc = "  \\param ncoord     Max number of coordinate triplets to read on input, actual"]
    #[doc = "                    number of coordinate triplets read on return. If this"]
    #[doc = "                    is smaller than the number of coordinates in the frame an"]
    #[doc = "                    error will occur."]
    #[doc = "  \\param precision  The precision used in the previous compression will be"]
    #[doc = "                    written to this variable on return."]
    #[doc = "  \\param xfp        Handle to portably binary file"]
    #[doc = ""]
    #[doc = "  \\return           Number of coordinate triplets read. If this is negative,"]
    #[doc = "                    an error occured."]
    #[doc = ""]
    #[doc = "  \\warning          Since we cannot count on being able to set/get the"]
    #[doc = "                    position of large files (>2Gb), it is not possible to"]
    #[doc = "                    recover from errors by re-reading the frame if the"]
    #[doc = "                    storage area you provided was too small. To avoid this"]
    #[doc = "                    from happening, we recommend that you store the number of"]
    #[doc = "                    coordinates triplet as an integer either in a header or"]
    #[doc = "                    just before the compressed coordinate data, so you can"]
    #[doc = "                    read it first and allocated enough memory."]
    pub fn xdrfile_decompress_coord_float(
        ptr: *mut ::std::os::raw::c_float,
        ncoord: *mut ::std::os::raw::c_int,
        precision: *mut ::std::os::raw::c_float,
        xfp: *mut XDRFILE,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    #[doc = " \\brief Compress coordiates in a double array to XDR file"]
    #[doc = ""]
    #[doc = "  This routine will perform \\a lossy compression on the three-dimensional"]
    #[doc = "  coordinate data data specified and store it in the XDR file. Double will"]
    #[doc = "  NOT give you any extra precision since the coordinates are compressed. This"]
    #[doc = "  routine just avoids allocating a temporary array of floats."]
    #[doc = ""]
    #[doc = "  The lossy part of the compression consists of multiplying each"]
    #[doc = "  coordinate with the precision argument and then rounding to integers."]
    #[doc = "  We suggest a default value of 1000.0, which means you are guaranteed"]
    #[doc = "  three decimals of accuracy. The only limitation is that scaled coordinates"]
    #[doc = "  must still fit in an integer variable, so if the precision is 1000.0 the"]
    #[doc = "  coordinate magnitudes must be less than +-2e6."]
    #[doc = ""]
    #[doc = "  \\param ptr        Pointer to coordinates to compress (length 3*ncoord)"]
    #[doc = "  \\param ncoord     Number of coordinate triplets in data"]
    #[doc = "  \\param precision  Scaling factor for lossy compression. If it is <=0, the"]
    #[doc = "                    default value of 1000.0 is used."]
    #[doc = "  \\param xfp        Handle to portably binary file"]
    #[doc = ""]
    #[doc = "  \\return           Number of coordinate triplets written."]
    #[doc = "                    IMPORTANT: Check that this is equal to ncoord - if it is"]
    #[doc = "                    negative, an error occured. This should not happen with"]
    #[doc = "                    normal data, but if your coordinates are NaN or very"]
    #[doc = "                    large (>1e6) it is not possible to use the compression."]
    #[doc = ""]
    #[doc = "  \\warning          The compression algorithm is not part of the XDR standard,"]
    #[doc = "                    and very complicated, so you will need this xdrfile module"]
    #[doc = "                    to read it later."]
    pub fn xdrfile_compress_coord_double(
        ptr: *mut ::std::os::raw::c_double,
        ncoord: ::std::os::raw::c_int,
        precision: ::std::os::raw::c_double,
        xfp: *mut XDRFILE,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    #[doc = " \\brief Decompress coordiates from XDR file to array of doubles"]
    #[doc = ""]
    #[doc = "  This routine will decompress three-dimensional coordinate data previously"]
    #[doc = "  stored in an XDR file and store it in the specified array of doubles."]
    #[doc = "  Double will NOT give you any extra precision since the coordinates are"]
    #[doc = "  compressed. This routine just avoids allocating a temporary array of floats."]
    #[doc = ""]
    #[doc = "  The precision used during the earlier compression is read from the file"]
    #[doc = "  and returned - you cannot adjust the accuracy at this stage."]
    #[doc = ""]
    #[doc = "  \\param ptr        Pointer to coordinates to compress (length>= 3*ncoord)"]
    #[doc = "  \\param ncoord     Max number of coordinate triplets to read on input, actual"]
    #[doc = "                    number of coordinate triplets read on return. If this"]
    #[doc = "                    is smaller than the number of coordinates in the frame an"]
    #[doc = "                    error will occur."]
    #[doc = "  \\param precision  The precision used in the previous compression will be"]
    #[doc = "                    written to this variable on return."]
    #[doc = "  \\param xfp        Handle to portably binary file"]
    #[doc = ""]
    #[doc = "  \\return           Number of coordinate triplets read. If this is negative,"]
    #[doc = "                    an error occured."]
    #[doc = ""]
    #[doc = "  \\warning          Since we cannot count on being able to set/get the"]
    #[doc = "                    position of large files (>2Gb), it is not possible to"]
    #[doc = "                    recover from errors by re-reading the frame if the"]
    #[doc = "                    storage area you provided was too small. To avoid this"]
    #[doc = "                    from happening, we recommend that you store the number of"]
    #[doc = "                    coordinates triplet as an integer either in a header or"]
    #[doc = "                    just before the compressed coordinate data, so you can"]
    #[doc = "                    read it first and allocated enough memory."]
    pub fn xdrfile_decompress_coord_double(
        ptr: *mut ::std::os::raw::c_double,
        ncoord: *mut ::std::os::raw::c_int,
        precision: *mut ::std::os::raw::c_double,
        xfp: *mut XDRFILE,
    ) -> ::std::os::raw::c_int;
}
