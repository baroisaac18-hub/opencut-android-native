# opencut-android-native

Android native build of the [OpenCut](https://github.com/OpenCut-app/OpenCut) Rust core (MIT licensed).

Route B: compile the Rust `time` engine into Android `.so` shared libraries via GitHub Actions
([cargo-ndk](https://github.com/bbqsrc/cargo-ndk) + Android NDK), then bridge to Kotlin through JNI.

## Layout

- `android/` — JNI bridge crate (`opencut-android-core`), package `com.opencut.android.core.OpencutCore`
- `rust/crates/` — upstream OpenCut Rust crates (`time`, `bridge`, `compositor`, `effects`, `gpu`, `masks`)

## Build

CI workflow `.github/workflows/android-core.yml` produces `.so` for
`arm64-v8a`, `armeabi-v7a`, `x86_64`, `x86` on every push to `main` (also `workflow_dispatch`).

Artifact: `opencut-android-core` → `jniLibs/<abi>/libopencut_android_core.so`

## JNI surface

| Kotlin external fn | Rust symbol |
|---|---|
| `nativeAdd(a, b)` | `Java_com_opencut_android_core_OpencutCore_nativeAdd` |
| `ticksPerSecond()` | `..._ticksPerSecond` |
| `version()` | `..._version` |
| `mediaTimeFromSeconds(seconds)` | `..._mediaTimeFromSeconds` |
| `mediaTimeToSeconds(ticks)` | `..._mediaTimeToSeconds` |
| `mediaTimeFromFrame(frame, num, den)` | `..._mediaTimeFromFrame` |
| `mediaTimeToFrameRound(ticks, num, den)` | `..._mediaTimeToFrameRound` |
| `formatTimecode(ticks, format, num, den)` | `..._formatTimecode` |
| `parseTimecode(str, format, num, den)` | `..._parseTimecode` |

Timecode formats: `0=MM:SS 1=HH:MM:SS 2=HH:MM:SS:CS 3=HH:MM:SS:FF`
