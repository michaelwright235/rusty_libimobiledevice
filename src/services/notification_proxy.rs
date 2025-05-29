// jkcoxson

use std::ffi::CString;

use crate::bindings as unsafe_bindings;
use crate::error::NpError;
use crate::idevice::Device;
use crate::services::lockdownd::LockdowndService;
use std::os::raw::{c_void, c_char};

struct NotifyCallback<'a> {
    pub callback: Box<dyn FnMut (String) + 'a>,
    pub wrapper: Box<unsafe extern "C" fn (*const c_char, *mut c_void)>
}

/// A service to proxy notifications to the device
pub struct NotificationProxyClient<'a> {
    pub(crate) pointer: unsafe_bindings::np_client_t,
    phantom: std::marker::PhantomData<&'a Device>,
    notify_callback: Option<Box<NotifyCallback<'a>>>,
}

impl<'a> NotificationProxyClient<'a> {
    /// Creates a new notification proxy from a lockdown service
    /// # Arguments
    /// * `device` - The device to connect to
    /// * `descriptor` - The lockdown service to connect on
    /// # Returns
    /// A struct containing the handle to the connection
    ///
    /// ***Verified:*** False
    pub fn new(device: &'a Device, descriptor: &LockdowndService) -> Result<Self, NpError> {
        let mut pointer = std::ptr::null_mut();
        let result = unsafe {
            unsafe_bindings::np_client_new(device.pointer, descriptor.pointer, &mut pointer)
        }
        .into();

        if result != NpError::Success {
            return Err(result);
        }

        Ok(Self {
            pointer,
            phantom: std::marker::PhantomData,
            notify_callback: None,
        })
    }

    /// Starts a new connection and adds a notification proxy to it
    /// # Arguments
    /// * `device` - The device to connect to
    /// * `label` - The label for the connection
    /// # Returns
    /// A struct containing the handle to the connection
    ///
    /// ***Verified:*** False
    pub fn start_service(device: &'a Device, label: impl Into<String>) -> Result<Self, NpError> {
        let label_c_string = CString::new(label.into()).unwrap();

        let mut pointer = std::ptr::null_mut();
        let result = unsafe {
            unsafe_bindings::np_client_start_service(
                device.pointer,
                &mut pointer,
                label_c_string.as_ptr(),
            )
        }
        .into();

        if result != NpError::Success {
            return Err(result);
        }

        Ok(Self {
            pointer,
            phantom: std::marker::PhantomData,
            notify_callback: None,
        })
    }

    /// Sends a notification to the device
    /// # Arguments
    /// * `notification` - The contents of the notification
    /// # Returns
    /// *none*
    ///
    /// ***Verified:*** False
    pub fn post_notification(&self, notification: impl Into<String>) -> Result<(), NpError> {
        let notification_c_string = CString::new(notification.into()).unwrap();
        let result = unsafe {
            unsafe_bindings::np_post_notification(self.pointer, notification_c_string.as_ptr())
        }
        .into();

        if result != NpError::Success {
            return Err(result);
        }

        Ok(())
    }

    /// Tells the proxy to send a notification when an event occurs
    /// # Arguments
    /// * `notification` - The contents of the notification
    /// # Returns
    /// *none*
    ///
    /// ***Verified:*** False
    pub fn observe_notification(&self, notification: impl Into<String>) -> Result<(), NpError> {
        let notification_c_string = CString::new(notification.into()).unwrap();
        let result = unsafe {
            unsafe_bindings::np_observe_notification(self.pointer, notification_c_string.as_ptr())
        }
        .into();

        if result != NpError::Success {
            return Err(result);
        }

        Ok(())
    }

    /// Tells the proxy to send notifications when an event occurs
    /// # Arguments
    /// * `notifications` - The contents of the notifications
    /// # Returns
    /// *none*
    ///
    /// ***Verified:*** False
    pub fn observe_notifications<I,S>(&self, notifications: I) -> Result<(), NpError>
    where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
    {
        let mut not_c_strings = Vec::new();
        let mut not_ptrs = Vec::new();

        for notification in notifications {
            not_c_strings.push(CString::new(notification.as_ref().to_string()).unwrap());
            not_ptrs.push(not_c_strings.last().unwrap().as_ptr());
        }
        not_ptrs.push(std::ptr::null());

        let result = unsafe {
            unsafe_bindings::np_observe_notifications(self.pointer, not_ptrs.as_mut_ptr())
        }
        .into();

        if result != NpError::Success {
            return Err(result);
        }

        Ok(())
    }

    /// Defines a callback function that will be called when a notification has been received.
    ///
    /// Only one callback function can be registered at the same time;
    /// any previously set callback function will be removed automatically.
    ///
    /// When a notification proxy client gets dropped so does the callback.
    /// Make sure to keep it in memory until you don't need to observe
    /// notifications anymore.
    ///
    /// # Arguments
    /// * `callback` - A callback function
    /// # Returns
    /// *none*
    /// # Example
    /// ```rust
    /// let mut proxy = NotificationProxyClient::new(&device, np_service)?;
    /// proxy.set_notify_callback(|notification| {
    ///     println!("Received notification: {notification}");
    /// });
    /// ```
    ///
    /// ***Verified:*** False
    pub fn set_notify_callback(&mut self, callback: impl FnMut(String) + 'a) -> Result<(), NpError> {

        unsafe extern "C" fn wrapper(notification: *const c_char, user_data: *mut c_void) {
            let notification = std::ffi::CStr::from_ptr(notification)
                .to_string_lossy()
                .into_owned();
            let callback_ptr =  &mut *(user_data as *mut NotifyCallback);
            callback_ptr.callback.as_mut()(notification);
        }

        let notify_callback = NotifyCallback {
            callback: Box::new(callback),
            wrapper: Box::new(wrapper)
        };
        self.notify_callback = Some(Box::new(notify_callback));

        // The memory address of NotifyCallback and a wrapper shoudn't change, so we put them inside boxes
        // and then get the underlying pointers
        let wrapper_ptr = *self.notify_callback.as_ref().unwrap().wrapper.as_ref();
        let callback_ptr = (self.notify_callback.as_mut().unwrap().as_mut() as *mut NotifyCallback) as *mut c_void;

        let result = unsafe {
            unsafe_bindings::np_set_notify_callback(
                self.pointer,
                Some(wrapper_ptr),
                callback_ptr)
        }.into();

        if result != NpError::Success {
            return Err(result);
        }

        Ok(())
    }

    /// Clears a callback function of a notification proxy client
    pub fn clear_notify_callback(&mut self) {
        unsafe {
            unsafe_bindings::np_set_notify_callback(
                self.pointer,
                None,
                std::ptr::null_mut())
        };
        self.notify_callback = None;
    }
}

impl Drop for NotificationProxyClient<'_> {
    fn drop(&mut self) {
        unsafe {
            self.clear_notify_callback();
            unsafe_bindings::np_client_free(self.pointer);
        }
    }
}
