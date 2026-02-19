use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FallbackMode {
  Fail,
  Stub,
  Replay,
}

#[derive(Debug, Clone)]
pub struct ConvergenceControls {
  pub nd_budget: Option<i32>,
  pub confidence: Option<f64>,
  pub max_iterations: u32,
  pub fallback: FallbackMode,
}

impl ConvergenceControls {
  pub fn from_meta(meta: &HashMap<String, String>) -> Self {
    let nd_budget = meta.get("nd_budget").and_then(|v| v.trim().parse::<i32>().ok());
    let confidence = meta.get("confidence").and_then(|v| v.trim().parse::<f64>().ok());
    let max_iterations = meta
      .get("max_iterations")
      .and_then(|v| v.trim().parse::<u32>().ok())
      .filter(|v| *v > 0)
      .unwrap_or(1);
    let fallback = match meta.get("fallback").map(|s| s.trim().to_ascii_lowercase()) {
      Some(v) if v == "stub" => FallbackMode::Stub,
      Some(v) if v == "replay" => FallbackMode::Replay,
      _ => FallbackMode::Fail,
    };
    Self {
      nd_budget,
      confidence,
      max_iterations,
      fallback,
    }
  }
}

impl FallbackMode {
  pub fn as_str(&self) -> &'static str {
    match self {
      FallbackMode::Fail => "fail",
      FallbackMode::Stub => "stub",
      FallbackMode::Replay => "replay",
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parses_controls_from_meta() {
    let mut meta = HashMap::new();
    meta.insert("nd_budget".to_string(), "30".to_string());
    meta.insert("confidence".to_string(), "0.8".to_string());
    meta.insert("max_iterations".to_string(), "3".to_string());
    meta.insert("fallback".to_string(), "stub".to_string());
    let c = ConvergenceControls::from_meta(&meta);
    assert_eq!(c.nd_budget, Some(30));
    assert_eq!(c.confidence, Some(0.8));
    assert_eq!(c.max_iterations, 3);
    assert_eq!(c.fallback, FallbackMode::Stub);
  }
}
