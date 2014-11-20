/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Just some stubs so that you can link against this library in place of the
//! Chromium version of CEF. If you call these functions you will assuredly
//! crash.

macro_rules! stub(
    ($name:ident) => (
        #[no_mangle]
        pub extern "C" fn $name() {
            println!("CEF stub function called: {}", stringify!($name));
            unsafe {
                ::std::intrinsics::abort()
            }
        }
    )
)

stub!(cef_add_cross_origin_whitelist_entry)
stub!(cef_add_web_plugin_directory)
stub!(cef_add_web_plugin_path)
stub!(cef_begin_tracing)
stub!(cef_binary_value_create)
stub!(cef_clear_cross_origin_whitelist)
stub!(cef_clear_scheme_handler_factories)
stub!(cef_cookie_manager_create_manager)
stub!(cef_cookie_manager_get_global_manager)
stub!(cef_create_url)
stub!(cef_dictionary_value_create)
stub!(cef_drag_data_create)
stub!(cef_end_tracing)
stub!(cef_force_web_plugin_shutdown)
stub!(cef_get_current_platform_thread_handle)
stub!(cef_get_current_platform_thread_id)
stub!(cef_get_extensions_for_mime_type)
stub!(cef_get_geolocation)
stub!(cef_get_mime_type)
stub!(cef_get_path)
stub!(cef_is_web_plugin_unstable)
stub!(cef_launch_process)
stub!(cef_list_value_create)
stub!(cef_now_from_system_trace_time)
stub!(cef_parse_url)
stub!(cef_post_delayed_task)
stub!(cef_post_task)
stub!(cef_print_settings_create)
stub!(cef_process_message_create)
stub!(cef_refresh_web_plugins)
stub!(cef_register_extension)
stub!(cef_register_scheme_handler_factory)
stub!(cef_register_web_plugin_crash)
stub!(cef_remove_cross_origin_whitelist_entry)
stub!(cef_remove_web_plugin_path)
stub!(cef_request_context_create_context)
stub!(cef_request_context_get_global_context)
stub!(cef_response_create)
stub!(cef_set_osmodal_loop)
stub!(cef_stream_reader_create_for_data)
stub!(cef_stream_reader_create_for_file)
stub!(cef_stream_reader_create_for_handler)
stub!(cef_stream_writer_create_for_file)
stub!(cef_stream_writer_create_for_handler)
stub!(cef_string_utf16_to_wide)
stub!(cef_string_wide_to_utf16)
stub!(cef_string_wide_to_utf8)
stub!(cef_task_runner_get_for_current_thread)
stub!(cef_task_runner_get_for_thread)
stub!(cef_unregister_internal_web_plugin)
stub!(cef_v8context_get_current_context)
stub!(cef_v8context_get_entered_context)
stub!(cef_v8context_in_context)
stub!(cef_v8stack_trace_get_current)
stub!(cef_v8value_create_array)
stub!(cef_v8value_create_bool)
stub!(cef_v8value_create_date)
stub!(cef_v8value_create_double)
stub!(cef_v8value_create_function)
stub!(cef_v8value_create_int)
stub!(cef_v8value_create_null)
stub!(cef_v8value_create_object)
stub!(cef_v8value_create_string)
stub!(cef_v8value_create_uint)
stub!(cef_v8value_create_undefined)
stub!(cef_visit_web_plugin_info)
stub!(cef_xml_reader_create)
stub!(cef_zip_reader_create)

