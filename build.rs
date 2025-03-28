fn main() {
    slint_build::compile("ui/app-window.slint").expect("Slint build failed");

    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        res.set_icon("E:\\Blender\\Nodot\\images\\icon.ico"); // Path to your .ico file
        res.compile().expect("Failed to embed icon");
    }
}
