mod engine;
mod helper;

use helper::DynError;
use std::{
    env,
    fs::File,
    io::{BufRead, BufReader},
};

fn main() -> Result<(), DynError> {
    let args: Vec<String> = env::args().collect();
    if let (Some(expr), Some(file)) = (args.get(1), args.get(2)) {
        match_file(expr, file)?;
    } else {
        eprintln!("usage: {} regex file", args[0]);
        return Err("invalid arguments".into());
    }

    Ok(())
}

/// ファイルをオープンし、行ごとにマッチングを行う。
///
/// マッチングはそれぞれの行頭から1文字ずつずらして行い、
/// いずれかのマッチした場合に、その行がマッチしたものとみなす。
///
/// たとえば、abcdという文字列があった場合、以下の順にマッチが行われ、
/// このいずれかにマッチした場合、与えられた正規表現にマッチする行と判定する。
///
/// - abcd
/// - bcd
/// - cd
/// - d
fn match_file(expr: &str, file: &str) -> Result<(), DynError> {
    let f = File::open(file)?;
    let reader = BufReader::new(f);

    engine::print(expr)?;
    println!();

    for line in reader.lines() {
        let line = line?;
        for (i, _) in line.char_indices() {
            // UTF-8の文字単位でのindexを取得
            if engine::do_matching(expr, &line[i..], true)? {
                println!("{}", line);
                break;
            }
        }
    }

    Ok(())
}

// 単体テスト
#[cfg(test)]
mod tests {
    use crate::{
        engine::do_matching,
        helper::{SafeAdd, safe_add},
    };

    #[test]
    fn test_safe_add() {
        let n = 10;
        assert_eq!(Some(30), n.safe_add(&20));

        let n = !0;
        assert_eq!(None, n.safe_add(&1));

        let mut n = 10;
        assert!(safe_add(&mut n, &20, || ()).is_ok());

        let mut n = !0;
        assert!(safe_add(&mut n, &1, || ()).is_err());
    }

    #[test]
    fn test_matching() {
        // パースエラー
        assert!(do_matching("+b", "bbb", true).is_err());
        assert!(do_matching("*b", "bbb", true).is_err());
        assert!(do_matching("|b", "bbb", true).is_err());
        assert!(do_matching("?b", "bbb", true).is_err());

        // パース成功、マッチ成功
        assert!(do_matching("abc|def", "def", true).unwrap());
        assert!(do_matching("(abc)*", "abcabc", true).unwrap());
        assert!(do_matching("(ab|cd)+", "abcdcd", true).unwrap());
        assert!(do_matching("abc?", "ab", true).unwrap());
        assert!(do_matching("((((a*)*)*)*)", "aaaaaaaaa", true).unwrap());
        assert!(do_matching("(a*)*b", "aaaaaaaaab", true).unwrap());
        assert!(do_matching("(a*)*b", "b", true).unwrap());
        assert!(do_matching("a**b", "aaaaaaaaab", true).unwrap());
        assert!(do_matching("a**b", "b", true).unwrap());

        // パース成功、マッチ失敗
        assert!(!do_matching("abc|def", "efa", true).unwrap());
        assert!(!do_matching("(ab|cd)+", "", true).unwrap());
        assert!(!do_matching("abc?", "acb", true).unwrap());
    }
}
