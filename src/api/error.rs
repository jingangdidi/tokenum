use std::io;
use std::num::ParseIntError;
use std::string::FromUtf8Error;

use thiserror::Error;

/// srx添加，自定义的错误类型，方便传递错误
/// 参考：https://github.com/dtolnay/thiserror
/// 参考：https://crates.io/crates/thiserror
/// 参考：https://juejin.cn/post/7272005801081126968
/// 参考：https://www.shakacode.com/blog/thiserror-anyhow-or-how-i-handle-errors-in-rust-apps/
/// 参考：https://rustcc.cn/article?id=1e20f814-c7d5-4aca-bb67-45dcfb65d9f9
#[derive(Debug, Error)]
pub enum MyError {
    // 读取文件错误
    #[error("Error - fs::read {file}: {error}")]
    ReadFileError{file: String, error: io::Error},

    // 打开文件错误
    #[error("Error - fs::File::open {file}: {error}")]
    OpenFileError{file: String, error: io::Error},

    // 创建文件错误
    #[error("Error - fs::create {file}: {error}")]
    CreateFileError{file: String, error: io::Error},

    // 创建路径错误
    #[error("Error - fs::create_dir_all {dir_name}: {error}")]
    CreateDirAllError{dir_name: String, error: io::Error},

    // 创建文件(一次写入)错误
    #[error("Error - fs::write {file}: {error}")]
    WriteFileError{file: String, error: io::Error},

    // 按行读取文件错误
    #[error("Error - read lines {file}: {error}")]
    LinesError{file: String, error: io::Error},

    // 获取指定路径下所有项错误
    #[error("Error - read_dir {dir}: {error}")]
    ReadDirError{dir: String, error: io::Error},

    // 删除文件夹错误
    #[error("Error - fs::remove_dir {dir}: {error}")]
    RemoveDirError{dir: String, error: io::Error},

    // 删除文件错误
    #[error("Error - fs::remove_file {file}: {error}")]
    RemoveFileError{file: String, error: io::Error},

    // 读取文件内容为字符串错误
    #[error("Error - read {file} to string: {error}")]
    ReadFileToStringError{file: String, error: io::Error},

    // 字符串转指定类型错误
    #[error("Error - parse {from} -> {to}: {error}")]
    ParseStringError{from: String, to: String, error: ParseIntError},

    // 路径不存在
    #[error("Error - {dir} does not exist")]
    DirNotExistError{dir: String},

    // 文件不存在
    #[error("Error - {file} does not exist")]
    FileNotExistError{file: String},

    // 读取文件转为UTF-8错误
    #[error("Error - {file} to UTF-8: {error}")]
    FileContentToUtf8Error{file: String, error: FromUtf8Error},

    // Tokenizer错误
    #[error("Error - Initialize {tokenizer} tokenizer: {error}")]
    TokenizerError{tokenizer: String, error: anyhow::Error},

    // 参数使用错误
    #[error("Error - {para}")]
    ParaError{para: String},

    // 常规io::Error，这里可以改为向上面那样将错误传过来，但不知道还能否使用`#[from]`
    #[error("I/O error occurred")]
    IoError(#[from] io::Error),
}
