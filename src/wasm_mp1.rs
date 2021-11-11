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

//! This module provide rust implementation for WASM module to talk with the
//! outside according to version 1 of 3nweb's message passing api (should be
//! called abi?).
//! 
//! Process of this message passing version is following.
//! 
//! - To send messages outside, WASM uses imported `_3nweb_mp1_send_out_msg`
//! during which call embeddder must read message from identified memory area.
//! 
//! - To send messages inside, embedder uses exported from WASM
//! `_3nweb_mp1_accept_msg`. During this call, WASM calls back embedder's
//! imported `_3nweb_mp1_write_msg_into`, where embedder actually copies data
//! into provided memory area.
//! 

mod internals {

	use wasm_bindgen::prelude::*;

	/// Sends given binary message to the outside. This is implementation.
	/// 
	pub fn send_msg_out(msg: &Vec<u8>) -> () {
		unsafe {
			_3nweb_mp1_send_out_msg(msg.as_ptr() as usize, msg.len());
		}
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

	/// This simple classic externing expects to find these functions in `env`
	/// object/namespace imported to WASM by embedding.
	/// 
	extern {

		/// Don't use this directly.
		/// WASM embedding is expected to provide this function in accordance with
		/// 3nweb's message passing api, version 1, indicated be `_3nweb_mp1_`
		/// prefix in the name.
		/// 
		/// This function is called to tell embedding that a message for the
		/// outside with length `len` can be copied from given pointer `ptr`.
		/// 
		/// Embedder provides this callback in `env` namespace of imports.
		/// 
		fn _3nweb_mp1_send_out_msg(ptr: usize, len: usize);

		/// Don't use this directly.
		/// WASM embedding is expected to provide this function in accordance with
		/// 3nweb's message passing api, version 1, indicated be `_3nweb_mp1_`
		/// prefix in the name.
		/// 
		/// This function is a callback invoked during passing message into WASM. 
		/// Embedding calls exported `_3nweb_mp1_accept_msg`, which itself
		/// prepairs memory for message, and calls this imported function so that
		/// embedding writes its message into the buffer.
		/// 
		/// Embedder provides this callback in `env` namespace of imports.
		/// 
		fn _3nweb_mp1_write_msg_into(ptr: usize);

	}

	/// Don't use this directly.
	/// This function is exported from WASM in accordance with 3nweb's message
	/// passing api, version 1, indicated be `_3nweb_mp1_` prefix in the name.
	/// 
	/// This is called by WASM embedding with `len` size of the message.
	/// Implementation prepares buffer for writing message bytes and calls back
	/// imported `_3nweb_mp1_write_msg_into`. When callback returns, message is
	/// given to processor.
	/// 
	#[wasm_bindgen]
	pub fn _3nweb_mp1_accept_msg(len: usize) -> () {
		let mut msg = Vec::with_capacity(len);
		unsafe {
			_3nweb_mp1_write_msg_into(msg.as_ptr() as usize);
			msg.set_len(len);
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
