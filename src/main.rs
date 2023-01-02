use std::{
    io::{self, Write},
    str::FromStr,
    env,
    path::Path,
    fs,
};
use colored::Colorize;

struct Command {
    keyword: String,
    arguments: Vec<String>,
}


enum BuiltinCommand {
    Echo,
    History,
    Cd,
    Pwd,
    Ls,
    Clear,
}

impl FromStr for BuiltinCommand {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "echo" => Ok(BuiltinCommand::Echo),
            "history" => Ok(BuiltinCommand::History),
            "cd" => Ok(BuiltinCommand::Cd),
            "pwd" => Ok(BuiltinCommand::Pwd),
            "ls" => Ok(BuiltinCommand::Ls),
            "clear" => Ok(BuiltinCommand::Clear),
            _ => Err(()),
        }
    }
}

fn main() {
    let prompt_char = '@';
    loop {
        print!("{} ", prompt_char);
        io::stdout().flush().unwrap();
        let mut command = String::new();
        io::stdin().read_line(&mut command).expect("Failed to read in command");
        
        let command = tokenize_command(command);
        process_command(command);
    }
}

fn tokenize_command(command: String) -> Command {
    if command == "\n" {
        return Command { keyword: String::new(), arguments: Vec::<String>::new(), };
    }
    let mut tokens: Vec<String> = command.split_whitespace().map(|s| s.to_string()).collect();
    
    Command { 
        keyword: tokens.remove(0), 
        arguments: tokens,
    }
}

fn process_command(command: Command) -> i32 {
    match BuiltinCommand::from_str(&command.keyword) {
        Ok(BuiltinCommand::Echo) => builtin_echo(&command.arguments),
        Ok(BuiltinCommand::History) => builtin_history(&command.arguments),
        Ok(BuiltinCommand::Cd) => builtin_cd(&command.arguments),
        Ok(BuiltinCommand::Pwd) => builtin_pwd(&command.arguments),
        Ok(BuiltinCommand::Ls) => builtin_ls(&command.arguments),
        Ok(BuiltinCommand::Clear) => builtin_clear(&command.arguments),
        Err(()) => {
            println!("Command not found");
            1
        }
    }
}

fn builtin_echo(args: &Vec<String>) -> i32 {
    println!("{}", args.join(" "));
    0
}

fn builtin_history(_args: &Vec<String>) -> i32 {
    println!("history");
    0
}

fn builtin_pwd(_args: &Vec<String>) -> i32 {
    println!("{}", env::current_dir().unwrap().to_str().unwrap());
    0
}

fn builtin_cd(args: &Vec<String>) -> i32 {
    if args.len() > 1 {
        panic!("Too many arguments");
    }
    if args.len() == 0 {
        let path = Path::new("/home/gleb");
        env::set_current_dir(path).unwrap();
        return 0;
    }
    let path = Path::new(&args[0]);
    if !path.exists() {
        panic!("This folder doesn't exist");
    }
    env::set_current_dir(path).unwrap();
    0
}

fn builtin_ls(_args: &Vec<String>) -> i32 {
    let paths = fs::read_dir("./").unwrap();

    for path in paths {
        let file_type = if path.as_ref().unwrap().metadata().unwrap().is_dir() { "dir " } else { "file" };
        let mut filename = path.unwrap().file_name().into_string().unwrap();
        if file_type == "dir " {
            filename = filename.bright_blue().on_bright_white().to_string();
        }        
        println!("{file_type} - {}", filename); 
    }
    0
}

fn builtin_clear(_args: &Vec<String>) -> i32 {
    print!("{}[2J", 27 as char);
    0
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn empty_command() {
        tokenize_command(String::from("")).keyword;
    }

    #[test]
    fn only_keyword() {
        assert_eq!("shell", tokenize_command(String::from("shell")).keyword);
    }

    #[test]
    fn keyword_and_arguments() {
        let tokenized_command = tokenize_command(String::from("shell arg1 arg2"));
        assert_eq!("shell", tokenized_command.keyword);
        assert_eq!(vec!["arg1", "arg2"], tokenized_command.arguments);
    }

    #[test]
    fn quotes() {
        assert_eq!(3, tokenize_command(String::from("shell 'first second maybeThird'")).arguments.len());
    }

    #[test]
    fn echo_cmd() {
        assert_eq!(0, process_command(tokenize_command(String::from("echo test"))));
    }
}
