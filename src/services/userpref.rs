// jkcoxson

use std::ffi::CString;

use plist_plus2::{from_pointer, Value};

use crate::bindings as unsafe_bindings;
use crate::error::UserPrefError;

/// Read the pair record from usbmuxd into a plist
/// # Arguments
/// * `udid` - The UDID of the device to fetch the pairing record of
/// # Returns
/// A plist containing the pair record
pub fn read_pair_record<'b>(udid: impl Into<String>) -> Result<Value<'b>, UserPrefError> {
    let udid = CString::new(udid.into()).unwrap();
    let mut to_fill = unsafe { std::mem::zeroed() };
    let results =
        unsafe { unsafe_bindings::userpref_read_pair_record(udid.as_ptr(), &mut to_fill) }.into();
    if results != UserPrefError::Success {
        return Err(results);
    }
    Ok(unsafe {from_pointer(to_fill)})
}
