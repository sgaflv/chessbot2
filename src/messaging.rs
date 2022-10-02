use log::info;

pub fn get_message() -> (String, String) {
    use std::io;

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    let input = input.trim().to_owned();

    info!("<== received: {}", input);

    if let Some(first) = input.find(' ') {
        let second = first + 1;

        let s1: String = input[..first].to_owned();
        let s2: String = input[second..].to_owned();

        (s1, s2)
    } else {
        return (input, "".to_owned());
    }
}

pub fn send_message(message: &str) {
    info!("==> sent: {}", message);
    println!("{}", message);
}
