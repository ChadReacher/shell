use colored::Colorize;
use std::{
    io::{self, Write},
    str::FromStr,
    env,
    path::Path,
    fs,
};


const SUCCESS_CODE: i32 = 0;
const ERROR_CODE  : i32 = 1;
const EXIT_CODE   : i32 = -1;


fn main() {
    let clear_escape_sequence = "\x1b[2J\x1b[1;1H";
    print!("{}", clear_escape_sequence);
    let prompt_seq = "@";
    loop {
        let current_dir = env::current_dir().unwrap();
        print!("\x1b[92m{}:\x1b[0m\x1b[94m{}\x1b[0m ", prompt_seq, current_dir.to_str().unwrap());

        io::stdout().flush().unwrap();
        let mut command_input = String::new();
        io::stdin().read_line(&mut command_input).expect("Failed to read in command");
        let command = tokenize_command(command_input);
        let return_code = process_command(command);
        if return_code == EXIT_CODE {
            break;
        }
    }
}

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
    Exit,
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
            "exit" => Ok(BuiltinCommand::Exit),
            _ => Err(()),
        }
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
        Ok(BuiltinCommand::Exit) => EXIT_CODE,
        Err(()) => {
            println!("Command not found");
            ERROR_CODE
        }
    }
}

fn builtin_echo(args: &Vec<String>) -> i32 {
    println!("{}", args.join(" "));
    SUCCESS_CODE
}

fn builtin_history(_args: &Vec<String>) -> i32 {
    println!("history");
    SUCCESS_CODE
}

fn builtin_pwd(_args: &Vec<String>) -> i32 {
    println!("{}", env::current_dir().unwrap().to_str().unwrap());
    SUCCESS_CODE
}

fn builtin_cd(args: &Vec<String>) -> i32 {
    if args.len() > 1 {
        println!("Too many arguments");
        return ERROR_CODE;
    }
    if args.len() == 0 {
        let path = Path::new("/home/gleb");
        env::set_current_dir(path).unwrap();
        return ERROR_CODE;
    }
    let path;
    if args[0] == "~" {
        path = Path::new("/home/gleb"); 
    } else {
        path = Path::new(&args[0]);
    }
    if !path.exists() {
        println!("This folder doesn't exist");
        return ERROR_CODE;
    }
    env::set_current_dir(path).unwrap();
    SUCCESS_CODE
}

fn builtin_ls(_args: &Vec<String>) -> i32 {
    let paths = fs::read_dir("./").unwrap();

    for path in paths {
        let file_type = if path.as_ref().unwrap().metadata().unwrap().is_dir() { "dir " } else { "file" };
        let mut filename = path.unwrap().file_name().into_string().unwrap();
        if file_type == "dir " {
            filename = filename.bright_blue().on_bright_white().to_string();
        }        
        println!("   {file_type} - {}", filename); 
    }
    SUCCESS_CODE
}

fn builtin_clear(_args: &Vec<String>) -> i32 {
    print!("\x1b[2J\x1b[1;1H");
    SUCCESS_CODE
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
