extern crate apriltag;
extern crate nalgebra;
extern crate nokhwa;

use std::{
    io::{stdout, BufWriter, Write},
    time::Instant,
};

use image::{imageops::FilterType, DynamicImage, GenericImageView, ImageFormat, Rgb, RgbImage};
use imageproc::{haar::HaarFeature, contours::BorderType, contrast::{adaptive_threshold, threshold}};
use nalgebra::{DMatrix, Matrix, MatrixViewMut6};
use nokhwa::{
    pixel_format::RgbFormat,
    utils::{RequestedFormat, RequestedFormatType},
    CallbackCamera,
};

struct Quad {
    corners: [(f32, f32); 4],
    reversed_border: bool,
    // matd_t *H, Hinv;
}
struct ApriltagFamily {
    ncodes: u32,
    codes: &'static [u64],

    width_at_border: i32,
    total_width: i32,
    reversed_border: bool,
}

pub fn load() -> DynamicImage {
    //
    image::open("capture291.png").unwrap()
}

pub fn scale(img: DynamicImage) -> DynamicImage {
    //img.resize(300, 400, FilterType::Lanczos3)
    img
}

pub fn prep(img: DynamicImage) -> DynamicImage {
    //img.blur(1.0).adjust_contrast(40.0).grayscale()
    //img.adjust_contrast(40.0).grayscale()
    img.grayscale()
}

pub fn cam_geom_stuff() {
    use cam_geom::*;
    use nalgebra::{Matrix2x3, Unit, Vector3};

    // Create two points in the world coordinate frame.
    let world_coords = Points::new(Matrix2x3::new(
        1.0, 0.0, 0.0, // point 1
        0.0, 1.0, 0.0, // point 2
    ));

    // perepective parameters - focal length of 100, no skew, pixel center at (640,480)
    let intrinsics = IntrinsicParametersPerspective::from(PerspectiveParams {
        fx: 100.0,
        fy: 100.0,
        skew: 0.0,
        cx: 640.0,
        cy: 480.0,
    });

    // Set extrinsic parameters - camera at (10,0,0), looing at (0,0,0), up (0,0,1)
    let camcenter = Vector3::new(10.0, 0.0, 0.0);
    let lookat = Vector3::new(0.0, 0.0, 0.0);
    let up = Unit::new_normalize(Vector3::new(0.0, 0.0, 1.0));
    let pose = ExtrinsicParameters::from_view(&camcenter, &lookat, &up);

    // Create a `Camera` with both intrinsic and extrinsic parameters.
    let camera = Camera::new(intrinsics, pose);

    // Project the original 3D coordinates to 2D pixel coordinates.
    let pixel_coords = camera.world_to_pixel(&world_coords);

    // Print the results.
    for i in 0..world_coords.data.nrows() {
        let wc = world_coords.data.row(i);
        let pix = pixel_coords.data.row(i);
        println!("{} -> {}", wc, pix);
    }
}

pub fn detect(img: DynamicImage) {
    // let img = Image::from_pnm_file("apriltag.pnm").unwrap();

    // let st = Instant::now();

    let mut iimg = img.to_luma8();
    iimg = threshold(&iimg, 220);
    iimg = adaptive_threshold(&iimg, 24);
    let mut fin = img.to_rgb8();

    for c in imageproc::contours::find_contours_with_threshold(&iimg, 90) {
        if c.border_type == BorderType::Outer {
            for p in c.points {
                fin.put_pixel(p.x, p.y, Rgb([255, 0, 0]));
            }
        } else {
            for p in c.points {
                fin.put_pixel(p.x, p.y, Rgb([0, 0, 255]));
            }
        }
    }

    for c in imageproc::corners::corners_fast9(&iimg, 90) {
        fin.put_pixel(c.x, c.y, Rgb([0, 255, 255]));
    }

    // Canny edge detection:
    //
    //    imageproc::edges::canny(&iimg, 250.0, 300.0)
    //      .save("/tmp/canny.png")
    //      .unwrap();

    // imageproc::haar::draw_haar_feature(&iimg, HaarFeature::evaluate(, ))
    //     .save("/tmp/corners.png")
    //     .unwrap();

    /*
    let mut marks: Vec<((u32, u32), (u32, u32))> = Vec::new();
    let (mut topmost, mut bottommost, mut leftmost, mut rightmost) =
        (u32::MAX, 0u32, u32::MAX, 0u32);

    let img_grey = img.to_luma8();
    for (row, r) in img_grey.enumerate_rows() {
        let (mut first, mut last) = (None, None);
        for (x, y, px) in r {
            if px.0[0] < 2 {
                {
                    if x < leftmost {
                        leftmost = x;
                    }
                    if x > rightmost {
                        rightmost = x;
                    }

                    if y < topmost {
                        topmost = y;
                    }
                    if y > bottommost {
                        bottommost = y;
                    }
                }
                if first.is_none() {
                    first = Some(x);
                } else {
                    last = Some(x);
                }
            }
        }
        if let (Some(f), Some(l)) = (first, last) {
            marks.push(((f, row), (l, row)));
        } else {
            if !marks.is_empty() {
                break;
            }
        }
    }

    // let (mut tl, mut tr, mut bl, mut br) = (
    //     (0u32, 0u32),
    //     (u32::MAX, 0u32),
    //     (0u32, u32::MAX),
    //     (u32::MAX, u32::MAX),
    // );

    // let img_grey = img.to_luma8();
    // for (row, r) in img_grey.enumerate_rows() {
    //     let (mut first, mut last) = (None, None);
    //     for (x, y, px) in r {
    //         if px.0[0] < 2 {
    //             {
    //                 // if x < tl.0 || y < tl.1 {
    //                 //     tl = (x, y);
    //                 // } else if x > tr.0 || y < tr.1 {
    //                 //     tr = (x, y);
    //                 // } else if x < bl.0 || y > bl.1 {
    //                 //     bl = (x, y);
    //                 // } else if x > br.0 || y > br.1 {
    //                 //     br = (x, y);
    //                 // }

    //                 // if y > tl.1 && y > tr.1 {
    //                 //   if x < br.0 {
    //                 //     bl = (x, y);
    //                 //   }
    //                 //   if x > bl.0 {
    //                 //     br = (x, y);
    //                 //   }
    //                 // } else if y < bl.1 && y < br.1 {
    //                 //   if x < tr.0 {
    //                 //     tl = (x, y);
    //                 //     //bl.1 = y;
    //                 //   }
    //                 //   if x > tl.0 {
    //                 //     tr = (x, y);
    //                 //     //br.1 = y;
    //                 //   }
    //                 // }

    //                 dbg!(tl, tr, bl, br, x, y);
    //             }
    //             if first.is_none() {
    //                 first = Some(x);
    //             } else {
    //                 last = Some(x);
    //             }
    //         }
    //     }
    //     if let (Some(f), Some(l)) = (first, last) {
    //         marks.push(((f, row), (l, row)));
    //     } else {
    //         if !marks.is_empty() {
    //             break;
    //         }
    //     }
    // }

    // println!("{}", st.elapsed().as_millis());

    let mut iimg = img.to_rgb8();
    for ((fx, fy), (lx, ly)) in marks.clone() {
        iimg.put_pixel(fx, fy, Rgb([0, 0, 255]));
        iimg.put_pixel(lx, ly, Rgb([0, 0, 255]));
    }
    // for (x, y) in [tl, tr, bl, br] {
    //   dbg!(x, y);
    //     iimg.put_pixel(x, y, Rgb([255,0,0]));
    // }

    let crop = img.crop_imm(
        leftmost,
        topmost,
        rightmost - leftmost,
        bottommost - topmost,
    );
    crop.save("/tmp/crop.png").unwrap();

    let mat = DMatrix::from_row_iterator(
        crop.height().try_into().unwrap(),
        crop.width().try_into().unwrap(),
        crop.to_luma8()
            .pixels()
            .map(|x| if x.0[0] < 2 { 0u8 } else { 255u8 }),
    );
    for row in mat.clone().row_iter() {
        // println!("{}", row.sum());
        for col in row.iter() {
            print!("{}", if *col == 0 { "#" } else { " " });
        }
        println!();
    }
    for row in mat
        .columns_with_step(mat.ncols() / 6, 6, mat.ncols() / 7)
        .row_iter()
    {
        for col in row.iter() {
            print!("{}", if *col == 0 { "#" } else { " " });
        }
        println!();
    }
    for col in mat
        .columns_with_step(mat.ncols() / 6, 6, mat.ncols() / 7)
        .column_iter()
    {
        // println!("{} ({})", col.sum(), col.sum() / 6 / 2);
    }
    */

    // cam_geom_stuff();

    // println!("{}", st.elapsed().as_millis());

    iimg.save("/tmp/work.png").unwrap();
    fin.save("/tmp/out.png").unwrap();

    // println!(
    //     "fir:{:?} sec:{:?}",
    //     marks.get(0).unwrap().0,
    //     marks.get(1).unwrap().0,
    // );

    /*
    for d in apriltag::detector::Detector::builder()
        .add_family_bits("tag16h5".parse::<Family>().unwrap(), 0)
        .build()
        .unwrap()
        .detect(&img)
    {
        println!(
            "{}\n{:?}\n{:?}\n{:?}",
            d.id(),
            d.center(),
            d.corners(),
            d.homography()
        );
    }
    */
}

/*
fn main() {
    let mut buf = Vec::new();

    let backend = nokhwa::native_api_backend().expect("failed to get native api backend");
    let devs = nokhwa::query(backend).unwrap();
    eprintln!("{backend:?}");
    for cam in devs.clone() {
        eprintln!("{}: {}", cam.index(), cam.human_name());
    }

    let mut cam = CallbackCamera::new(
        devs.last().unwrap().index().clone(),
        RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestFrameRate),
        |buf| {
            buf.decode_image::<RgbFormat>().unwrap();
        },
    )
    .unwrap();

    eprintln!(
        "{:?} {}",
        cam.resolution().unwrap(),
        cam.frame_rate().unwrap()
    );

    cam.open_stream().unwrap();
    for i in 0..20 {
        let frame = cam.poll_frame().unwrap();
        frame
            .decode_image::<RgbFormat>()
            .unwrap()
            .save(format!("out{i}.png"))
            .unwrap();
        eprintln!("{:?}", frame.resolution());
        //buf.extend_from_slice(frame.buffer());
    }
    cam.stop_stream().unwrap();

    stdout().write_all(buf.as_slice()).unwrap();
}
*/
