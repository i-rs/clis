use crate::error::ClawError;
use crate::tools::{ClawTool, ToolContext, run_blocking};
use serde_json::Value;
use std::path::PathBuf;
use std::process::Command;

/// Built-in tool for reading images from clipboard (OCR + vision).
///
/// macOS-only: reads clipboard images, extracts text using built-in OCR
/// (Apple Vision framework via Shortcuts or osascript).
pub struct VisionTool;

#[async_trait::async_trait]
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

    async fn execute(&self, _args: &Value, _ctx: &ToolContext) -> Result<String, ClawError> {
        run_blocking("vision", read_clipboard_image_text).await
    }
}

fn read_clipboard_image_text() -> Result<String, ClawError> {
    // Try multiple OCR methods in order of preference

    // Method 1: macOS Shortcuts "Extract Text from Image"
    if let Ok(text) = ocr_via_shortcuts()
        && !text.trim().is_empty()
    {
        return Ok(format!("从剪贴板图片中识别的文字:\n\n{}", text.trim()));
    }

    // Method 2: osascript with Apple Vision Framework
    if let Ok(text) = ocr_via_osascript()
        && !text.trim().is_empty()
    {
        return Ok(format!("从剪贴板图片中识别的文字:\n\n{}", text.trim()));
    }

    // Method 3: Check if clipboard has image at all
    let has_image = clipboard_has_image();
    if has_image {
        Err(ClawError::Execution(
            "剪贴板中有图片，但无法提取文字。请尝试：
1. 安装 'Extract Text from Image' Shortcut
2. 或使用第三方 OCR 工具如 TextSniper"
                .to_string(),
        ))
    } else {
        Err(ClawError::Execution(
            "剪贴板中没有图片。请先复制一张图片到剪贴板（截图 Cmd+Shift+4 或复制图片）".to_string(),
        ))
    }
}

/// Check if clipboard contains an image via osascript.
fn clipboard_has_image() -> bool {
    // Simpler approach: just try to get TIFF data
    let output = Command::new("osascript")
        .args([
            "-e",
            r#"
try
    set theImage to (the clipboard as picture)
    return "image"
on error
    return "no_image"
end try
"#,
        ])
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
fn ocr_via_shortcuts() -> Result<String, ClawError> {
    let temp_png = temp_path("vision_clipboard.png");

    // Save clipboard image to file using osascript + Image Events
    let save_result = Command::new("osascript")
        .args([
            "-e",
            &format!(
                r#"set theImage to (the clipboard as «class PNGf»)
set outFile to open for access POSIX file "{}" with write permission
write theImage to outFile
close access outFile"#,
                temp_png.display().to_string().replace("\"", "\\\"")
            ),
        ])
        .output();

    if save_result.is_err() {
        return Err(ClawError::Execution("无法保存剪贴板图片".to_string()));
    }

    // Check the saved file exists and has content
    let file_size = std::fs::metadata(&temp_png).map(|m| m.len()).unwrap_or(0);
    if file_size == 0 {
        let _ = std::fs::remove_file(&temp_png);
        return Err(ClawError::Execution("剪贴板中无图片数据".to_string()));
    }

    // Try the "Extract Text from Image" shortcut
    let output = Command::new("shortcuts")
        .args([
            "run",
            "Extract Text from Image",
            "--input-path",
            &temp_png.to_string_lossy(),
        ])
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
    Err(ClawError::Execution("Shortcuts OCR 返回空结果".to_string()))
}

/// OCR via osascript using Apple's Vision framework (VNVNRequest).
/// This uses a compiled Swift snippet approach.
fn ocr_via_osascript() -> Result<String, ClawError> {
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
        return Err(ClawError::Execution("无法创建 OCR 临时脚本".to_string()));
    }

    let output = Command::new("swift")
        .arg(&swift_path)
        .output()
        .map_err(|e| {
            let _ = std::fs::remove_file(&swift_path);
            format!("Swift OCR 执行失败: {}", e)
        })?;

    let _ = std::fs::remove_file(&swift_path);

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let result = if !stdout.trim().is_empty() {
            stdout
        } else if !stderr.trim().is_empty() && !stderr.contains("ERROR:") {
            stderr
        } else {
            return Err(ClawError::Execution(format!("Swift OCR 错误: {}", stderr)));
        };
        Ok(result.trim().to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Err(ClawError::Execution(format!("Swift OCR 失败: {}", stderr)))
    }
}

/// Get a temp file path in system temp directory.
fn temp_path(filename: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push("i-rs-claw");
    let _ = std::fs::create_dir_all(&path);
    let unique_name = format!("{}-{}", fastrand::u64(0..u64::MAX), filename);
    path.push(unique_name);
    path
}
