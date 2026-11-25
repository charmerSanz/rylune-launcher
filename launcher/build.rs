fn main(){#[cfg(windows)] {let mut r=winresource::WindowsResource::new();r.set_icon("icons/icon.ico");r.compile().expect("Windows resources");}}
