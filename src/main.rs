use ansi_term::Colour;
use atty;
use clap::{Arg, ArgAction, ArgMatches, Command};
use rayon;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use regex::Regex;
use std::env::current_dir;
use std::fs::File;
use std::io::{BufRead, BufReader};
use walkdir::WalkDir;

/// 自分用grep
fn main() -> Result<(), Box<dyn std::error::Error>> {
   let matches = get_command_matches();
   let pattern = get_pattern(&matches);
   branch_atty(&matches, &pattern)?;
   Ok(())
}

/// -------------------------------Utility start --------------------------------
/// ファイルパスを取得する
/// common
///
/// # Arguments
/// * `matches` - コマンドライン引数
///
/// # Returns
/// ファイルパス
fn get_file_path(matches: &ArgMatches) -> Option<&String> {
   let file_path = matches.get_one::<String>("file");
   file_path
}

/// コマンドライン引数を取得する
/// common
///
/// # Returns
/// コマンドライン引数
fn get_command_matches() -> ArgMatches {
   let args_matches = Command::new("g")
      .about("My grep tool")
      .arg(Arg::new("pattern").required(true).index(1))
      .arg(Arg::new("file").required(false).index(2))
      .arg(
         Arg::new("number")
            .help("行ナンバーを表示する")
            .short('n')
            .long("number")
            .action(ArgAction::SetTrue),
      )
      .arg(
         Arg::new("read_file")
            .help(
               "複数のファイルパスを渡した時中身まで読むか？",
            )
            .short('o')
            .long("open")
            .action(ArgAction::SetTrue),
      )
      .arg(
         Arg::new("match_case")
            .help("大文字小文字を区別するか？")
            .short('m')
            .long("match")
            .action(ArgAction::SetTrue),
      )
      .try_get_matches();

   match args_matches {
      Ok(args) => args,
      Err(_) => {
         println!("正規表現パターンは必ず必要です");
         std::process::exit(1);
      }
   }
}

/// -------------------------------Utility end--------------------------------
/// パターンを取得する
/// common
///
/// # Arguments
/// * `matches` - コマンドライン引数
///
/// # Returns
/// パターン
fn get_pattern(matches: &ArgMatches) -> Regex {
   match matches.get_one::<String>("pattern") {
      Some(p) => {
         // ignore case ?
         let pattern;
         if matches.get_flag("match_case") {
            pattern = p.to_string();
         } else {
            pattern = format!("(?i){}", p);
         }
         if let Ok(pattern) = Regex::new(&pattern) {
            pattern
         } else {
            println!("Error: 正規表現を作成できませんでしたパターンを見直してください");
            std::process::exit(2);
         }
      }
      None => {
         println!("Error: パターンが指定されていません");
         std::process::exit(2);
      }
   }
}

/// パイプかファイルかを判定する
/// # Arguments
/// * `matches` - コマンドライン引数
///
/// # Returns
/// パイプの場合はその内容をVec<String>で返す
/// ファイルの場合はファイルを開きその内容をVec<String>で返す
/// `Result<Vec<String>, std::io::Error>`
fn branch_atty(
   matches: &ArgMatches,
   pattern: &Regex,
) -> Result<(), Box<dyn std::error::Error>> {
   // パイプ入力の場合はTrue
   if atty::isnt(atty::Stream::Stdin) {
      input_pipe_pattern(&matches, &pattern)?;
   } else {
      input_file_pattern(&matches, &pattern)?;
   }
   Ok(())
}

///　ファイルで入力するパターン
/// # Arguments
/// * `matches` - コマンドライン引数
/// * `pattern` - パターン
fn input_file_pattern(
   matches: &ArgMatches,
   pattern: &Regex,
) -> Result<(), Box<dyn std::error::Error>> {
   // ファイル入力
   let file_path = get_file_path(matches);
   match file_path {
      Some(file_path) => {
         let file = File::open(file_path);
         match file {
            Ok(file) => {
               let reader = BufReader::new(file);
               let lines_tmp = reader
                  .lines()
                  .collect::<Result<Vec<String>, _>>()?;
               print_display(
                  &matches, &pattern, &lines_tmp, None,
               );
            }
            Err(_e) => {}
         }
      }
      None => {
         let file_pathes = get_subdir_files();
         read_and_display_files(
            &matches,
            &pattern,
            &file_pathes,
         );
      }
   }
   Ok(())
}

///　現在より下の階層をすべてのファイルを探査
fn get_subdir_files() -> Vec<String> {
   let cur_dir = current_dir()
      .expect("カレントパスを取得できませんでした");
   let walk_dir = WalkDir::new(cur_dir);
   let mut path_lst = Vec::<String>::new();
   for it in walk_dir.into_iter() {
      match it {
         Ok(i) => {
            if let Some(p) = i.path().to_str() {
               path_lst.push(p.to_string());
            } else {
               continue;
            }
         }
         Err(_) => {
            continue;
         }
      }
   }
   path_lst
}

/// パイプで入力するパターン
/// # Arguments
/// * `matches` - コマンドライン引数
/// * `pattern` - パターン
fn input_pipe_pattern(
   matches: &ArgMatches,
   pattern: &Regex,
) -> Result<(), Box<dyn std::error::Error>> {
   // パイプ入力
   let stdin = std::io::stdin();
   let bufer = stdin.lock();
   let lines_tmp =
      bufer.lines().collect::<Result<Vec<String>, _>>()?;
   // パイプ入力でファイルをリードする場合
   if matches.get_flag("read_file") {
      read_and_display_files(&matches, &pattern, &lines_tmp);
   } else {
      // moji wo sonomama syuturyokusurubaai
      print_display(&matches, &pattern, &lines_tmp, None);
   }
   Ok(())
}

// ファイルを読み込んで画像に出力する処理
fn read_and_display_files(
   matches: &ArgMatches,
   pattern: &Regex,
   lines_tmp: &[String],
) {
   lines_tmp.par_iter().for_each(|line| {
      // ファイルネームを出力
      let file = File::open(&line);
      match file {
         Ok(file) => {
            let reader = BufReader::new(file);
            let lines_tmp = reader
               .lines()
               .collect::<Result<Vec<String>, _>>();
            match lines_tmp {
               Ok(lines_tmp) => {
                  print_display(
                     &matches,
                     &pattern,
                     &lines_tmp,
                     Some(&line),
                  );
               }
               Err(_e) => {}
            }
         }
         Err(_e) => {}
      }
   });
}

/// 画面に出力する
/// # Arguments
/// * `matches` - コマンドライン引数
/// * `pattern` - パターン
/// * `str_vec` - ファイルの内容
///   
/// # Returns
/// パターンにマッチした行を表示する
fn print_display(
   matches: &ArgMatches,
   pattern: &Regex,
   str_vec: &Vec<String>,
   file_name: Option<&String>,
) {
   // パイプ出力の場合 True
   let atty_out_flag = atty::isnt(atty::Stream::Stdout);
   // 一度目のフラグ
   let mut flag_one_gate: bool = false;
   let flag_number = matches.get_flag("number");
   let mut index = 1;
   // ouput
   for str in str_vec {
      if pattern.is_match(str) {
         if !flag_one_gate {
            flag_one_gate = true;
            // ファイル名の出力
            if let Some(f) = file_name {
               let f = f.as_str();
               let f = Colour::Cyan.paint(f).to_string();
               println!("\n\n{}", f);
            }
         }
         // パイプ出力
         if atty_out_flag {
            if flag_number {
               println!("{}: {}", index, &str);
            } else {
               println!("{}", &str);
            }
         } else {
            // パイプ出力ではない
            let rep = if let Some(s) = pattern.find(str) {
               s
            } else {
               continue;
            };
            let rep = rep.as_str();
            let red_str = Colour::Red.paint(rep).to_string();
            if flag_number {
               println!(
                  "{}: {}",
                  index,
                  str.replace(rep, &red_str)
               );
            } else {
               println!("{}", str.replace(rep, &red_str));
            }
         }
      }
      index += 1;
   }
}
