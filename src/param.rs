#![allow(clippy::missing_safety_doc)]
//! Gurobi parameters for [`Env`](crate::Env)  and [`Model`](crate::Model) objects.  See the
//! [manual](https://www.gurobi.com/documentation/9.1/refman/parameters.html) for a list
//! of parameters and their uses.
use grb_sys as ffi;
use std::ffi::CString;
use std::result::Result as StdResult;

use crate::util::{copy_c_str};
use crate::constants::{ERROR_INVALID_ARGUMENT, GRB_MAX_STRLEN};

pub use ffi::{IntParam, DoubleParam, StringParam};
#[doc(inline)]
pub use IntParam::*;
pub use DoubleParam::*;
pub use StringParam::*;
// TODO add an Undocumented parameter type - eg GRB_MINPARFORBID

type RawResult<T> = StdResult<T, ffi::c_int>;

fn check_error_code(code: ffi::c_int) -> RawResult<()> {
  if code == 0 { Ok(()) } else { Err(code) }
}

/// This is an implementation detail and should not be considered as part of the public API of this crate.
pub trait Param: Sized + Into<CString> {
  /// This paramter's value type (string, double, int, char)
  type Value;
  /// Query a parameter from an environment
  unsafe fn get_param(self, env: *mut ffi::GRBenv) -> RawResult<Self::Value>;
  /// Set a parameter on an environment
  unsafe fn set_param(self, env: *mut ffi::GRBenv, value: Self::Value) -> RawResult<()>;
}

macro_rules! impl_param_copy_ty {
    ($t:ty, $vt:ty, $init:expr, $get:path, $set:path) => {
      impl Param for $t {
        type Value = $vt;

        #[inline]
        unsafe fn get_param(self, env: *mut ffi::GRBenv) -> RawResult<Self::Value> {
        let pname: CString = self.into();
        let mut val = $init;
        check_error_code($get(env, pname.as_ptr(), &mut val))?;
        Ok(val)
        }

        #[inline]
        unsafe fn set_param(self, env: *mut ffi::GRBenv, value: Self::Value) -> RawResult<()> {
        let pname: CString = self.into();
        check_error_code($set(env, pname.as_ptr(), value))
        }
      }
    };
}

impl_param_copy_ty!(IntParam, i32, i32::MIN, ffi::GRBgetintparam, ffi::GRBsetintparam);
impl_param_copy_ty!(DoubleParam, f64, f64::NAN, ffi::GRBgetdblparam, ffi::GRBsetdblparam);


impl Param for StringParam {
  type Value = String;

  #[inline]
  unsafe fn get_param(self, env: *mut ffi::GRBenv) -> RawResult<String> {
    let pname : CString = self.into();
    let mut buf = [0i8; GRB_MAX_STRLEN];
    check_error_code(ffi::GRBgetstrparam(env, pname.as_ptr(), buf.as_mut_ptr()))?;
    Ok(copy_c_str(buf.as_ptr()))
  }

  #[inline]
  unsafe fn set_param(self, env: *mut ffi::GRBenv, value: Self::Value) -> RawResult<()> {
    let pname : CString = self.into();
    let value = CString::new(value).map_err(|_| ERROR_INVALID_ARGUMENT)?;
    check_error_code(ffi::GRBsetstrparam(env, pname.as_ptr(), value.as_ptr()))
  }
}
