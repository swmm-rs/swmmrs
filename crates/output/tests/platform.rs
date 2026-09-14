use swmm_output::OutputReader;

fn assert_send_sync<T: Send + Sync>() {}

const _: fn() = assert_send_sync::<OutputReader>;

// macOS rejects this path during file creation, before OutputReader can exercise it.
#[cfg(target_os = "linux")]
#[test]
fn retains_non_utf8_source_path() {
    use std::ffi::OsString;
    use std::fs;
    use std::os::unix::ffi::{OsStrExt, OsStringExt};
    use std::path::{Path, PathBuf};

    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/legacy/Model.out");
    let directory = tempfile::tempdir().expect("create temporary directory");
    let mut source_bytes = directory.path().as_os_str().as_bytes().to_vec();
    source_bytes.extend_from_slice(b"/output-\xff.out");
    let source_path = PathBuf::from(OsString::from_vec(source_bytes));

    fs::copy(fixture, &source_path).expect("copy fixture to non-UTF-8 path");
    let reader = OutputReader::open(&source_path).expect("open non-UTF-8 source path");

    assert_eq!(reader.source_path(), source_path);
    assert!(reader.source_path().as_os_str().as_bytes().contains(&0xff));
}
