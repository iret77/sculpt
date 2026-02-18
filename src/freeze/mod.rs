use std::fs;
use std::path::Path;

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use sha2::Digest;

use crate::ir::{to_canonical_string, IrModule};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockFile {
  pub provider: String,
  pub model: String,
  pub target: String,
  pub ir_hash: String,
  pub target_ir: Value,
}

pub fn create_lock(
  ir: &IrModule,
  provider: &str,
  target: &str,
  target_ir: &Value,
  model: &str,
) -> Result<LockFile> {
  let ir_hash = compute_ir_hash(ir)?;
  Ok(LockFile {
    provider: provider.to_string(),
    model: model.to_string(),
    target: target.to_string(),
    ir_hash,
    target_ir: target_ir.clone(),
  })
}

pub fn write_lock(path: &Path, lock: &LockFile) -> Result<()> {
  let json = serde_json::to_string_pretty(lock)?;
  fs::write(path, json)?;
  Ok(())
}

pub fn read_lock(path: &Path) -> Result<LockFile> {
  let data = fs::read_to_string(path)?;
  Ok(serde_json::from_str(&data)?)
}

pub fn verify_lock(ir: &IrModule, lock: &LockFile) -> Result<()> {
  let hash = compute_ir_hash(ir)?;
  if hash != lock.ir_hash {
    bail!("IR hash mismatch: lock {}, current {}", lock.ir_hash, hash);
  }
  Ok(())
}


pub fn compute_ir_hash(ir: &IrModule) -> Result<String> {
  let canonical = to_canonical_string(ir)?;
  let mut hasher = sha2::Sha256::new();
  hasher.update(canonical.as_bytes());
  Ok(format!("{:x}", hasher.finalize()))
}

// nd outputs are now part of target IR generation
