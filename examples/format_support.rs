//! 多格式文件支持示例
//!
//! 这个示例展示了 COS Rust SDK 对各种文件格式的支持，
//! 包括图片、视频、音频、文档等多种类型的文件。
//!
//! 运行示例：
//! ```bash
//! cargo run --example format_support
//! ```
//!
//! 注意：运行前请设置环境变量：
//! - COS_SECRET_ID: 腾讯云 SecretId
//! - COS_SECRET_KEY: 腾讯云 SecretKey
//! - COS_REGION: 地域，如 ap-beijing
//! - COS_BUCKET: 存储桶名称（包含 APPID）

use cos_rust_sdk::{Config, CosClient, ObjectClient};
use std::env;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 COS Rust SDK - 多格式文件支持示例");
    println!();

    // 创建配置和客户端
    let config = Config::new(
        &env::var("COS_SECRET_ID").expect("请设置 COS_SECRET_ID 环境变量"),
        &env::var("COS_SECRET_KEY").expect("请设置 COS_SECRET_KEY 环境变量"),
        &env::var("COS_REGION").expect("请设置 COS_REGION 环境变量"),
        &env::var("COS_BUCKET").expect("请设置 COS_BUCKET 环境变量"),
    )
    .with_timeout(Duration::from_secs(120))
    .with_https(true);

    let cos_client = CosClient::new(config)?;
    let object_client = ObjectClient::new(cos_client);

    println!("📋 支持的文件格式列表：");
    println!();

    // 图片格式示例
    println!("🖼️  图片格式：");
    let image_formats = vec![
        ("sample.jpg", "image/jpeg", "JPEG - 最常用的有损压缩图片格式"),
        ("sample.jpeg", "image/jpeg", "JPEG - 另一种扩展名"),
        ("sample.png", "image/png", "PNG - 支持透明度的无损压缩格式"),
        ("sample.gif", "image/gif", "GIF - 支持动画的图片格式"),
        ("sample.webp", "image/webp", "WebP - Google 开发的现代图片格式"),
        ("sample.bmp", "image/bmp", "BMP - Windows 位图格式"),
        ("sample.tiff", "image/tiff", "TIFF - 高质量的无损压缩格式"),
        ("sample.tif", "image/tiff", "TIFF - 另一种扩展名"),
        ("sample.svg", "image/svg+xml", "SVG - 可缩放矢量图形"),
        ("sample.ico", "image/x-icon", "ICO - Windows 图标格式"),
        ("sample.heic", "image/heic", "HEIC - Apple 高效图像格式"),
        ("sample.heif", "image/heif", "HEIF - 高效图像文件格式"),
        ("sample.avif", "image/avif", "AVIF - AV1 图像文件格式"),
        ("sample.jxl", "image/jxl", "JXL - JPEG XL 下一代图像格式"),
    ];

    for (filename, expected_mime, description) in &image_formats {
        test_format_detection(&object_client, filename, expected_mime, description).await;
    }
    println!();

    // 视频格式示例
    println!("🎬 视频格式：");
    let video_formats = vec![
        ("sample.mp4", "video/mp4", "MP4 - 最常用的视频容器格式"),
        ("sample.avi", "video/x-msvideo", "AVI - Microsoft 视频格式"),
        ("sample.mov", "video/quicktime", "MOV - Apple QuickTime 格式"),
        ("sample.wmv", "video/x-ms-wmv", "WMV - Windows Media Video"),
        ("sample.flv", "video/x-flv", "FLV - Flash 视频格式"),
        ("sample.webm", "video/webm", "WebM - Google 开发的开源格式"),
        ("sample.mkv", "video/x-matroska", "MKV - Matroska 多媒体容器"),
        ("sample.m4v", "video/x-m4v", "M4V - iTunes 视频格式"),
        ("sample.3gp", "video/3gpp", "3GP - 3G 移动设备视频格式"),
        ("sample.3g2", "video/3gpp2", "3G2 - 3G2 移动设备视频格式"),
        ("sample.ts", "video/mp2t", "TS - MPEG-2 传输流"),
        ("sample.mts", "video/mp2t", "MTS - MPEG-2 传输流"),
        ("sample.m2ts", "video/mp2t", "M2TS - MPEG-2 传输流"),
        ("sample.ogv", "video/ogg", "OGV - Ogg 视频格式"),
    ];

    for (filename, expected_mime, description) in &video_formats {
        test_format_detection(&object_client, filename, expected_mime, description).await;
    }
    println!();

    // 音频格式示例
    println!("🎵 音频格式：");
    let audio_formats = vec![
        ("sample.mp3", "audio/mpeg", "MP3 - 最常用的有损音频格式"),
        ("sample.wav", "audio/wav", "WAV - 无损音频格式"),
        ("sample.flac", "audio/flac", "FLAC - 自由无损音频编解码器"),
        ("sample.aac", "audio/aac", "AAC - 高级音频编码"),
        ("sample.ogg", "audio/ogg", "OGG - Ogg Vorbis 音频格式"),
        ("sample.wma", "audio/x-ms-wma", "WMA - Windows Media Audio"),
        ("sample.m4a", "audio/mp4", "M4A - MPEG-4 音频格式"),
        ("sample.opus", "audio/opus", "Opus - 现代低延迟音频编解码器"),
    ];

    for (filename, expected_mime, description) in &audio_formats {
        test_format_detection(&object_client, filename, expected_mime, description).await;
    }
    println!();

    // 文档格式示例
    println!("📄 文档格式：");
    let document_formats = vec![
        ("sample.pdf", "application/pdf", "PDF - 便携式文档格式"),
        ("sample.doc", "application/msword", "DOC - Microsoft Word 文档"),
        ("sample.docx", "application/vnd.openxmlformats-officedocument.wordprocessingml.document", "DOCX - Microsoft Word 文档 (新格式)"),
        ("sample.xls", "application/vnd.ms-excel", "XLS - Microsoft Excel 表格"),
        ("sample.xlsx", "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet", "XLSX - Microsoft Excel 表格 (新格式)"),
        ("sample.ppt", "application/vnd.ms-powerpoint", "PPT - Microsoft PowerPoint 演示文稿"),
        ("sample.pptx", "application/vnd.openxmlformats-officedocument.presentationml.presentation", "PPTX - Microsoft PowerPoint 演示文稿 (新格式)"),
        ("sample.rtf", "application/rtf", "RTF - 富文本格式"),
    ];

    for (filename, expected_mime, description) in &document_formats {
        test_format_detection(&object_client, filename, expected_mime, description).await;
    }
    println!();

    // 压缩文件格式示例
    println!("🗜️  压缩文件格式：");
    let archive_formats = vec![
        ("sample.zip", "application/zip", "ZIP - 最常用的压缩格式"),
        ("sample.rar", "application/vnd.rar", "RAR - WinRAR 压缩格式"),
        ("sample.7z", "application/x-7z-compressed", "7Z - 7-Zip 压缩格式"),
        ("sample.tar", "application/x-tar", "TAR - Unix 归档格式"),
        ("sample.gz", "application/gzip", "GZ - Gzip 压缩格式"),
        ("sample.bz2", "application/x-bzip2", "BZ2 - Bzip2 压缩格式"),
    ];

    for (filename, expected_mime, description) in &archive_formats {
        test_format_detection(&object_client, filename, expected_mime, description).await;
    }
    println!();

    // 文本文件格式示例
    println!("📝 文本文件格式：");
    let text_formats = vec![
        ("sample.txt", "text/plain", "TXT - 纯文本文件"),
        ("sample.html", "text/html", "HTML - 超文本标记语言"),
        ("sample.htm", "text/html", "HTM - 超文本标记语言 (另一种扩展名)"),
        ("sample.css", "text/css", "CSS - 层叠样式表"),
        ("sample.js", "application/javascript", "JS - JavaScript 脚本"),
        ("sample.json", "application/json", "JSON - JavaScript 对象表示法"),
        ("sample.xml", "application/xml", "XML - 可扩展标记语言"),
        ("sample.csv", "text/csv", "CSV - 逗号分隔值"),
        ("sample.md", "text/markdown", "MD - Markdown 文档"),
    ];

    for (filename, expected_mime, description) in &text_formats {
        test_format_detection(&object_client, filename, expected_mime, description).await;
    }
    println!();

    // 其他格式示例
    println!("🔧 其他格式：");
    let other_formats = vec![
        ("sample.bin", "application/octet-stream", "BIN - 二进制文件"),
        ("sample.exe", "application/octet-stream", "EXE - 可执行文件"),
        ("sample.dmg", "application/x-apple-diskimage", "DMG - Apple 磁盘映像"),
        ("sample.iso", "application/x-iso9660-image", "ISO - 光盘映像文件"),
    ];

    for (filename, expected_mime, description) in &other_formats {
        test_format_detection(&object_client, filename, expected_mime, description).await;
    }
    println!();

    println!("✅ 格式支持测试完成！");
    println!();
    println!("💡 使用提示：");
    println!("   - SDK 会根据文件扩展名自动检测 MIME 类型");
    println!("   - 您也可以手动指定 Content-Type");
    println!("   - 对于未知格式，将使用 application/octet-stream");
    println!("   - 建议为大文件（如视频）增加超时时间");
    
    Ok(())
}

/// 测试格式检测功能
async fn test_format_detection(
    object_client: &ObjectClient,
    filename: &str,
    expected_mime: &str,
    description: &str,
) {
    // 创建示例数据
    let sample_data = create_sample_data_for_format(filename);
    let key = format!("format-test/{}", filename);
    
    // 上传文件（不指定 Content-Type，让 SDK 自动检测）
    match object_client.put_object(&key, sample_data, None).await {
        Ok(_) => {
            // 获取文件信息验证 MIME 类型
            match object_client.head_object(&key).await {
                Ok(metadata) => {
                    let detected_mime = &metadata.content_type;
                    if detected_mime == expected_mime {
                        println!("   ✅ {} - {} ({})", filename, description, detected_mime);
                    } else {
                        println!("   ⚠️  {} - {} (期望: {}, 实际: {})", filename, description, expected_mime, detected_mime);
                    }
                }
                Err(_) => {
                    println!("   ❌ {} - 无法获取文件信息", filename);
                }
            }
        }
        Err(e) => {
            println!("   ❌ {} - 上传失败: {}", filename, e);
        }
    }
}

/// 为不同格式创建示例数据
fn create_sample_data_for_format(filename: &str) -> Vec<u8> {
    let extension = filename.split('.').last().unwrap_or("").to_lowercase();
    
    match extension.as_str() {
        // 图片格式
        "jpg" | "jpeg" => create_jpeg_header(),
        "png" => create_png_header(),
        "gif" => create_gif_header(),
        "bmp" => create_bmp_header(),
        "svg" => create_svg_content(),
        
        // 视频格式
        "mp4" | "m4v" => create_mp4_header(),
        "avi" => create_avi_header(),
        "mov" => create_mov_header(),
        
        // 音频格式
        "mp3" => create_mp3_header(),
        "wav" => create_wav_header(),
        
        // 文本格式
        "txt" => b"Sample text content for format testing.".to_vec(),
        "html" | "htm" => b"<!DOCTYPE html><html><head><title>Test</title></head><body>Test</body></html>".to_vec(),
        "css" => b"body { margin: 0; padding: 0; }".to_vec(),
        "js" => b"console.log('Hello, World!');".to_vec(),
        "json" => b"{\"message\": \"Hello, World!\", \"format\": \"json\"}".to_vec(),
        "xml" => b"<?xml version=\"1.0\"?><root><message>Hello, World!</message></root>".to_vec(),
        "csv" => b"Name,Age,City\nJohn,25,New York\nJane,30,Los Angeles".to_vec(),
        "md" => b"# Sample Markdown\n\nThis is a **sample** markdown file.".to_vec(),
        
        // 默认二进制数据
        _ => format!("Sample {} file content for COS format testing.", extension).into_bytes(),
    }
}

/// 创建 JPEG 文件头
fn create_jpeg_header() -> Vec<u8> {
    vec![
        0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46, 0x00,
        0x01, 0x01, 0x01, 0x00, 0x48, 0x00, 0x48, 0x00, 0x00, 0xFF, 0xD9
    ]
}

/// 创建 PNG 文件头
fn create_png_header() -> Vec<u8> {
    vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG 签名
        0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, // IHDR chunk
        0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, // 1x1 像素
        0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53, 0xDE
    ]
}

/// 创建 GIF 文件头
fn create_gif_header() -> Vec<u8> {
    b"GIF89a\x01\x00\x01\x00\x00\x00\x00!\xF9\x04\x01\x00\x00\x00\x00,\x00\x00\x00\x00\x01\x00\x01\x00\x00\x02\x02\x04\x01\x00;".to_vec()
}

/// 创建 BMP 文件头
fn create_bmp_header() -> Vec<u8> {
    vec![
        0x42, 0x4D, // "BM"
        0x3A, 0x00, 0x00, 0x00, // 文件大小
        0x00, 0x00, 0x00, 0x00, // 保留
        0x36, 0x00, 0x00, 0x00, // 数据偏移
        0x28, 0x00, 0x00, 0x00, // 头大小
        0x01, 0x00, 0x00, 0x00, // 宽度
        0x01, 0x00, 0x00, 0x00, // 高度
        0x01, 0x00, 0x18, 0x00, // 平面数和位深度
    ]
}

/// 创建 SVG 内容
fn create_svg_content() -> Vec<u8> {
    b"<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"100\" height=\"100\"><circle cx=\"50\" cy=\"50\" r=\"40\" fill=\"red\"/></svg>".to_vec()
}

/// 创建 MP4 文件头
fn create_mp4_header() -> Vec<u8> {
    vec![
        0x00, 0x00, 0x00, 0x20, 0x66, 0x74, 0x79, 0x70, // ftyp box
        0x69, 0x73, 0x6F, 0x6D, 0x00, 0x00, 0x02, 0x00,
        0x69, 0x73, 0x6F, 0x6D, 0x69, 0x73, 0x6F, 0x32,
        0x61, 0x76, 0x63, 0x31, 0x6D, 0x70, 0x34, 0x31,
    ]
}

/// 创建 AVI 文件头
fn create_avi_header() -> Vec<u8> {
    b"RIFF\x24\x00\x00\x00AVI LIST\x1C\x00\x00\x00hdrlavih\x38\x00\x00\x00".to_vec()
}

/// 创建 MOV 文件头
fn create_mov_header() -> Vec<u8> {
    vec![
        0x00, 0x00, 0x00, 0x14, 0x66, 0x74, 0x79, 0x70, // ftyp
        0x71, 0x74, 0x20, 0x20, 0x20, 0x05, 0x03, 0x00,
        0x71, 0x74, 0x20, 0x20,
    ]
}

/// 创建 MP3 文件头
fn create_mp3_header() -> Vec<u8> {
    vec![
        0xFF, 0xFB, 0x90, 0x00, // MP3 帧头
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ]
}

/// 创建 WAV 文件头
fn create_wav_header() -> Vec<u8> {
    b"RIFF\x24\x00\x00\x00WAVE\x66\x6D\x74\x20\x10\x00\x00\x00\x01\x00\x01\x00\x44\xAC\x00\x00\x88\x58\x01\x00\x02\x00\x10\x00data\x00\x00\x00\x00".to_vec()
}