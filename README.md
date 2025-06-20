# tokenum

**calculate file or string token number**

**计算文件、指定路径下每个文件、字符串的token数**

## Arguments
```
Usage: tokenum.exe [-f <files>] [-s <str>] [-p <path>] [-e <encoding>] [-m <max-size>] [-t <token-low>] [-T <token-hight>] [-d]

calculate token number

Options:
  -f, --files       files to tiktoken, e.g. file1,file2,file3
  -s, --str         string to tiktoken
  -p, --path        recursively traverse all files along the specified path
  -e, --encoding    encoding, support: o200k_base(GPT-4o models, o1 models), cl100k_base(ChatGPT models, text-embedding-ada-002), p50k_base(Code models, text-davinci-002, text-davinci-003), p50k_edit(edit models, text-davinci-edit-001, code-davinci-edit-001), r50k_base(GPT-3 models, davinci), default: o200k_base
  -m, --max-size    file size exceeding -m will not calculate token, support b, k, m, g, e.g. 26b, 78k, 98m, 4g, use 0b, 0k, 0m, 0g for unlimit, default: 10m
  -t, --token-low   files with fewer than -t tokens will be omitted from the output tree, only output [-t, -T], default: 0
  -T, --token-hight files exceeding -T tokens will be omitted from the output tree, 0 means unlimit, only output [-t, -T], default: 0
  -d, --valid       omit invalid (e.g. binary files, large files, empty files, files containing invalid characters) files from the output tree
  --help, help      display usage information
```

## download pre-built binary

[latest release](https://github.com/jingangdidi/tokenum/releases)

## Usage
**1. calculate files tokens**
```
tokenum -f test1.txt,test2.txt

# +------------------------------------------------------+
# | test1.txt (10.62Kb, 4197 tokens)                     |
# | test2.txt (10.32Mb, file size 10828960 bytes > 10Mb) |
# +------------------------------------------------------+
```
**2. calculate string tokens**
```
tokenum -s "The Vec type allows access to values by index"

# +---------------------+
# | -s string: 9 tokens |
# +---------------------+
```
**3. calculate the number of tokens for all files in the specified path**
```
tokenum -p ./test

# +---------------------------------------------------------------+
# | test (19.59Mb, total 9624 tokens)                             |
# | ├── readme (8.23Kb, 2024 tokens)                              |
# | ├── tokenum (22.81Kb, total 7600 tokens)                      |
# | │   ├── Cargo.lock (10.62Kb, 4197 tokens)                     |
# | │   ├── Cargo.toml (382 bytes, 143 tokens)                    |
# | │   └── src (11.82Kb, total 3260 tokens)                      |
# | │       ├── api (11.20Kb, total 3102 tokens)                  |
# | │       │   ├── error.rs (2.88Kb, 852 tokens)                 |
# | │       │   ├── mod.rs (68 bytes, 18 tokens)                  |
# | │       │   ├── parse_paras.rs (8.25Kb, 2232 tokens)          |
# | │       │   ├── token.rs (6.01Kb, contain invalid UTF-8)      | this file contains invalid UTF-8 character, ignore
# | │       │   └── traverse.rs (10.67Kb, contain invalid UTF-8)  | this file contains invalid UTF-8 character, ignore
# | │       ├── lib.rs (29 bytes, 8 tokens)                       |
# | │       └── main.rs (605 bytes, 150 tokens)                   |
# | ├── tokenum-ubuntu (10.33Mb, file size 10828960 bytes > 10Mb) | this file larger than 10Mb, ignore
# | └── tokenum.exe (9.23Mb, binary file)                         | this file is a binary file, ignore
# +---------------------------------------------------------------+
```
**4. calculate the number of tokens for all files in the specified path, and use `-t` and `-T` to specify that only files within the range of `[100, 1000]` tokens should be displayed. Use `-d` to not display or count binary files, files larger than 10Mb, files containing invalid UTF-8 characters, and empty files**
```
tokenum -p ./test -t 100 -T 1000 -d

# +-----------------------------------------------+
# | test (3.85Kb, total 1145 tokens)              |
# | └── tokenum (3.85Kb, total 1145 tokens)       |
# |     ├── Cargo.toml (382 bytes, 143 tokens)    |
# |     └── src (3.47Kb, total 1002 tokens)       |
# |         ├── api (2.88Kb, total 852 tokens)    |
# |         │   └── error.rs (2.88Kb, 852 tokens) |
# |         ├── lib.rs (29 bytes, 8 tokens)       |
# |         └── main.rs (605 bytes, 150 tokens)   |
# +-----------------------------------------------+
```

## Building from source
```
git clone https://github.com/jingangdidi/tokenum.git
cd tokenum
cargo build --release
```
