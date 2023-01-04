use colored::Colorize;
use std::{
    collections::HashMap,
    io::{self, Write},
    str::FromStr,
    env,
    path::{Path, PathBuf},
    fs,
};

mod username;


const SUCCESS_CODE: i32 = 0;
const ERROR_CODE  : i32 = 1;
const EXIT_CODE   : i32 = -1;
const CRLF        : &str = "\r\n";


fn main() {
    let clear_escape_sequence = "\x1b[2J\x1b[1;1H";
    print!("{}", clear_escape_sequence);
    let prompt_char = '🚀';

    let username = username::get_username();
    loop {
        let current_dir = env::current_dir().unwrap();
        print!("\x1b[94m{username}\x1b[0m{}:\x1b[94m{}\x1b[0m ", prompt_char, current_dir.to_str().unwrap());

        io::stdout().flush().unwrap();
        let mut command_input = String::new();
        io::stdin().read_line(&mut command_input).expect("Failed to read in command");
        if command_input == CRLF {
            continue;
        }
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
    Cp,
    Rm,
    Mv,
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
            "cp" => Ok(BuiltinCommand::Cp),
            "rm" => Ok(BuiltinCommand::Rm),
            "mv" => Ok(BuiltinCommand::Mv),
            _ => Err(()),
        }
    }
}

fn tokenize_command(command: String) -> Command {
    if command == "\r\n" {
        return Command { keyword: String::new(), arguments: Vec::<String>::new(), };
    }
    let mut tokens: Vec<String> = command.split_whitespace().map(|s| s.to_string()).collect();
   
    Command { 
        keyword: tokens.remove(0), 
        arguments: tokens,
    }
}

fn process_command(mut command: Command) -> i32 {
    match BuiltinCommand::from_str(&command.keyword) {
        Ok(BuiltinCommand::Echo) => builtin_echo(&command.arguments),
        Ok(BuiltinCommand::History) => builtin_history(&command.arguments),
        Ok(BuiltinCommand::Cd) => builtin_cd(&command.arguments),
        Ok(BuiltinCommand::Pwd) => builtin_pwd(&command.arguments),
        Ok(BuiltinCommand::Ls) => builtin_ls(&command.arguments),
        Ok(BuiltinCommand::Clear) => builtin_clear(&command.arguments),
        Ok(BuiltinCommand::Cp) => builtin_cp(&command.arguments),
        Ok(BuiltinCommand::Rm) => builtin_rm(&command.arguments),
        Ok(BuiltinCommand::Mv) => builtin_mv(&command.arguments),
        Ok(BuiltinCommand::Exit) => EXIT_CODE,
        Err(()) => {
            let args = command.arguments.clone();
            if !command.keyword.contains(&String::from(".exe")) {
                command.keyword.push_str(".exe");
            }
            match find_executable(command) {
                Ok(path) => {
                    let mut process = std::process::Command::new(path);
                    process.args(args);
                    if let Ok(mut child) = process.spawn() {
                        child.wait().expect("Command wasn't running");
                        SUCCESS_CODE
                    } else {
                        println!("Command didn't start");
                        ERROR_CODE
                    }
                }
                Err(_) => {
                    println!("Command not found");
                    ERROR_CODE
                }
            }
        }
    }
}

fn find_executable(command: Command) -> Result<PathBuf, std::io::Error> {
    fn search(keyword: &str, dir: &Path) -> Result<(), std::io::Error> {
        for entry in fs::read_dir(dir)? {
            if let Ok(entry) = entry {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() || metadata.is_symlink() {
                        if let Some(filename) = entry.path().file_name() {
                            if filename == keyword {
                                if metadata.is_symlink() {
                                    println!("It's a symbolic link");
                                    return Err(std::io::ErrorKind::InvalidData.into());
                                }
                                return Ok(());
                            }
                        }
                    }
                }
            }
        }
        Err(std::io::ErrorKind::NotFound.into())
    }

    if let Ok(mut dir) = env::current_dir() {
        if let Ok(()) = search(&command.keyword, &dir) {
            let exec_name = command.keyword;
            dir.push(exec_name);
            return Ok(dir);
        }
    }

    let vars: HashMap<String, String> = env::vars().into_iter().collect();
    let values: &Vec<&str> = &vars["Path"].split(";").collect();

    for entry in values {
        if let Ok(()) = search(&command.keyword, Path::new(entry)) {
            let mut path = PathBuf::from(entry);
            let exec_name = command.keyword;
            path.push(exec_name);
            return Ok(path);
        }
    }
    return Err(std::io::ErrorKind::NotFound.into());
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
        let path = Path::new("C:\\Users\\Глеб");
        env::set_current_dir(path).unwrap();
        return ERROR_CODE;
    }
    let path;
    if args[0] == "~" {
        path = Path::new("C:\\Users\\Глеб"); 
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

fn builtin_rm(args: &Vec<String>) -> i32 {
    if args.contains(&String::from("-r")) {
        let args: Vec<&String> = args.iter().filter(|&arg| arg != "-r").collect();
        for arg in args {
            let arg_path = Path::new(arg);
            if arg_path.is_file() {
                match fs::remove_file(arg_path) {
                    Ok(()) => (),
                    Err(e) => {
                        println!("{}", e);
                        return ERROR_CODE;
                    },
                }
            } else if arg_path.is_dir() {
                match fs::remove_dir(arg_path) {
                    Ok(()) => (),
                    Err(e) => {
                        println!("{}", e);
                        return ERROR_CODE;
                    },
                }
            }
        }
        return SUCCESS_CODE;
    }

    for arg in args {
        let arg_path = Path::new(arg);
        if arg_path.is_file() {
            match fs::remove_file(arg_path) {
                Ok(()) => (),
                Err(e) => {
                    println!("{}", e);
                    return ERROR_CODE;
                },
            }
        } else if arg_path.is_dir() {
            println!("Cannot remove directory - {}", arg);
        }
    }

    SUCCESS_CODE
}

fn builtin_cp(args: &Vec<String>) -> i32 {
    if args.len() >= 2 {
        let path_to = Path::new(&args[args.len() - 1]);
        if path_to.is_dir() {
            for i in 0..args.len() - 1 {
                let file_from = Path::new(&args[i]);
                let mut new_path_to = path_to.to_path_buf();
                new_path_to.push(&args[i]);
                match fs::copy(file_from, new_path_to) {
                    Ok(_) => (),
                    Err(_) => {
                        println!("Error occurred during copying");
                        return ERROR_CODE;
                    }
                }
            }
            return SUCCESS_CODE;
        } else {
            println!("Erroc occurred - {} is not a dir", args[args.len() - 1]);
            return ERROR_CODE;
        }
    } else if args.len() < 2 {
        println!("Wrong number and types of arguments");
        return ERROR_CODE;
    }
    
    let file_from = Path::new(&args[0]);
    let file_to = Path::new(&args[1]);
    match fs::copy(file_from, file_to) {
        Ok(_) => {
            SUCCESS_CODE
        },
        Err(_) => {
            println!("Error occurred during copying");
            ERROR_CODE
        }
    }
}

fn builtin_mv(args: &Vec<String>) -> i32 {
    if args.len() >= 2 {
        let path_to = Path::new(&args[args.len() - 1]);
        if path_to.is_dir() {
            //let mut args = args;
            if builtin_cp(args) == ERROR_CODE {
                return ERROR_CODE;
            }
            if builtin_rm(&args[..args.len() - 1].to_vec()) == ERROR_CODE {
                return ERROR_CODE;
            }
        } else {
            println!("Erroc occurred - {} is not a dir", args[args.len() - 1]);
            return ERROR_CODE;
        }
        return SUCCESS_CODE;
    } else if args.len() < 2 {
        println!("Wrong number and types of arguments");
        return ERROR_CODE;
    }

    let file_from = Path::new(&args[0]);
    let file_to = Path::new(&args[args.len() - 1]);
    
    match fs::rename(file_from, file_to) {
        Ok(_) => {
            SUCCESS_CODE
        },
        Err(e) => {
            println!("Error occurred during moving _ {e}");
            ERROR_CODE
        }
    }
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
