use super::*;
use serde_json::Number;
/**
 * taken from https://github.com/marvindv/jsonlogic_rs/blob/master/src/operators/logic.rs
 */

pub fn less_than( lhs: &PicoValue, rhs: &PicoValue) -> bool {

  match(lhs,rhs) {

    (PicoValue::Null, PicoValue::Null) => false,
    (PicoValue::Bool(false), PicoValue::Bool(true)) => true,

    (PicoValue::Bool(_), PicoValue::Bool(_)) => false,
    (PicoValue::String(l), PicoValue::String(r)) => l < r,


    _ => false //TODO
  }

}

pub fn is_strict_equal(lhs: &PicoValue, rhs: &PicoValue) -> bool {

  match(lhs, rhs) {
    (PicoValue::Array(_), PicoValue::Array(_)) => false,
    (PicoValue::Bool(l), PicoValue::Bool(r)) => l==r,
    (PicoValue::Null, PicoValue::Null) => true,
    (PicoValue::Number(l), PicoValue::Number(r)) => equal_number(l,r),
    (PicoValue::Object(_), PicoValue::Object(_)) => false,
    (PicoValue::String(l), PicoValue::String(r)) => l == r,

    _ => false

  }

}

pub fn equality( lhs: &PicoValue, rhs: &PicoValue) -> bool {

  match(lhs, rhs) {

    ( PicoValue::Array(_), PicoValue::Array(_) )
    |( PicoValue::Bool(_), PicoValue::Bool(_) )
    
    => is_strict_equal(lhs, rhs ),

    _ => false

  }
}

pub fn equal_number(lhs: &Number, rhs: &Number) -> bool {

  if lhs.is_u64() && rhs.is_u64() {
    lhs.as_u64().unwrap() == rhs.as_u64().unwrap()
  } else if lhs.is_i64() && rhs.is_i64()  {
    lhs.as_i64().unwrap() == rhs.as_i64().unwrap()
  } else {
    lhs.as_f64().unwrap() == rhs.as_f64().unwrap()
  }

}