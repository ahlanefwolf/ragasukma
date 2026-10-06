fn main() {
    #[cfg(target_os = "windows")]
    {
        // Enforce Windows GUI Subsystem pada level binary executable target
        println!("cargo:rustc-link-arg-bins=/SUBSYSTEM:WINDOWS");
        println!("cargo:rustc-link-arg-bins=/ENTRY:mainCRTStartup");

        let mut res = winres::WindowsResource::new();
        res.set_icon("icons/icon.ico");
        if let Err(e) = res.compile() {
            eprintln!("Warning: Failed to compile winres icon: {e}");
        }
    }
}
