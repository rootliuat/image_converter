#[cfg(windows)]
fn main() {
    let mut res = winres::WindowsResource::new();
    res.set_icon("resources/icon.ico")
        .set("FileDescription", "图片格式转换工具")
        .set("ProductName", "Image Converter")
        .set("CompanyName", "Your Company");
    
    if let Err(e) = res.compile() {
        eprintln!("Failed to compile resources: {}", e);
    }
}

#[cfg(not(windows))]
fn main() {
    // Non-Windows platforms don't need resource compilation
}