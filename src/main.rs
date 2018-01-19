use std::path::Path;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::io::Result;

const YSIZE: usize = 64800;
const XSIZE: usize = 129600;

fn cci_to_usgs(x: u8) -> u8 {
    match x {
        0 => 0,
        10 => 102,
        11 => 102,
        12 => 115,
        20 => 103,
        30 => 106,
        40 => 106,
        50 => 113,
        60 => 111,
        61 => 111,
        62 => 111,
        70 => 114,
        71 => 114,
        72 => 114,
        80 => 112,
        81 => 112,
        82 => 112,
        90 => 115,
        100 => 109,
        110 => 109,
        120 => 108,
        121 => 108,
        122 => 108,
        130 => 107,
        140 => 123,
        150 => 119,
        151 => 119,
        152 => 119,
        153 => 119,
        160 => 118,
        170 => 118,
        180 => 117,
        190 => 101,
        200 => 119,
        201 => 119,
        202 => 119,
        210 => 116,
        220 => 124,
        _ => panic!("Unexpected val in data"),
    }
}

fn convert_file(mut in_file: &File, out_path: &Path) -> Result<()> {
    let mut out_file = File::create(out_path)?;

    let mut data: Box<[u8]> = vec![0; XSIZE*YSIZE/2].into_boxed_slice();
    in_file.read_exact(&mut data)?;

    let mut status = 0;

    for i in 0..data.len() {
        data[i] = cci_to_usgs(data[i]);
        if i % (data.len() / 10) == 0 {
            status += 10;
            println!("{}...", status);
        }
    }

    out_file.write_all(&mut data)?;

    Ok(())
}

fn split_files(mut in_file: &File, out_1: &Path, out_2: &Path) -> Result<()> {
    let mut outf1 = File::create(out_1)?;
    let mut outf2 = File::create(out_2)?;

    let mut read_buf: [u8; XSIZE/2] = [0; XSIZE/2];

    for y in 0..YSIZE {
        in_file.read_exact(&mut read_buf)?;
        outf1.write_all(&mut read_buf)?;
        in_file.read_exact(&mut read_buf)?;
        outf2.write_all(&mut read_buf)?;
        println!("Row {}", y);
    }
    Ok(())
}

fn main() {
    let fp = Path::new("C:/delivery_data/innowind/raster/raster.dat");
    let f = File::open(fp).unwrap();

    let out_path_1 = Path::new("C:/delivery_data/innowind/raster/west.dat");
    let out_path_2 = Path::new("C:/delivery_data/innowind/raster/east.dat");

    split_files(&f, &out_path_1, &out_path_2).unwrap();

    let mut west_file = File::open(out_path_1).unwrap();
    let mut east_file = File::open(out_path_2).unwrap();

    let west_path = Path::new("C:/delivery_data/innowind/geogrid/west/00001-64800.00001-64800");
    let east_path = Path::new("C:/delivery_data/innowind/geogrid/east/00001-64800.00001-64800");

    convert_file(&mut west_file, &west_path).unwrap();
    convert_file(&mut east_file, &east_path).unwrap();

}
