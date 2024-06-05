//用于失败的时候打印红色的错误信息
pub fn connect_failed() {
    bunt::eprintln!("{$red}CONNECTION REFUSED{/$}");
}

//用于成功的时候打印成功信息
pub fn log(request: &httparse::Request, response: &httparse::Response) {
    let out = match response.code {
        Some(code @ 200..=299) => format!("\x1b[32m{}\x1b[0m", code),
        Some(code) => format!("\x1b[31m{}\x1b[0m", code),
        _ => "\x1b[31m???\x1b[0m".to_string(),
    };

    let method = request.method.unwrap_or("????");
    let path = request.path.unwrap_or("");

    eprint!("{}", out);
    bunt::eprintln!("\t\t{[yellow]}\t{[blue]}", method.to_uppercase(), path);
}
