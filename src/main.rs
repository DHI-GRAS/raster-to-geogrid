use std::path::Path;
use std::fs::File;
use std::io::{Read, Write, ErrorKind};
use std::io::Error as IOError;
use std::boxed::Box;

const YSIZE: usize = 64800;
const XSIZE: usize = 129600;
const TILE_DIM: usize = 10;

struct Dataset {
    data: Box<[u8]>,
    x_dim: usize,
    y_dim: usize,
}

impl Dataset {

    fn new(mut file: File, x_siz: usize, y_siz: usize) -> Result<Dataset, IOError> {
        let mut read_buf: Vec<u8> = vec![0; x_siz*y_siz];
        let bytes_read = file.read_to_end(&mut read_buf)?;
        let data_buf = read_buf.into_boxed_slice();

        if bytes_read < x_siz*y_siz {
            return Err(IOError::new(ErrorKind::InvalidInput,
                                    "Given dimensions do not match file"));
        }

        Ok(Dataset {
            data: data_buf,
            x_dim: x_siz,
            y_dim: y_siz,
           })
    }

    fn to_tiles(&self, n_dim: usize) -> Result<Vec<Tile>, IOError>  {
        if (self.x_dim % n_dim != 0) || (self.y_dim % n_dim != 0) {
            return Err(IOError::new(ErrorKind::InvalidInput,
                                    "Dataset dimensions must be divisible by n_dim"));
        }
        let tile_x = self.x_dim / n_dim;
        let tile_y = self.y_dim / n_dim;

        let mut xs: Vec<(usize, usize)> = Vec::new();
        let mut ys: Vec<(usize, usize)> = Vec::new();

        let mut x = 0;
        while x < self.x_dim {
            let tup = (x, x + tile_x);
            xs.push(tup);
            x += tile_x;
        }
        assert_eq!(n_dim, xs.len());

        let mut y = 0;
        while y < self.y_dim {
            let tup = (y, y + tile_y);
            ys.push(tup);
            y += tile_y;
        }
        assert_eq!(n_dim, ys.len());

        let mut tiles: Vec<Tile> = Vec::new();
        for x in &xs {
            for y in &ys {
                tiles.push(Tile {dataset: &self,
                                 x_range: *x,
                                 y_range: *y,});
            }
        }

        Ok(tiles)
    }
}


struct Tile<'a> {
    dataset: &'a Dataset,
    x_range: (usize, usize),
    y_range: (usize, usize),
}


impl<'a> Tile<'a> {

    fn get_fname(&self) -> String {
        let (x1, x2) = (self.x_range.0 + 1, self.x_range.1);
        let (y1, y2) = (self.y_range.0 + 1, self.y_range.1);

        format!("{:05}-{:05}.{:05}-{:05}", x1, x2, y1, y2)
    }

    fn write_to_file(&self, file: &mut File) -> Result<(), IOError> {
        let (x_min, x_max) = self.x_range;
        let (y_min, y_max) = self.y_range;
        let x_dim = self.dataset.x_dim;

        let idx = |y: usize| -> (usize, usize) {
            let min_idx = y*x_dim + x_min;
            let max_idx = min_idx + (x_max - x_min);
            (min_idx, max_idx)
        };

        for y in y_min..y_max {
            let (min, max) = idx(y);
            file.write_all(&self.dataset.data[min .. max])?;
        }

        file.flush()

    }

    fn to_file(&self, dir: &Path) -> Result<(), IOError> {
        if !dir.is_dir() {
            return Err(IOError::new(ErrorKind::InvalidInput, "dir must point to a folder"));
        }

        let name = self.get_fname();
        let out_path = dir.join(name);
        let mut out_file = File::create(out_path)?;

        self.write_to_file(&mut out_file)?;

        out_file.flush()
    }

}


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


fn convert_classes(data: &mut [u8]) -> () {
    let mut status = 0;
    for i in 0..data.len() {
        data[i] = cci_to_usgs(data[i]);
        if i % (data.len() / 10) == 0 {
            status += 10;
            println!("{}...", status);
        }
    }
}


fn split_files(mut in_file: &File, out_1: &Path, out_2: &Path) -> Result<(), IOError> {
    let mut outf1 = File::create(out_1)?;
    let mut outf2 = File::create(out_2)?;

    let mut read_buf: [u8; XSIZE/2] = [0; XSIZE/2];

    for y in 0..YSIZE {
        in_file.read_exact(&mut read_buf)?;
        outf1.write_all(&mut read_buf)?;
        in_file.read_exact(&mut read_buf)?;
        outf2.write_all(&mut read_buf)?;
        println!("Splitting row {} of {}", y, YSIZE);
    }
    outf1.flush()?;
    outf2.flush()
}


fn process_file(in_path: &Path, out_dir: &Path, n_tiles: usize) -> Result<(), IOError> {

    let in_file = File::open(in_path)?;
    println!("Reading data from file...");
    let mut ds = Dataset::new(in_file, XSIZE/2, YSIZE)?;

    println!("Converting classes to USGS...");
    convert_classes(&mut ds.data);

    println!("Tiling...");
    let tiles: Vec<Tile> = ds.to_tiles(n_tiles)?;

    for (i, tile) in tiles.iter().enumerate() {
        println!("Writing tile {} of {}...", i, tiles.len());
        tile.to_file(&out_dir)?;
    }

    Ok(())
}


fn main() {
    let fp = Path::new("datadir/raster/raster.dat");
    let f = File::open(fp).unwrap();

    let west_path = Path::new("datadir/raster/west.dat");
    let east_path = Path::new("datadir/raster/east.dat");
    split_files(&f, &west_path, &east_path).unwrap();

    let east_dir = Path::new("datadir/geogrid/east");
    let west_dir = Path::new("datadir/geogrid/west");
    process_file(&west_path, &west_dir, TILE_DIM).unwrap();
    process_file(&east_path, &east_dir, TILE_DIM).unwrap();

}
