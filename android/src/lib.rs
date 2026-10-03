//! OpenCut Android core — JNI bridge exposing the Rust time engine to Kotlin.
//!
//! Kotlin side declares (package `com.opencut.android.core`):
//! ```kotlin
//! object OpencutCore {
//!     external fun nativeAdd(a: Int, b: Int): Int
//!     external fun ticksPerSecond(): Long
//!     external fun version(): String
//!     external fun mediaTimeFromSeconds(seconds: Double): Long
//!     external fun mediaTimeToSeconds(ticks: Long): Double
//!     external fun mediaTimeFromFrame(frame: Long, numerator: Int, denominator: Int): Long
//!     external fun mediaTimeToFrameRound(ticks: Long, numerator: Int, denominator: Int): Long
//!     external fun formatTimecode(ticks: Long, format: Int, numerator: Int, denominator: Int): String
//!     external fun parseTimecode(timeCode: String, format: Int, numerator: Int, denominator: Int): Long
//! }
//! ```
//! Timecode formats: 0=MM:SS, 1=HH:MM:SS, 2=HH:MM:SS:CS, 3=HH:MM:SS:FF

use jni::objects::{JClass, JString};
use jni::sys::{jdouble, jint, jlong, jstring};
use jni::JNIEnv;
use time::{
    FormatTimecodeOptions, FrameRate, MediaTime, MediaTimeFromSecondsOptions,
    MediaTimeToSecondsOptions, ParseTimecodeOptions, TimeCodeFormat, TICKS_PER_SECOND,
    format_timecode, media_time_from_seconds, media_time_to_seconds, parse_timecode,
};

// Ensure render-engine crates are linked into the cdylib (not dead-code-eliminated).
use compositor::Compositor;
use effects::EffectPipeline;
use gpu::GpuContext;
use masks::MaskFeatherPipeline;
use std::mem::size_of;

const ENGINE_MODULES: &str = "time,compositor,effects,masks,gpu";

fn rate(numerator: jint, denominator: jint) -> FrameRate {
    FrameRate::new(numerator.max(1) as u32, denominator.max(1) as u32)
}

fn fmt_code(format: jint) -> TimeCodeFormat {
    match format {
        0 => TimeCodeFormat::MmSs,
        1 => TimeCodeFormat::HhMmSs,
        2 => TimeCodeFormat::HhMmSsCs,
        _ => TimeCodeFormat::HhMmSsFf,
    }
}

/// Smoke-test symbol: proves the .so is loaded and callable.
#[no_mangle]
pub extern "system" fn Java_com_opencut_android_core_OpencutCore_nativeAdd(
    _env: JNIEnv,
    _class: JClass,
    a: jint,
    b: jint,
) -> jint {
    a.wrapping_add(b)
}

#[no_mangle]
pub extern "system" fn Java_com_opencut_android_core_OpencutCore_ticksPerSecond(
    _env: JNIEnv,
    _class: JClass,
) -> jlong {
    TICKS_PER_SECOND as jlong
}

#[no_mangle]
pub extern "system" fn Java_com_opencut_android_core_OpencutCore_version(
    mut env: JNIEnv,
    _class: JClass,
) -> jstring {
    env.new_string("opencut-rust-core-0.2.0 (route-b, render-engine linked)")
        .map(|s| s.into_raw())
        .unwrap_or(std::ptr::null_mut())
}

/// Reports which OpenCut classic Rust modules are compiled into this core.
/// Also serves as a link-time anchor so compositor/effects/masks/gpu
/// are never optimized out of the cdylib.
#[no_mangle]
pub extern "system" fn Java_com_opencut_android_core_OpencutCore_engineModules(
    mut env: JNIEnv,
    _class: JClass,
) -> jstring {
    let _ = (size_of::<Compositor>(), size_of::<EffectPipeline>(), size_of::<GpuContext>(), size_of::<MaskFeatherPipeline>());
    env.new_string(ENGINE_MODULES)
        .map(|s| s.into_raw())
        .unwrap_or(std::ptr::null_mut())
}

#[no_mangle]
pub extern "system" fn Java_com_opencut_android_core_OpencutCore_mediaTimeFromSeconds(
    _env: JNIEnv,
    _class: JClass,
    seconds: jdouble,
) -> jlong {
    media_time_from_seconds(MediaTimeFromSecondsOptions { seconds })
        .map(|t| t.as_ticks())
        .unwrap_or(0)
}

#[no_mangle]
pub extern "system" fn Java_com_opencut_android_core_OpencutCore_mediaTimeToSeconds(
    _env: JNIEnv,
    _class: JClass,
    ticks: jlong,
) -> jdouble {
    media_time_to_seconds(MediaTimeToSecondsOptions {
        time: MediaTime::from_ticks(ticks),
    })
}

#[no_mangle]
pub extern "system" fn Java_com_opencut_android_core_OpencutCore_mediaTimeFromFrame(
    _env: JNIEnv,
    _class: JClass,
    frame: jlong,
    numerator: jint,
    denominator: jint,
) -> jlong {
    MediaTime::from_frame(frame, rate(numerator, denominator))
        .map(|t| t.as_ticks())
        .unwrap_or(0)
}

#[no_mangle]
pub extern "system" fn Java_com_opencut_android_core_OpencutCore_mediaTimeToFrameRound(
    _env: JNIEnv,
    _class: JClass,
    ticks: jlong,
    numerator: jint,
    denominator: jint,
) -> jlong {
    MediaTime::from_ticks(ticks)
        .to_frame_round(rate(numerator, denominator))
        .unwrap_or(0)
}

#[no_mangle]
pub extern "system" fn Java_com_opencut_android_core_OpencutCore_formatTimecode(
    mut env: JNIEnv,
    _class: JClass,
    ticks: jlong,
    format: jint,
    numerator: jint,
    denominator: jint,
) -> jstring {
    let out = format_timecode(FormatTimecodeOptions {
        time: MediaTime::from_ticks(ticks),
        format: Some(fmt_code(format)),
        rate: Some(rate(numerator, denominator)),
    })
    .unwrap_or_default();
    env.new_string(out)
        .map(|s| s.into_raw())
        .unwrap_or(std::ptr::null_mut())
}

#[no_mangle]
pub extern "system" fn Java_com_opencut_android_core_OpencutCore_parseTimecode(
    mut env: JNIEnv,
    _class: JClass,
    input: JString,
    format: jint,
    numerator: jint,
    denominator: jint,
) -> jlong {
    let s: String = env
        .get_string(&input)
        .map(|j| j.into())
        .unwrap_or_default();
    parse_timecode(ParseTimecodeOptions {
        time_code: s,
        format: Some(fmt_code(format)),
        rate: Some(rate(numerator, denominator)),
    })
    .map(|t| t.as_ticks())
    .unwrap_or(0)
}
