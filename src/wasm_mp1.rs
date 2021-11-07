// Copyright(c) 2021 3NSoft Inc.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

mod internals {

	use std::alloc::{ alloc, Layout, dealloc };
	use std::ptr::copy;
	use std::slice;
	use wasm_bindgen::prelude::*;

	struct MsgExchangeBufferInfo {
		ptr: *mut u8,
		#[allow(dead_code)]
		len: usize,
		#[allow(dead_code)]
		layout: Option<Layout>,
	}

	static mut BUFFER_INFO: MsgExchangeBufferInfo = MsgExchangeBufferInfo {
		ptr: 0 as *mut u8,
		len: 0,
		layout: None,
	};

	/// Don't use this directly.
	/// This function is exported from WASM in accordance with 3nweb's message
	/// passing api, version 1, indicated be `_3nweb_mp1_` prefix in the name.
	/// 
	/// When WASM embedding is ready to give message, it asks to prepare a
	/// buffer. This particular implementation either reuses or allocates new
	/// buffer. Buffer pointer is returned, while `buf_size` has been given by
	/// and is known to the caller. It is expected that `_3nweb_mp1_accept_msg`
	/// is called by the caller right after it has copied messages bytes into a
	/// given buffer.
	/// 
	#[wasm_bindgen]
	pub fn _3nweb_mp1_get_buffer(buf_size: usize) -> usize {
		unsafe {
			if BUFFER_INFO.layout.is_some() {
				if BUFFER_INFO.len <= buf_size {
					return BUFFER_INFO.ptr as usize;
				}
				dealloc(BUFFER_INFO.ptr, BUFFER_INFO.layout.unwrap())
			}
			let layout = Layout::array::<u8>(buf_size).ok().unwrap();
			BUFFER_INFO.len = buf_size;
			BUFFER_INFO.ptr = alloc(layout);
			BUFFER_INFO.layout = Some(layout);
			return BUFFER_INFO.ptr as usize;
		}
	}

	/// Sends given binary message to the outside. This is implementation.
	/// 
	pub fn send_msg_out(msg: &Vec<u8>) -> () {
		unsafe {
			if BUFFER_INFO.len < msg.len() {
				_3nweb_mp1_get_buffer(msg.len());
			}
			copy(msg.as_ptr(), BUFFER_INFO.ptr, msg.len());
			_3nweb_mp1_send_out_msg(BUFFER_INFO.ptr as usize, msg.len());
		}
	}

	/// Reads message from buffer, returning `Vec<u8>`'s separate from message
	/// exchange buffer(s).
	/// 
	#[allow(dead_code)]
	fn read_msg_from_buffer(len: usize) -> Vec<u8> {
		let buf = unsafe {
			slice::from_raw_parts(BUFFER_INFO.ptr, len)
		};
		Vec::from(buf)
	}

	#[allow(dead_code)]
	static mut MSG_PROCESSOR: Option<&dyn Fn(Vec<u8>) -> ()> = None;

	/// Sets a message `processor` function/closure that will be called with
	/// binary messages from the outside. This is implementation.
	/// 
	/// Messages are given to `processor` as `Vec<u8>` completely separated from
	/// workings of message exchange buffer(s).
	/// 
	pub fn set_msg_processor(processor: &'static dyn Fn(Vec<u8>) -> ()) -> () {
		unsafe {
			MSG_PROCESSOR = Some(processor);
		}
	}

	extern {
		/// Don't use this directly.
		/// WASM embedding is expected to provide this function in accordance with
		/// 3nweb's message passing api, version 1, indicated be `_3nweb_mp1_`
		/// prefix in the name.
		/// 
		/// This function is called to tell embedding that a message for the
		/// outside with length `len` can be copied from given pointer `ptr`.
		/// Embedder provides this callback in `env` namespace of imports.
		/// 
		fn _3nweb_mp1_send_out_msg(ptr: usize, len: usize);
	}

	/// Don't use this directly.
	/// This function is exported from WASM in accordance with 3nweb's message
	/// passing api, version 1, indicated be `_3nweb_mp1_` prefix in the name.
	/// 
	/// This is called by WASM embedding right after it copying a message into
	/// a buffer set during `_3nweb_mp1_get_buffer` call. Call gets only `len` of
	/// message, while it is expected to start at buffer's head. Implementation
	/// extracts messages into `Vec<u8>`'s separate from message exchange
	/// buffer(s), passing them into processor function/closure set by
	/// `set_msg_processor`.
	/// 
	#[wasm_bindgen]
	pub fn _3nweb_mp1_accept_msg(len: usize) -> () {
		let msg = if len > 0 { read_msg_from_buffer(len) } else { Vec::new() };
		unsafe {
			if MSG_PROCESSOR.is_some() {
				(MSG_PROCESSOR.as_ref().unwrap())(msg);
			}
		}
	}

}

/// Sends given binary message to the outside.
/// 
#[inline]
pub fn send_msg_out(msg: &Vec<u8>) -> () {
	internals::send_msg_out(msg);
}

/// Sets a message `processor` function/closure that will be called with binary
/// messages from the outside.
/// 
/// Messages are given to `processor` as `Vec<u8>` completely separated from
/// workings of message exchange buffer(s).
/// 
#[inline]
pub fn set_msg_processor(processor: &'static dyn Fn(Vec<u8>) -> ()) -> () {
	internals::set_msg_processor(processor);
}
