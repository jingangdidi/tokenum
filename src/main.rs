use tokenum::{
    parse_paras::parse_para,
    error::MyError,
    token::calculate_token,
};

fn main() {
    if let Err(e) = run() {
        println!("{}", e); // 这里不要用`{:?}`，会打印结构体而不是打印指定的错误信息
    }
}

fn run() -> Result<(), MyError> {
    // 解析参数
    let paras = parse_para()?;

    // 计算token
    calculate_token(
        paras.files,
        paras.string,
        paras.path,
        &paras.encoding,
        paras.max_size,
        &paras.max_size_str,
        paras.min_token,
        paras.max_token,
        paras.only_valid,
    )
}
