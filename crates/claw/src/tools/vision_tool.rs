use crate::tools::{ClawTool, ToolContext};
use serde_json::Value;
use std::path::PathBuf;
use std::process::Command;

/// Built-in tool for reading images from clipboard (OCR + vision).
///
/// macOS-only: reads clipboard images, extracts text using built-in OCR
/// (Apple Vision framework via Shortcuts or osascript).
pub struct VisionTool;

impl ClawTool for VisionTool {
    fn name(&self) -> &str {
        "read_clipboard_image"
    }

    fn description(&self) -> &str {
        "Read text from an image in the clipboard using OCR. Use this when the \
         user says they copied a screenshot or image to the clipboard and wants \
         you to read text from it. macOS only."
    }

    fn parameter_schema(&self, _enabled_cli_tools: &[&str]) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {}
        })
    }

    fn execute(&self, _args: &Value, _ctx: &ToolContext) -> Result<String, String> {
        read_clipboard_image_text()
    }
}

fn read_clipboard_image_text() -> Result<String, String> {
    // Try multiple OCR methods in order of preference

    // Method 1: macOS Shortcuts "Extract Text from Image"
    if let Ok(text) = ocr_via_shortcuts()
        && !text.trim().is_empty() {
            return Ok(format!("从剪贴板图片中识别的文字:\n\n{}", text.trim()));
        }

    // Method 2: osascript with Apple Vision Framework
    if let Ok(text) = ocr_via_osascript()
        && !text.trim().is_empty() {
            return Ok(format!("从剪贴板图片中识别的文字:\n\n{}", text.trim()));
        }

    // Method 3: Check if clipboard has image at all
    let has_image = clipboard_has_image();
    if has_image {
        Err("剪贴板中有图片，但无法提取文字。请尝试：
1. 安装 'Extract Text from Image' Shortcut
2. 或使用第三方 OCR 工具如 TextSniper".to_string())
    } else {
        Err("剪贴板中没有图片。请先复制一张图片到剪贴板（截图 Cmd+Shift+4 或复制图片）".to_string())
    }
}

/// Check if clipboard contains an image via osascript.
fn clipboard_has_image() -> bool {
    // Simpler approach: just try to get TIFF data
    let output = Command::new("osascript")
        .args(["-e", r#"
try
    set theImage to (the clipboard as picture)
    return "image"
on error
    return "no_image"
end try
"#])
        .output();
    match output {
        Ok(out) => {
            let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
            s == "image"
        }
        Err(_) => false,
    }
}

/// OCR via macOS Shortcuts "Extract Text from Image" if installed.
fn ocr_via_shortcuts() -> Result<String, String> {
    // Run the shortcut - if it exists, it will OCR the clipboard image
    let temp_png = temp_path("vision_clipboard.png");

    // First save clipboard image to file
    let save_result = Command::new("osascript")
        .args(["-e", &format!(
            r#"try
    set theImage to (the clipboard as picture)
    set outFile to (POSIX file "{}")
    tell application "Image Events"
        launch
        set thisImage to open outFile
        -- write image data
    end tell
end try"#,
            temp_png.display().to_string().replace("\"", "\\\"")
        )])
        .output();

    if save_result.is_err() {
        return Err("无法保存剪贴板图片".to_string());
    }

    // Try the "Extract Text from Image" shortcut
    let output = Command::new("shortcuts")
        .args(["run", "Extract Text from Image", "--input-path", &temp_png.to_string_lossy()])
        .output()
        .map_err(|e| format!("Shortcuts 不可用: {}", e))?;

    if output.status.success() {
        let text = String::from_utf8_lossy(&output.stdout).to_string();
        let _ = std::fs::remove_file(&temp_png);
        if !text.trim().is_empty() {
            return Ok(text);
        }
    }

    // Cleanup
    let _ = std::fs::remove_file(&temp_png);
    Err("Shortcuts OCR 返回空结果".to_string())
}

/// OCR via osascript using Apple's Vision framework (VNVNRequest).
/// This uses a compiled Swift snippet approach.
fn ocr_via_osascript() -> Result<String, String> {
    let temp_png = temp_path("vision_ocr.png");

    // Save clipboard image to temp file using sips
    let save_result = Command::new("osascript")
        .args(["-e", &format!(
            r#"try
    set theImage to (the clipboard as picture)
    set outFile to (POSIX file "{}")
    tell application "System Events"
        -- save via sips
    end tell
end try"#,
            temp_png.display().to_string().replace("\"", "\\\"")
        )])
        .output();

    if save_result.is_err() {
        return Err("无法保存剪贴板图片".to_string());
    }

    // Use sips to convert clipboard to temp file
    // Actually, sips can't read from clipboard directly.
    // Let's use a different approach: write a small Swift script

    // Simpler: use `osascript` to call Apple's built-in OCR via Vision framework
    // This requires a compiled Swift executable or using `swift` interpreter

    let swift_code = r#"
import Cocoa
import Vision

func extractText() -> String? {
    guard let image = NSPasteboard.general.readObjects(forClasses: [NSImage.self], options: nil)?.first as? NSImage else {
        return nil
    }
    guard let cgImage = image.cgImage(forProposedRect: nil, context: nil, hints: nil) else {
        return nil
    }
    
    let semaphore = DispatchSemaphore(value: 0)
    var resultText = ""
    
    let request = VNRecognizeTextRequest { request, error in
        if let error = error {
            print("ERROR: \(error.localizedDescription)")
            semaphore.signal()
            return
        }
        let observations = request.results as? [VNRecognizedTextObservation] ?? []
        for observation in observations {
            if let topCandidate = observation.topCandidates(1).first {
                resultText += topCandidate.string + "\n"
            }
        }
        semaphore.signal()
    }
    request.recognitionLevel = .accurate
    request.usesLanguageCorrection = true
    
    let handler = VNImageRequestHandler(cgImage: cgImage, options: [:])
    DispatchQueue.global().async {
        try? handler.perform([request])
    }
    semaphore.wait()
    
    return resultText.isEmpty ? nil : resultText
}

if let text = extractText() {
    print(text)
}
"#;

    // Write the Swift code to a temp file and execute with `swift`
    let swift_path = temp_path("vision_ocr.swift");
    if std::fs::write(&swift_path, swift_code).is_err() {
        let _ = std::fs::remove_file(&temp_png);
        return Err("无法创建 OCR 临时脚本".to_string());
    }

    let output = Command::new("swift")
        .arg(&swift_path)
        .output()
        .map_err(|e| {
            let _ = std::fs::remove_file(&swift_path);
            let _ = std::fs::remove_file(&temp_png);
            format!("Swift OCR 执行失败: {}", e)
        })?;

    let _ = std::fs::remove_file(&swift_path);
    let _ = std::fs::remove_file(&temp_png);

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let result = if !stdout.trim().is_empty() {
            stdout
        } else if !stderr.trim().is_empty() && !stderr.contains("ERROR:") {
            stderr
        } else {
            return Err(format!("Swift OCR 错误: {}", stderr));
        };
        Ok(result.trim().to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Err(format!("Swift OCR 失败: {}", stderr))
    }
}

/// Get a temp file path in system temp directory.
fn temp_path(filename: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push("i-rs-claw");
    let _ = std::fs::create_dir_all(&path);
    path.push(filename);
    path
}
