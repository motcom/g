use atty;
use clap::{Arg, ArgAction, ArgMatches, Command};
use regex::Regex;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};

const DEBUG: bool = true;

fn debug() {
   get_file_vector();
}

fn get_file_vector() {
   let _file_vector: Vec<File> = Vec::new();

   // パイプ入力の場合はTrue
   if atty::isnt(atty::Stream::Stdin) {
      // パイプ入力
      println!("not atty");
   } else {
      // ファイル入力
      println!("is atty");
   }
}

/// 自分用grep
fn main() {
   if DEBUG {
      debug()
   } else {
      match main_exe() {
         Ok(_) => {}
         Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
         }
      }
   }
}

/// メイン処理
/// common
///
/// # Returns
/// 正常終了
fn main_exe() -> Result<(), Box<dyn std::error::Error>> {
   let matches = get_command_matches();
   let pattern = get_pattern(&matches);

   Ok(())
}

/// コマンドライン引数を取得する
/// common
///
/// # Returns
/// コマンドライン引数
fn get_command_matches() -> ArgMatches {
   Command::new("g")
      .about("My grep tool")
      .arg(Arg::new("pattern").required(true).index(1))
      .arg(Arg::new("file").required(false).index(2))
      .arg(
         Arg::new("no_number")
            .help("行ナンバーを表示しない")
            .short('n')
            .long("nonumber")
            .action(ArgAction::SetTrue),
      )
      .arg(
         Arg::new("read_file")
            .help(
               "複数のファイルパスを渡した時中身まで読むか？",
            )
            .short('r')
            .long("read")
            .action(ArgAction::SetTrue),
      )
      .arg(
         Arg::new("match_case")
            .help("大文字小文字を区別するか？")
            .short('i')
            .long("match")
            .action(ArgAction::SetTrue),
      )
      .get_matches()
}

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
         if let Ok(pattern) = Regex::new(p) {
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
