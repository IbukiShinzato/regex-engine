# regex
Rustで正規表現エンジンを自作するプロジェクトです。
現在はパーサを実装し、抽象構文木（AST）の可視化に対応しています。

## 実行結果
正規表現をパースして、抽象構文木（AST）をツリー構造で出力できるようにしました。

### 修正前（標準の Debug 出力）
```
❯ cargo run "a|b|c|d|e"
   Compiling regex v0.1.0 (/Users/sitz_bnk21/Rust/regex)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.27s
     Running `target/debug/regex 'a|b|c|d|e'`
Or(Seq([Char('a')]), Or(Seq([Char('b')]), Or(Seq([Char('c')]), Or(Seq([Char('d')]), Seq([Char('e')])))))
```

### 修正後（Display トレイトによるツリー構造）
```
❯ cargo run "a|b|c|d|e"
   Compiling regex v0.1.0 (/Users/sitz_bnk21/Rust/regex)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.55s
     Running `target/debug/regex 'a|b|c|d|e'`
Or
├─Char(a)
└─Or
  ├─Char(b)
  └─Or
    ├─Char(c)
    └─Or
      ├─Char(d)
      └─Char(e)
```

## 出典

本リポジトリは、書籍『ゼロから学ぶRust』（Yuuki Takano著）および
[公開リポジトリ](https://github.com/ytakano/rust_zero) に基づき、一部コードを拡張・改良しています。

元コードはMITライセンスで提供されています。
Copyright (c) 2022 Yuuki Takano <ytakanoster@gmail.com>