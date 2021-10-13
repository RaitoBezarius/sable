extern crate proc_macro;

use proc_macro::TokenStream;

mod define_event_details;

#[proc_macro]
pub fn event_details(input: TokenStream) -> TokenStream
{
    define_event_details::event_details(input)
}

mod define_command_handler;

#[proc_macro]
pub fn command_handler(input: TokenStream) -> TokenStream
{
    define_command_handler::command_handler(input)
}

mod define_object_id;

#[proc_macro]
pub fn object_ids(input: TokenStream) -> TokenStream
{
    define_object_id::object_ids(input)
}