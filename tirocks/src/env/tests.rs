use super::*;
use std::fs::File;
use std::io::{Read, Write};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

#[test]
fn test_env_check() {
    let env = Env::default();
    env.file_exists("Cargo.toml").unwrap();
    assert!(env.file_exists("test.file").unwrap_err().path_not_exist());
}

#[test]
fn test_mem_check() {
    let env = Env::with_mem(Env::default());
    assert!(env.file_exists("Cargo.toml").unwrap_err().path_not_exist());
}

struct TestDrop {
    called: Arc<AtomicUsize>,
}

impl Drop for TestDrop {
    fn drop(&mut self) {
        self.called.fetch_add(1, Ordering::SeqCst);
    }
}

struct TestFileSystemInspector {
    pub refill_bytes: usize,
    pub read_called: usize,
    pub _drop: Option<TestDrop>,
}

impl Default for TestFileSystemInspector {
    fn default() -> Self {
        TestFileSystemInspector {
            refill_bytes: 0,
            read_called: 0,
            _drop: None,
        }
    }
}

impl FileSystemInspector for Arc<Mutex<TestFileSystemInspector>> {
    fn read(&self, len: usize) -> Result<usize> {
        let mut inner = self.lock().unwrap();
        inner.read_called += 1;
        if len <= inner.refill_bytes {
            Ok(len)
        } else {
            Err(Status::with_error(
                Code::kIncomplete,
                "request exceeds refill bytes",
            ))
        }
    }
}

#[test]
fn test_create_and_destroy_inspector() {
    let drop_called = Arc::new(AtomicUsize::new(0));
    let fs_inspector = Arc::new(Mutex::new(TestFileSystemInspector {
        _drop: Some(TestDrop {
            called: drop_called.clone(),
        }),
        ..Default::default()
    }));
    let env = Env::with_file_system_inspected(Env::default(), fs_inspector).unwrap();
    assert_eq!(0, drop_called.load(Ordering::SeqCst));
    drop(env);
    assert_eq!(1, drop_called.load(Ordering::SeqCst));
}

#[test]
fn test_inspected_operation() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path()).unwrap();
    let file_path = dir.path().join("test.txt");
    File::create(&file_path)
        .unwrap()
        .write_all(&[0; 16])
        .unwrap();
    let fs_inspector = Arc::new(Mutex::new(TestFileSystemInspector {
        refill_bytes: 4,
        ..Default::default()
    }));
    let env = Env::with_file_system_inspected(Env::default(), fs_inspector.clone()).unwrap();
    let mut f = env
        .new_sequential_file(file_path.to_str().unwrap(), EnvOptions::default())
        .unwrap();
    f.read_exact(&mut [0; 2]).unwrap();
    let e = f.read_exact(&mut [0; 8]).unwrap_err();
    let msg = format!("{}", e);
    assert!(msg.contains("refill bytes"), "{}", msg);
    let record = fs_inspector.lock().unwrap();
    assert_eq!(2, record.read_called);
}
