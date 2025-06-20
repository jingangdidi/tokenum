use std::collections::HashMap;
use std::path::Path;

use ignore::WalkBuilder;
use termtree::Tree;
use tiktoken_rs::CoreBPE;

use crate::{
    token::{
        FileToken,
        get_file_size,
    },
    error::MyError,
};

/// 递归获取指定项目路径下所有文件
pub fn traverse_directory(
    root_path: &Path,
    bpe: CoreBPE,
    max_size: u64,
    max_size_str: &str,
    min_token: usize,
    max_token: usize,
    only_valid: bool
) -> Result<String, MyError> {
    // 初始化
    let canonical_root_path = root_path.canonicalize()?; // 获取绝对路径
    let parent_prefix = canonical_root_path.parent().unwrap(); // 父路径，作为后面每个路径要去除的前缀
    let parent_directory = match &canonical_root_path.file_name() { // 获取指定路径的文件夹名，`file_name`获取指定path的最后一项
        Some(name) => name.to_string_lossy().to_string(), // 返回指定path的最后一项，可能是文件，也可能是文件夹
        None => canonical_root_path.to_str().unwrap().to_string(), // 指定的path是`/`或以`..`结尾时`file_name`会返回None，此时直接返回指定的path字符串
    };
    let mut file_size: u64 = 0; // 存储文件大小
    let mut idx = 0; // 每个路径的id
    let mut tokens = 0; // 计算的文件token数
    let mut dir_tokens: HashMap<usize, (String, usize, u64)> = HashMap::from([(0, (parent_directory.clone(), 0, 0))]); // key: 每个路径的id，value: (该路径去除前缀后的路径字符串, 该路径下所有文件的总token数, 该路径下所有文件的总大小)
    // 创建tree
    let tree = WalkBuilder::new(&canonical_root_path)
        .git_ignore(true)
        .build()
        .filter_map(|e| e.ok())
        .fold(Tree::new(parent_directory.to_owned()+" srx0"), |mut root, entry| { // 遍历指定路径下每一项，以指定路径作为根路径，递归添加子项
            let path = entry.path(); // 当前项的路径
            if let Ok(relative_path) = path.strip_prefix(&canonical_root_path) { // 获取相对路径
                // 递归获取指定路径下所有项，创建树结构，用于显示在生成文件的起始
                let mut current_tree = &mut root; // 当前树结构可变引用
                let relative_path_components = relative_path.components();
                for component in relative_path_components.clone() { // 遍历当前相对路径的每个父文件夹
                    let component_str = component.as_os_str().to_string_lossy().to_string(); // 转为String
                    // 从当前树结构中获取当前component节点的可变引用，不存在则插入
                    current_tree = if let Some(pos) = current_tree
                        .leaves // Vec<Tree>
                        .iter_mut() // 遍历当前树结构中每个叶子节点
                        //.position(|child| child.root == component_str) // 获取叶子节点的父节点与当前component相同的节点在Vec<Tree>中的索引
                        .position(|child| {
                            if child.root.contains(" srx") {
                                child.root.rsplit_once(" srx").unwrap().0 == component_str
                            } else {
                                child.root == component_str
                            }
                        }) // 获取叶子节点的父节点与当前component相同的节点在Vec<Tree>中的索引
                    {
                        &mut current_tree.leaves[pos] // 找到pos索引，则当前树结构更新为以该叶子节点为root的树结构，返回可变引用
                    } else { // 此时说明当前component不在当前树结构中
                        // 当前项是文件，且不是二进制文件，且大小在10M范围内
                        let component_str_token = if path.is_file() {
                            // 判断文件大小
                            file_size = path.metadata().unwrap().len();
                            if file_size <= max_size {
                                let mut file_token = FileToken::new(path, file_size);
                                if file_token.not_binary() {
                                    if file_token.string.is_empty() {
                                        if only_valid {
                                            continue
                                        }
                                        format!("{} ({}, 0 token)", component_str, file_token.size)
                                    } else {
                                        // 代码中含有无效UTF-8字符则报错，REPLACEMENT_CHARACTER表示无效字符“�”
                                        if file_token.string.contains(char::REPLACEMENT_CHARACTER) {
                                            if only_valid {
                                                continue
                                            }
                                            //println!("[warning]: invalid UTF-8 in {}", relative_path.display());
                                            format!("{} ({}, contain invalid UTF-8)", component_str, file_token.size)
                                        } else {
                                            tokens = bpe.encode_with_special_tokens(&file_token.string).len(); // 计算该文件token数
                                            if tokens < min_token || tokens > max_token { // 该文件token数超不在指定上下限范围内，则不写入tree中
                                                continue
                                            }
                                            let rltv_path = path.strip_prefix(parent_prefix).unwrap(); // 当前文件路径去除前缀
                                            for i in 0..=idx { // 遍历已访问的每个路径，如果该路径是当前文件的父级路径，则该路径总大小和总token数要加上当前文件的大小和token数
                                                if rltv_path.starts_with(&dir_tokens.get(&i).unwrap().0) {
                                                    dir_tokens.get_mut(&i).unwrap().1 += tokens;
                                                    dir_tokens.get_mut(&i).unwrap().2 += file_size;
                                                }
                                            }
                                            format!("{} ({}, {} tokens)", component_str, file_token.size, tokens)
                                        }
                                    }
                                } else {
                                    if only_valid {
                                        continue
                                    }
                                    let rltv_path = path.strip_prefix(parent_prefix).unwrap(); // 当前文件路径去除前缀
                                    for i in 0..=idx { // 遍历已访问的每个路径，如果该路径是当前文件的父级路径，则该路径总大小要加上当前文件的大小
                                        if rltv_path.starts_with(&dir_tokens.get(&i).unwrap().0) {
                                            dir_tokens.get_mut(&i).unwrap().2 += file_size;
                                        }
                                    }
                                    //println!("[skip]: this file might be a binary file: {}", relative_path.display());
                                    format!("{} ({}, binary file)", component_str, file_token.size)
                                }
                            } else {
                                if only_valid {
                                    continue
                                }
                                let rltv_path = path.strip_prefix(parent_prefix).unwrap(); // 当前文件路径去除前缀
                                for i in 0..=idx { // 遍历已访问的每个路径，如果该路径是当前文件的父级路径，则该路径总大小要加上当前文件的大小
                                    if rltv_path.starts_with(&dir_tokens.get(&i).unwrap().0) {
                                        dir_tokens.get_mut(&i).unwrap().2 += file_size;
                                    }
                                }
                                //println!("[skip]: file size {} > {}, {}", file_size, max_size, relative_path.display());
                                format!("{} ({}, file size {} bytes > {})", component_str, get_file_size(file_size), file_size, max_size_str)
                            }
                        } else {
                            idx += 1;
                            let rltv_path = path.strip_prefix(parent_prefix).unwrap();
                            dir_tokens.insert(idx, (rltv_path.to_str().unwrap().to_string(), 0, 0));
                            format!("{} srx{}", component_str, idx) // 这里在路径后面加上` srx编号`，例如` srx0`、` srx1`，最后会根据这个idx编号从dir_tokens中获取该路径的总token数
                        };
                        let new_tree = Tree::new(component_str_token); // 以当前component（如果是文件，则后面加上了token数或没有计算token的原因，如果是文件夹则仅有名称）创建新的tree
                        current_tree.leaves.push(new_tree); // 将刚创建的tree作为叶子节点加入到当前树结构中
                        current_tree.leaves.last_mut().unwrap() // 返回当前树结构中新增的节点的可变引用
                    };
                }
            }
            root
        });
    //println!("{:?}", dir_tokens);
    let mut out: Vec<String> = vec![];
    for i in tree.to_string().split("\n") { // 遍历输出字符串tree的每行，根据其中路径后面的id获取相应总token数
        if i.contains(" srx") {
            let tmp = i.rsplit_once(" srx").unwrap();
            if let Ok(id) = tmp.1.parse::<usize>() {
                let num = dir_tokens.get(&id).unwrap().1; // 获取该路径下所有文件token总数
                if num == 0 {
                    out.push(format!("{} ({}, total 0 token)", tmp.0, get_file_size(dir_tokens.get(&id).unwrap().2))); // 在输出的tree字符串中该路径后面加上文件大小之和以及总token数
                } else {
                    out.push(format!("{} ({}, total {} tokens)", tmp.0, get_file_size(dir_tokens.get(&id).unwrap().2), num)); // 在输出的tree字符串中该路径后面加上文件大小之和以及总token数
                }
            } else {
                out.push(i.to_string()); // 提取usize数值报错，则输出原始内容
            }
        } else {
            out.push(i.to_string());
        }
    }
    Ok(out.join("\n"))
}
