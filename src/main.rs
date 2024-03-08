use apriltag_rs::{detect, load, prep, scale};
use cam_geom::Camera;
use image::SubImage;
use nokhwa::{
    nokhwa_initialize,
    pixel_format::RgbFormat,
    utils::{
        ApiBackend, CameraFormat, CameraInfo, FrameFormat, RequestedFormat, RequestedFormatType,
        Resolution,
    },
    FormatDecoder,
};
use std::sync::{mpsc, Arc};

fn main() {
    //nokhwa_crap();
    detect(prep(scale(load())));
}

fn nokhwa_crap() {
    let cams = nokhwa::query(ApiBackend::Video4Linux).unwrap();

    // Crappy fix for this v4l thing until I get the time to submit a patch to nokhwa upstream
    #[cfg(target_os = "linux")]
    let cams = cams.iter().rev().step_by(2).collect::<Vec<&CameraInfo>>();

    let (tx, rx) = mpsc::sync_channel(4096);

    for cam in cams {
        let index = cam.index().clone();
        let tx = tx.clone();
        std::thread::spawn(move || {
            // println!(
            //     "name: {}\ndesc: {}\nmisc: {}\nindex: {}",
            //     cam.human_name(),
            //     cam.description(),
            //     cam.misc(),
            //     cam.index()
            // );

            let rs_id = 0;

            let mut nkhw =
                nokhwa::CallbackCamera::new(
                    index,
                    RequestedFormat::new::<RgbFormat>(RequestedFormatType::Exact(
                        CameraFormat::new(Resolution::new(640, 480), FrameFormat::MJPEG, 30),
                    )),
                    move |buf| tx.send((rs_id, buf)).unwrap(),
                )
                .unwrap();
            // for fmt in nkhw.compatible_camera_formats().unwrap() {
            //     println!("fmt: {fmt}");
            // }

            nkhw.open_stream().unwrap();

            loop {
                let _ = nkhw.poll_frame();
            }
        });
        println!("{}", cam.index());
    }

    loop {
        println!("pre-recv");

        let (id, fr) = rx.recv().unwrap();

        println!("post-recv");

        let img = fr.decode_image::<RgbFormat>().unwrap();
        //imageproc::contrast::threshold(, 90);
        // .save(format!("cap{id}.png"))
        // .unwrap();
        println!("a");
    }
}
