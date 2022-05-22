// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use std::sync::Arc;

use tirocks::{env::Env, db::DefaultCfOnlyBuilder, OpenOptions, option::{WriteOptions, ReadOptions, FlushOptions}};

use super::tempdir_with_prefix;

#[test]
fn test_ctr_encrypted_env() {
    let test_cipher_texts: &[&[u8]] = &[
        &[16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1],
        &[8, 7, 6, 5, 4, 3, 2, 1],
    ];
    for ciphertext in test_cipher_texts {
        let base_env = Env::with_mem(Env::default());
        test_ctr_encrypted_env_impl(Arc::new(
            Env::with_ctr_encrypted(base_env, ciphertext).unwrap()
        ));
    }
    for ciphertext in test_cipher_texts {
        test_ctr_encrypted_env_impl(Arc::new(
            Env::with_ctr_encrypted(Env::default(), ciphertext).unwrap()
        ));
    }
}

fn test_ctr_encrypted_env_impl(encrypted_env: Arc<Env>) {
    let path = tempdir_with_prefix("_rust_rocksdb_cryption_env");
    let mut builder = DefaultCfOnlyBuilder::default();
    builder.set_create_if_missing(true).options_mut().set_env(encrypted_env);
    let db = builder.open(path.path()).unwrap();
    let write_opt = WriteOptions::default();
    let read_opt = ReadOptions::default();
    let cf = db.default_cf();

    let samples = vec![
        (b"key1".to_vec(), b"value1".to_vec()),
        (b"key2".to_vec(), b"value2".to_vec()),
        (b"key3".to_vec(), b"value3".to_vec()),
        (b"key4".to_vec(), b"value4".to_vec()),
    ];
    for (k, v) in &samples {
        db.put(&write_opt, cf, k, v).unwrap();

        // check value
        assert_eq!(Ok(Some(v.to_vec())), db.get(&read_opt, cf, k));
    }

    // flush to sst fil
    db.flush(FlushOptions::default().set_wait(true), cf).unwrap();

    // check value in db
    for (k, v) in &samples {
        assert_eq!(Ok(Some(v.to_vec())), db.get(&read_opt, cf, k));
    }

    // close db and open again.
    drop(db);
    
    let db = builder.open(path.path()).unwrap();
    let cf = db.default_cf();

    // check value in db again
    for &(ref k, ref v) in &samples {
        assert_eq!(Ok(Some(v.to_vec())), db.get(&read_opt, cf, k));
    }
}
