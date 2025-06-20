use std::fs::read;
use std::path::{Path, PathBuf};

use tiktoken_rs::{
    o200k_base, // GPT-4o models
    cl100k_base, // ChatGPT models text-embedding-ada-002
    p50k_base, // Code models, text-davinci-002, text-davinci-003
    p50k_edit, // edit models like text-davinci-edit-001, code-davinci-edit-001
    r50k_base, // GPT-3 models like davinci, also known as gpt2
    CoreBPE,
};

use crate::{
    error::MyError,
    traverse::traverse_directory,
};

/// 根据指定编码类型，返回CoreBPE对象
fn get_tokenizer(encoding: &str) -> Result<CoreBPE, MyError> {
    match encoding {
        "o200k_base" => match o200k_base() {
            Ok(e) => Ok(e),
            Err(e) => Err(MyError::TokenizerError{tokenizer: "o200k_base".to_string(), error: e}),
        },
        "cl100k_base" => match cl100k_base() {
            Ok(e) => Ok(e),
            Err(e) => Err(MyError::TokenizerError{tokenizer: "cl100k_base".to_string(), error: e}),
        },
        "p50k_base" => match p50k_base() {
            Ok(e) => Ok(e),
            Err(e) => Err(MyError::TokenizerError{tokenizer: "cl100k_base".to_string(), error: e}),
        },
        "p50k_edit" => match p50k_edit() {
            Ok(e) => Ok(e),
            Err(e) => Err(MyError::TokenizerError{tokenizer: "cl100k_base".to_string(), error: e}),
        },
        "r50k_base" | "gpt2" => match r50k_base() {
            Ok(e) => Ok(e),
            Err(e) => Err(MyError::TokenizerError{tokenizer: "cl100k_base".to_string(), error: e}),
        },
        _ => match o200k_base() {
            Ok(e) => Ok(e),
            Err(e) => Err(MyError::TokenizerError{tokenizer: "o200k_base".to_string(), error: e}),
        },
    }
}

/// 计算token
pub fn calculate_token(
    files: Option<Vec<PathBuf>>,
    string: Option<String>,
    path: Option<PathBuf>,
    encoding: &str,
    max_size: u64,
    max_size_str: &str,
    min_token: usize,
    max_token: usize,
    only_valid: bool,
) -> Result<(), MyError> {
    let bpe = get_tokenizer(encoding)?;
    let mut num: usize;
    // 指定的文件
    if let Some(files) = files {
        let mut file_size: u64;
        for f in files {
            file_size = f.metadata().unwrap().len();
            if file_size <= max_size {
                let mut file_token = FileToken::new(&f, file_size);
                if file_token.not_binary() {
                    if file_token.string.is_empty() {
                        if only_valid {
                            continue
                        }
                        println!("{} ({}, 0 token)", f.display(), file_token.size);
                    } else {
                        // 代码中含有无效UTF-8字符则报错，REPLACEMENT_CHARACTER表示无效字符“�”
                        if file_token.string.contains(char::REPLACEMENT_CHARACTER) {
                            if only_valid {
                                continue
                            }
                            println!("{} ({}, contain invalid UTF-8)", f.display(), file_token.size);
                        } else {
                            num = bpe.encode_with_special_tokens(&file_token.string).len();
                            if num >= min_token && num <= max_token {
                                println!("{} ({}, {} tokens)", f.display(), file_token.size, num);
                            }
                        }
                    }
                } else {
                    println!("{} ({}, binary file)", f.display(), file_token.size);
                }
            } else {
                println!("{} ({}, file size {} bytes > {})", f.display(), get_file_size(file_size), file_size, max_size_str);
            }
        }
    }
    // 指定的字符串
    if let Some(s) = string {
        num = bpe.encode_with_special_tokens(&s).len();
        println!("-s string: {} tokens", num);
    }
    // 指定的路径
    if let Some(p) = path {
        let tree = traverse_directory(&p, bpe, max_size, max_size_str, min_token, max_token, only_valid)?;
        println!("{}", tree);
    }
    Ok(())
}

/// 检查文件是否有效
pub struct FileToken {
    raw: Vec<u8>, // 文件原始内容
    pub size: String, // 转为合适单位的文件大小
    pub string: String, // 转为的字符串
}

impl FileToken {
    /// 读取文件，创建对象
    pub fn new(f: &Path, size: u64) -> Self {
        FileToken{
            raw: read(f).unwrap(),
            size: get_file_size(size),
            string: "".to_string(),
        }
    }

    /// 检查前50个byte判断是否是二进制文件，如果不是二进制文件则将raw转为String
    /// 读取文件开头指定数量byte，判断是否`<=0x08`（在ASCII码中该字符及其前面的字符不会出现在文本文件中），满足则说明该文件是二进制文件
    /// https://github.com/dalance/amber/blob/master/src/pipeline_matcher.rs
    pub fn not_binary(&mut self) -> bool {
        let mut not_binary = true;
        for byte in self.raw.iter().take(50) { // 判断前50个byte
            if byte <= &0x08 {
                //println!("{:?}, {:?}", &0x08, byte);
                not_binary = false;
                break;
            }
        }
        if not_binary {
            self.string = String::from_utf8_lossy(&self.raw).to_string(); // 转为UTF-8
        }
        not_binary
    }
}

/// 获取文件大小字符串，转为合适的单位
pub fn get_file_size(size: u64) -> String {
    if size > 1073741824 { // 1Gb = 1024*1024*1024 = 1073741824
        format!("{:.2}Gb", size_convert(size, 1073741824))
    } else if size > 1048576 { // 1Mb = 1024*1024 = 1048576
        format!("{:.2}Mb", size_convert(size, 1048576))
    } else if size > 1024 { // 1Kb = 1024
        format!("{:.2}Kb", size_convert(size, 1024))
    } else {
        format!("{} bytes", size)
    }
}

/// u64直接转f64可能溢出，先计算整数部分，再计算小数部分
fn size_convert(size: u64, div: u64) -> f64 {
    let left = size / div; // 整数部分
    let right = (size - div * left) as f64 / div as f64; // 小数部分
    left as f64 + right
}
