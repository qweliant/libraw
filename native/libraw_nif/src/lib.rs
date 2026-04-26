mod error;
mod raw;

use rustler::{Encoder, Env, OwnedBinary, Term};
use rustler::types::atom;
use rustler::types::map::map_new;

use error::{encode_error, LibRawError};
use raw::RawHandle;

// Atoms used for map keys returned to Elixir.
mod atoms {
    rustler::atoms! {
        ok,
        error,
        pixels,
        width,
        height,
        colors,
        bps,
        camera_make,
        camera_model,
        captured_at,
        iso,
        shutter,
        aperture,
        orientation,
    }
}

/// Decode a RAW image file and return processed pixel data as an Elixir map.
///
/// Runs on a Dirty CPU Scheduler because decoding takes 100–500 ms.
#[rustler::nif(schedule = "DirtyCpu")]
fn decode_nif<'a>(
    env: Env<'a>,
    path: String,
    use_camera_wb: i32,
    no_auto_bright: i32,
    output_bps: i32,
    gamma0: f64,
    gamma1: f64,
) -> Term<'a> {
    let result = decode_inner(env, path, use_camera_wb, no_auto_bright, output_bps, gamma0, gamma1);
    match result {
        Ok(term) => (atoms::ok(), term).encode(env),
        Err(e) => encode_error(env, e),
    }
}

fn decode_inner<'a>(
    env: Env<'a>,
    path: String,
    use_camera_wb: i32,
    no_auto_bright: i32,
    output_bps: i32,
    gamma0: f64,
    gamma1: f64,
) -> Result<Term<'a>, LibRawError> {
    let handle = RawHandle::new()?;
    handle.open(&path)?;
    handle.set_params(use_camera_wb, no_auto_bright, output_bps, gamma0, gamma1);
    handle.unpack()?;
    handle.dcraw_process()?;

    let img = handle.make_mem_image()?;

    let pixel_bytes = img.pixel_bytes();
    let mut bin = OwnedBinary::new(pixel_bytes.len())
        .ok_or(LibRawError::NullPointer)?;
    bin.as_mut_slice().copy_from_slice(&pixel_bytes);

    let map = map_new(env);
    let map = map.map_put(atoms::pixels().encode(env), bin.release(env).encode(env)).unwrap();
    let map = map.map_put(atoms::width().encode(env),  img.width().encode(env)).unwrap();
    let map = map.map_put(atoms::height().encode(env), img.height().encode(env)).unwrap();
    let map = map.map_put(atoms::colors().encode(env), img.colors().encode(env)).unwrap();
    let map = map.map_put(atoms::bps().encode(env),    img.bits().encode(env)).unwrap();

    Ok(map)
}

/// Read EXIF metadata from a RAW file without full decoding.
///
/// Runs on a Dirty CPU Scheduler because file I/O can block.
#[rustler::nif(schedule = "DirtyCpu")]
fn metadata_nif<'a>(env: Env<'a>, path: String) -> Term<'a> {
    let result = metadata_inner(env, path);
    match result {
        Ok(term) => (atoms::ok(), term).encode(env),
        Err(e) => encode_error(env, e),
    }
}

fn metadata_inner<'a>(env: Env<'a>, path: String) -> Result<Term<'a>, LibRawError> {
    let handle = RawHandle::new()?;
    handle.open(&path)?;

    // No unpack/process needed for metadata; libraw populates idata/other/sizes
    // during open_file.

    let ts = handle.timestamp();
    let captured_at: Term = if ts == 0 {
        atom::nil().encode(env)
    } else {
        ts.encode(env)
    };

    let map = map_new(env);
    let map = map.map_put(atoms::camera_make().encode(env),  handle.make().encode(env)).unwrap();
    let map = map.map_put(atoms::camera_model().encode(env), handle.model().encode(env)).unwrap();
    let map = map.map_put(atoms::captured_at().encode(env),  captured_at).unwrap();
    let map = map.map_put(atoms::iso().encode(env),          handle.iso().encode(env)).unwrap();
    let map = map.map_put(atoms::shutter().encode(env),      handle.shutter().encode(env)).unwrap();
    let map = map.map_put(atoms::aperture().encode(env),     handle.aperture().encode(env)).unwrap();
    let map = map.map_put(atoms::orientation().encode(env),  handle.flip().encode(env)).unwrap();

    Ok(map)
}

rustler::init!("Elixir.LibRaw.NIF", [decode_nif, metadata_nif]);
