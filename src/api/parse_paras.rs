use std::path::PathBuf;

use argh::FromArgs;

use crate::error::MyError;

#[derive(FromArgs)]
/// calculate token number
struct Paras {
    /// files to tiktoken, e.g. file1,file2,file3
    #[argh(option, short = 'f')]
    files: Option<String>,

    /// string to tiktoken
    #[argh(option, short = 's')]
    str: Option<String>,

    /// recursively traverse all files along the specified path
    #[argh(option, short = 'p')]
    path: Option<String>,

    /// encoding, support: o200k_base(GPT-4o models, o1 models), cl100k_base(ChatGPT models, text-embedding-ada-002), p50k_base(Code models, text-davinci-002, text-davinci-003), p50k_edit(edit models, text-davinci-edit-001, code-davinci-edit-001), r50k_base(GPT-3 models, davinci), default: o200k_base
    #[argh(option, short = 'e')]
    encoding: Option<String>,

    /// file size exceeding -m will not calculate token, support b, k, m, g, e.g. 26b, 78k, 98m, 4g, use 0b, 0k, 0m, 0g for unlimit, default: 10m
    #[argh(option, short = 'm')]
    max_size: Option<String>,

    /// files with fewer than -t tokens will be omitted from the output tree, only output [-t, -T], default: 0
    #[argh(option, short = 't')]
    token_low: Option<usize>,

    /// files exceeding -T tokens will be omitted from the output tree, 0 means unlimit, only output [-t, -T], default: 0
    #[argh(option, short = 'T')]
    token_hight: Option<usize>,

    /// omit invalid (e.g. binary files, large files, empty files, files containing invalid characters) files from the output tree
    #[argh(switch, short = 'd')]
    valid: bool,
}

/// 存储解析后的命令行参数
#[derive(Debug)]
pub struct ParsedParas {
    pub files:        Option<Vec<PathBuf>>, // 要计算token的文件，多个之间逗号间隔
    pub string:       Option<String>,       // 要计算token的字符串
    pub path:         Option<PathBuf>,      // 要递归的路径，程序会递归计算该路径下每个文件（自动排除二进制文件和大小超过10M的文件）的token数，并以tree的形式打印
    pub encoding:     String,               // 编码集，默认o200k_base
    pub max_size:     u64,                  // 指定文件大小上限，大小>-m的文件不计算token，但会包含在打印的tree中，支持4种单位b、k(1024b)、m(1024k)、g(1024m)，大小写都行，例如：15b、500k、200m、4g，默认10m，0表示无限制（此时单位无所谓）
    pub max_size_str: String,               // 指定文件大小上限的原始参数
    pub min_token:    usize,                // 指定token数下限，token数<-t的文件不会包含在打印的tree中，只输出token数在[-t, -T]范围内的文件，默认0
    pub max_token:    usize,                // 指定token数上限，token数>-T的文件不会包含在打印的tree中，0表示不限制，只输出token数在[-t, -T]范围内的文件，默认0
    pub only_valid:   bool,                 // 仅输出有效文件结果，二进制文件、大小超过-m的文件、含有非UTF-8字符的文件、空文件，将不会包含在打印结果中
}

/// 解析参数
pub fn parse_para() -> Result<ParsedParas, MyError> {
    let para: Paras = argh::from_env();
    // 解析文件大小上限
    let (max_size, max_size_str) = match para.max_size { // 指定文件大小上限，大小>-m的文件不计算token，支持4种单位b、k(1024b)、m(1024k)、g(1024m)，大小写都行，例如：15b、500k、200m、4g，默认10m，0表示无限制（此时单位无所谓）
        Some(m) => {
            let mut para_size = m.to_lowercase();
            match para_size.pop() {
                Some(p) => match para_size.parse::<u64>() { // 这里p是指定参数的最后一个字符
                    Ok(n) => match p { // 这里n是指定参数的数值
                        'b' => if n == 0 {
                            (u64::MAX, format!("{}Gb", u64::MAX/1024/1024/1024))
                        } else {
                            (n, format!("{n} bytes"))
                        },
                        'k' => if n == 0 {
                            (u64::MAX, format!("{}Gb", u64::MAX/1024/1024/1024))
                        } else {
                            (n*1024, format!("{n}Kb"))
                        },
                        'm' => if n == 0 {
                            (u64::MAX, format!("{}Gb", u64::MAX/1024/1024/1024))
                        } else {
                            (n*1024*1024, format!("{n}Mb"))
                        },
                        'g' => if n == 0 {
                            (u64::MAX, format!("{}Gb", u64::MAX/1024/1024/1024))
                        } else {
                            (n*1024*1024*1024, format!("{n}Gb"))
                        },
                        _ => return Err(MyError::ParaError{para: format!("-m suffix only support b, k, m, g, not {}", p)}),
                    },
                    Err(e) => return Err(MyError::ParseStringError{from: m.to_string(), to: "u64".to_string(), error: e}),
                },
                None => (10485760, "10Mb".to_string()), // 10M=10*1024*1024=10485760
            }
        },
        None => (10485760, "10Mb".to_string()), // 10M=10*1024*1024=10485760
    };
    // 其他参数
    let out: ParsedParas = ParsedParas{
        files: match para.files { // 要计算token的文件，多个之间逗号间隔
            Some(f) => {
                let mut tmp_files: Vec<PathBuf> = vec![];
                for i in f.split(",") {
                    let tmp_file = PathBuf::from(i);
                    if !(tmp_file.exists() && tmp_file.is_file()) {
                        return Err(MyError::FileNotExistError{file: i.to_string()})
                    }
                    tmp_files.push(tmp_file);
                }
                Some(tmp_files)
            },
            None => None,
        },
        string: para.str, // 要计算token的字符串
        path: match para.path { // 要递归的路径，程序会递归计算该路径下每个文件（自动排除二进制文件和大小超过10M的文件）的token数，并以tree的形式打印
            Some(p) => {
                let tmp_path = PathBuf::from(&p);
                if !(tmp_path.exists() && tmp_path.is_dir()) {
                    return Err(MyError::DirNotExistError{dir: p})
                }
                Some(tmp_path)
            },
            None => None,
        },
        encoding: match para.encoding { // 编码集，默认o200k_base
            Some(e) => {
                if ["o200k_base", "cl100k_base", "p50k_base", "p50k_edit", "r50k_base"].iter().any(|x| x == &e) {
                    e
                } else {
                    return Err(MyError::ParaError{para: format!("-e only support o200k_base, cl100k_base, p50k_base, p50k_edit, r50k_base, not: {}", e)})
                }
            },
            None => "o200k_base".to_string(),
        },
        max_size, // 指定文件大小上限，大小>-m的文件不计算token，支持4种单位b、k(1024b)、m(1024k)、g(1024m)，大小写都行，例如：15b、500k、200m、4g，默认10m，0表示无限制（此时单位无所谓）
        max_size_str, // 指定文件大小上限的原始参数
        min_token: match para.token_low { // 指定token数下限，token数<-t的文件不会包含在打印的tree中，只输出token数在[-t, -T]范围内的文件，默认0
            Some(t) => t,
            None => 0,
        },
        max_token: match para.token_hight { // 指定token数上限，token数>-T的文件不会包含在打印的tree中，0表示不限制，只输出token数在[-t, -T]范围内的文件，默认0
            Some(t) => if t == 0 {
                usize::MAX
            } else {
                t
            },
            None => usize::MAX,
        },
        only_valid: para.valid, // 仅输出有效文件结果，二进制文件、大小超过-m的文件、含有非UTF-8字符的文件、空文件，将不会包含在打印结果中
    };
    // -f、-s、-p必须至少指定1个
    if out.files.is_none() && out.string.is_none() && out.path.is_none() {
        return Err(MyError::ParaError{para: "must specify -f or -s or -p".to_string()});
    }
    Ok(out)
}
