use colored::Colorize;
use chrono::prelude::*;
use filetime::{FileTime, set_file_atime, set_file_mtime};
use std::{
    time::{Duration, UNIX_EPOCH},
    collections::HashMap,
    io::{self, Write},
    str::FromStr,
    env,
    path::{Path, PathBuf},
    fs::{self, File, Metadata, Permissions, DirEntry},
};

mod username;

const SUCCESS_CODE: i32 = 0;
const ERROR_CODE  : i32 = 1;
const EXIT_CODE   : i32 = -1;
const CRLF        : &str = "\r\n";
const HELP_FILE_INFO: &str = "help.txt";


fn main() {
    let mut commands_vector: Vec<String> = Vec::new();
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

        let mut command_input_clone = command_input.clone();
        command_input_clone.pop();
        command_input_clone.pop();
        commands_vector.push(command_input_clone);

        let command = tokenize_command(command_input);
        let return_code = process_command(command, &commands_vector);
        if return_code == EXIT_CODE {
            break;
        }
    }
}

#[derive(Clone)]
struct Command {
    keyword: String,
    arguments: Vec<String>,
    options: Vec<String>,
    plain_command_input: String,
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
    Touch,
    Mkdir,
    Cat,
    Help,
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
            "touch" => Ok(BuiltinCommand::Touch),
            "mkdir" => Ok(BuiltinCommand::Mkdir),
            "cat" => Ok(BuiltinCommand::Cat),
            "help" => Ok(BuiltinCommand::Help),
            _ => Err(()),
        }
    }
}

fn tokenize_command(command: String) -> Command {
    let mut tokens: Vec<String> = command.split_whitespace().map(|s| s.to_string()).collect();
    let keyword = tokens.remove(0);
    let plain_command_input = tokens.join(" ");
    let arguments: Vec<String> = tokens.clone().into_iter().filter(|arg| !arg.starts_with("-")).collect();
    let options: Vec<String> = tokens.into_iter().filter(|arg| arg.starts_with("-")).collect();
   
    Command { 
        keyword,
        arguments,
        options,
        plain_command_input,
    }
}

fn process_command(mut command: Command, commands_vector: &Vec<String>) -> i32 {
    match BuiltinCommand::from_str(&command.keyword) {
        Ok(BuiltinCommand::Echo) => builtin_echo(command),
        Ok(BuiltinCommand::History) => builtin_history(command, commands_vector),
        Ok(BuiltinCommand::Cd) => builtin_cd(command),
        Ok(BuiltinCommand::Pwd) => builtin_pwd(command),
        Ok(BuiltinCommand::Ls) => builtin_ls(command),
        Ok(BuiltinCommand::Clear) => builtin_clear(command),
        Ok(BuiltinCommand::Cp) => builtin_cp(command),
        Ok(BuiltinCommand::Rm) => builtin_rm(command),
        Ok(BuiltinCommand::Mv) => builtin_mv(command),
        Ok(BuiltinCommand::Touch) => builtin_touch(command),
        Ok(BuiltinCommand::Mkdir) => builtin_mkdir(command),
        Ok(BuiltinCommand::Cat) => builtin_cat(command),
        Ok(BuiltinCommand::Help) => builtin_help(command),
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

fn builtin_echo(command: Command) -> i32 {
    println!("{}", command.plain_command_input);
    SUCCESS_CODE
}

fn builtin_history(command: Command, commands_vector: &Vec<String>) -> i32 {
    if command.options.contains(&String::from("--help")) {
        println!("history:\n\
                  Display a history of entered commands.\n\
                  The list of commands contains their index number from 1.");
        return SUCCESS_CODE;
    }
    for i in 0..commands_vector.len() {
        println!("{} {}", i + 1, commands_vector[i]);
    }
    SUCCESS_CODE
}

fn builtin_pwd(command: Command) -> i32 {
    if command.options.contains(&String::from("--help")) {
        println!("pwd:\n\
                     Display you currrent/working directory");
        return SUCCESS_CODE;
    }
    println!("{}", env::current_dir().unwrap().to_str().unwrap());
    SUCCESS_CODE
}

fn builtin_cd(command: Command) -> i32 {
    if command.arguments.len() + command.options.len() > 1 {
        println!("Too many arguments");
        return ERROR_CODE;
    }

    if command.options.contains(&String::from("--help")) {
        println!("cd:\n\
                  Change the current directory to user specified one.\n\
                  If destination directory is not specified, the default will be 'HOMEDRIVE + HOMEPATH' shell variable");
        return SUCCESS_CODE;
    }

    let env_vars: HashMap<String, String> = env::vars().collect();
    let system_drive = &env_vars["HOMEDRIVE"];
    let full_home_path = system_drive.to_owned() + &env_vars["HOMEPATH"];
 
    let path = if command.arguments.len() == 0 || command.arguments[0] == "~" { 
        Path::new(&full_home_path) 
    } else { 
        Path::new(&command.arguments[0])
    };
    if !path.exists() {
        println!("This folder does not exist");
        return ERROR_CODE;
    }
    match env::set_current_dir(path) {
        Ok(_) => SUCCESS_CODE,
        Err(err) => {
            println!("Error - {err}");
            ERROR_CODE
        },
    }
}

struct FileDisplayInfo {
    filename: String,
    last_modified_time: DateTime<Local>,
    filesize: u64,
    file_type: Metadata,
    permissions: Permissions,
}

fn builtin_ls(command: Command) -> i32 { 
    if command.options.contains(&String::from("--help")) {
        println!("ls:\n\
                  Print list of files in specified directory(default directory is current directory).\n\
                  -l        display folder contens as a list with size, last modified time, permissions");
        return SUCCESS_CODE;
    }
    let mut dirs_list = command.arguments.clone();

    let current_dir = String::from("./");

    if dirs_list.len() == 0 {
        dirs_list.push(current_dir);
    }

    let mut dir_files_map: HashMap<String, Vec<FileDisplayInfo>> = HashMap::new();

    for dir in dirs_list {

        let paths = fs::read_dir(&dir);
        if let Err(err) = paths {
            println!("Error: {err}");
            continue;
        }
        let paths: Vec<Result<DirEntry, std::io::Error>> = fs::read_dir(&dir).unwrap().collect();

        if paths.len() == 0 { 
            return ERROR_CODE;
        }
        let mut files = Vec::new();

        let mut longest_filesize: String = paths[0].as_ref().unwrap().metadata().unwrap().len().to_string();

        for path in paths {
            let file_type = path.as_ref().unwrap().metadata().unwrap();
            let filename = path.as_ref().unwrap().file_name().into_string().unwrap();
            let filesize = path.as_ref().unwrap().metadata().unwrap().len();

            let last_modified_time = path.as_ref().unwrap().metadata().unwrap().modified().unwrap();
            let seconds = last_modified_time.duration_since(UNIX_EPOCH).unwrap().as_secs();
            let datetime = DateTime::<Local>::from(UNIX_EPOCH + Duration::from_secs(seconds));

            let permissions = path.as_ref().unwrap().metadata().unwrap().permissions();

            let file_display_info = FileDisplayInfo {
                filename,
                last_modified_time: datetime,
                filesize,
                file_type,
                permissions,
            };

            files.push(file_display_info);

            if filesize.to_string().len() > longest_filesize.len() {
                longest_filesize = filesize.to_string();
            }
        }

        dir_files_map.insert(dir, files);
    }

    if command.options.contains(&String::from("-l")) {
        for (dir_name, files) in dir_files_map {
            if command.arguments.len() > 1 {
                println!("{dir_name}:");
            }
            for file in &files {
                let longest_filesize = files
                    .iter()
                    .max_by(|x, y| 
                            x.filesize.to_string().len()
                            .cmp(&y.filesize.to_string().len()))
                    .unwrap()
                    .filesize
                    .to_string();
                let mut file_info_str = String::new();

                let permission = if file.permissions.readonly() { "rd " } else { "wr " };
                file_info_str.push_str(permission);
                let file_type = if file.file_type.is_dir() { "dir  " } else { "file " };
                file_info_str.push_str(file_type);
                file_info_str.push_str(&file.filesize.to_string());

                let spaces = " ".repeat((longest_filesize.len() + 1) - file.filesize.to_string().len());
                file_info_str.push_str(&spaces);

                let timestamp_str = file.last_modified_time.format("%d-%m-%Y %H:%M:%S").to_string();
                file_info_str.push_str(&timestamp_str);
                file_info_str.push(' ');

                if file_type == "dir  " {
                    let filename = file.filename.blue().on_white().to_string();
                    file_info_str.push_str(&filename);
                } else {
                    file_info_str.push_str(&file.filename);
                }
                println!("{}", file_info_str);
            }                  
            println!();
        }
    } else {
        for (dir_name, files) in dir_files_map {
            if command.arguments.len() > 1 {
                println!("{dir_name}:");
            }
            for file in files {
                let mut file_info_str = String::new();

                let file_type = if file.file_type.is_dir() { "dir  " } else { "file " };
                file_info_str.push_str(file_type);

                if file_type == "dir  " {
                    let filename = file.filename.blue().on_white().to_string();
                    file_info_str.push_str(&filename);
                } else {
                    file_info_str.push_str(&file.filename);
                }
                println!("{}", file_info_str);
            }
            println!();
        }
    }
    SUCCESS_CODE
}

fn builtin_clear(command: Command) -> i32 {
    if command.arguments.contains(&String::from("--help")) {
        println!("clear:\n\
                  Clears the terminal screen.");
        return SUCCESS_CODE;
    }
    print!("\x1b[2J\x1b[1;1H");
    SUCCESS_CODE
}

fn builtin_rm(command: Command) -> i32 {
    if command.arguments.contains(&String::from("--help")) {
        println!("rm:\n\
                 Removes files and directories in current directory.\n\
                 -r         Remove recursively(for removing directories)");
        return SUCCESS_CODE;
    }
    for arg in command.arguments {
        let arg_path = Path::new(&arg);
        if arg_path.is_file() {
            match fs::remove_file(arg_path) {
                Ok(()) => (),
                Err(e) => {
                    println!("Error occurred when removing file - {}", e);
                    return ERROR_CODE;
                },
            }
        } else if arg_path.is_dir() {
            if command.options.contains(&String::from("-r")) {
                match fs::remove_dir_all(arg_path) {
                    Ok(()) => (),
                    Err(e) => {
                        println!("Error occurred when removing directory - {}", e);
                        return ERROR_CODE;
                    },
                }
            } else {
                println!("Cannot remove directory - {}", arg);
            }
        }
    }

    SUCCESS_CODE
}

fn builtin_cp(command: Command) -> i32 {
    if command.arguments.contains(&String::from("--help")) {
        println!("cp:\n\
                  Copy file or files into another file or given folder");
        return SUCCESS_CODE;
    }
    if command.arguments.len() >= 2 {
        let last_item = command.arguments.last().unwrap();
        let path_to = Path::new(&last_item);
        if !path_to.exists() {
            println!("Destination file doesn't exist");
            return ERROR_CODE;
        }
        if path_to.is_dir() {
            for i in 0..command.arguments.len() - 1 {
                let file_from = Path::new(&command.arguments[i]);
                let mut new_path_to = path_to.to_path_buf();
                new_path_to.push(&command.arguments[i]);

                match fs::copy(file_from, new_path_to) {
                    Ok(_) => (),
                    Err(err) => {
                        println!("Error occurred during copying - {err}");
                    }
                }
            }
            return SUCCESS_CODE;
        } else if command.arguments.len() == 2 {
            let file_from = Path::new(&command.arguments[0]);
            let file_to = Path::new(&command.arguments[1]);
            match fs::copy(file_from, file_to) {
                Ok(_) => {
                    SUCCESS_CODE
                },
                Err(err) => {
                    println!("Error occurred during copying - {err}");
                    ERROR_CODE
                }
            }
        } else {
            println!("Erroc occurred - {} is not a dir", command.arguments.last().unwrap());
            ERROR_CODE
        }
    } else {
        println!("Wrong number and types of arguments");
        ERROR_CODE
    } 
}

fn builtin_mv(command: Command) -> i32 {
    if command.arguments.contains(&String::from("--help")) {
        println!("mv:\n\
                  Moves one file to another place.\n\
                  If the files already exists, changes his name");
        return SUCCESS_CODE;
    }
    if command.arguments.len() > 2 {
        let last_item = command.arguments.last().unwrap();
        let path_to = Path::new(&last_item);
        if path_to.is_dir() {
            if builtin_cp(command.clone()) == ERROR_CODE {
                return ERROR_CODE;
            }
            let cmd = Command { keyword: command.keyword, arguments: command.arguments[..command.arguments.len() - 1].to_vec(), 
                options: command.options, plain_command_input: command.plain_command_input.clone() };
            if builtin_rm(cmd) == ERROR_CODE {
                return ERROR_CODE;
            }
        } else {
            println!("Erroc occurred - {} is not a dir", command.arguments.last().unwrap());
            return ERROR_CODE;
        }
        return SUCCESS_CODE;
    } else if command.arguments.len() < 2 {
        println!("Wrong number and types of arguments");
        return ERROR_CODE;
    }

    let file_from = Path::new(&command.arguments[0]);
    let last_item = command.arguments.last().unwrap();
    let file_to = Path::new(&last_item);
    if file_from.is_dir() && file_to.is_file() {
        println!("Cannot rename non-directory {} with directory {}", last_item, command.arguments[0]);
        return ERROR_CODE;
    }
    
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

fn builtin_touch(command: Command) -> i32 {
    if command.arguments.contains(&String::from("--help")) {
        println!("touch:\n\
                  Update modified and accessed time on the file or directory to the current time.\n\
                  If the file does not exist, it will be created.\n\
                  -a            Change only accessed time.\n\
                  -m            Change only modified time.");
        return SUCCESS_CODE;
    }
    if command.arguments.len() == 0 {
        println!("Not enough arguments");
        return ERROR_CODE;
    }

    let mut set_accces_time = true;
    let mut set_mod_time = true;
    if command.options.contains(&String::from("-a")) && !command.options.contains(&String::from("-m")) {
        set_mod_time = false;
    } else if !command.options.contains(&String::from("-a")) && command.options.contains(&String::from("-m")) {
        set_accces_time = false;
    }

    for arg in command.arguments {
        let arg_path = Path::new(&arg);
        if !arg_path.exists() {
            if let Err(err) = File::create(arg_path) {
                println!("Couldn't create a new file - {err}");
            }
        } else {
            if set_accces_time {
                if let Err(_) = set_file_atime(arg_path, FileTime::now()) {
                    println!("Error while setting access time");
                }
            }
            if set_mod_time {
                if let Err(_) = set_file_mtime(arg_path, FileTime::now()) {
                    println!("Error while setting modification time");
                }
            }
        }
    }
    SUCCESS_CODE
}

fn builtin_mkdir(command: Command) -> i32 {
    if command.arguments.contains(&String::from("--help")) {
        println!("mkdir:\n\
                  Creates a folder, if it does not exist");
        return SUCCESS_CODE;
    }
    if command.arguments.len() == 0 {
        println!("Not enough arguments");
        return ERROR_CODE;
    }

    for arg in command.arguments {
        let arg_path = Path::new(&arg);
        if arg_path.exists() {
            println!("Cannot create directory: file {} exists", arg);
            continue;
        }
        if let Err(e) = fs::create_dir(arg_path) {
            println!("Could not create a directory - {}", e);
        }
    }

    SUCCESS_CODE
}

fn builtin_cat(mut command: Command) -> i32 {
    if command.arguments.contains(&String::from("--help")) {
        println!("cat:\n\
                  Concatenate files into another file or prints it to standard output.\n\
                  The result can be redirected via '>' to file.");
        return SUCCESS_CODE;
    }
    if command.arguments.len() == 0 {
        println!("Not enough arguments");
        return ERROR_CODE;
    }

     
    let redirection_sign = String::from(">");
    let mut file_string = String::new();

    let mut dest_file = String::new();
    let mut contains_redirection = false;
    if command.arguments.contains(&redirection_sign) {
        if command.arguments[command.arguments.len() - 2] != redirection_sign {
            println!("Wrong position of arguments");
            return ERROR_CODE;
        } else {
            dest_file = command.arguments.pop().unwrap();
            command.arguments.pop();
            contains_redirection = true;
        }
    }

    for arg in &command.arguments { 
        let file_path = Path::new(&arg);
        if file_path.is_dir() {
            println!("Error occurred - {} is directory", arg);
            continue;
        }
        let file_contents = fs::read_to_string(file_path);
        match file_contents {
            Ok(file_contents) => file_string.push_str(&file_contents),
            Err(err) => {
                println!("Error occurred while reading file: {}", err);
            }
        }
    }
    //dbg!(&file_string);
    //file_string.pop();
    //file_string.pop();
    //dbg!(&file_string);

    if contains_redirection && !file_string.is_empty() {
        let file_path = Path::new(&dest_file);
        if file_path.is_dir() {
            println!("Cannot redirect output to directory - {}", dest_file);
            return ERROR_CODE;
        }
        let file = File::create(file_path);
        match file {
            Ok(mut file) => {
                if let Err(err) = file.write_all(file_string.as_bytes()) {
                    println!("Error occurred while redirecting output: {err}");
                    return ERROR_CODE;
                }
            },
            Err(err) => {
                println!("Error occurred when opening file: {err}");
                return ERROR_CODE;
            }
        }
    } else {
        println!("{file_string}");
    }


    SUCCESS_CODE
}

fn builtin_help(command: Command) -> i32 {
    if command.arguments.contains(&String::from("--help")) {
        println!("help:\n\
                  Display information aboult all available buitlin commands");
        return SUCCESS_CODE;
    }
    let help_info = fs::read_to_string(HELP_FILE_INFO);
    match help_info {
        Ok(help_info_content) => {
            println!("{}", help_info_content);
            SUCCESS_CODE
        }
        Err(err) => {
            println!("Could not print help info - {err}");
            ERROR_CODE
        }
    }
}


#[cfg(test)]
mod tokenizing_tests {
    use super::*;

    #[test]
    fn only_keyword() {
        let tokenized_command = tokenize_command(String::from("shell"));
        assert_eq!("shell", tokenized_command.keyword);
        assert_eq!(Vec::<String>::new(), tokenized_command.arguments);
    }

    #[test]
    fn keyword_and_one_argument() {
        let tokenized_command = tokenize_command(String::from("cat arg1 "));
        assert_eq!("cat", tokenized_command.keyword);
        assert_eq!(vec![String::from("arg1")], tokenized_command.arguments);
    }

    #[test]
    fn keyword_and_two_arguments() {
        let tokenized_command = tokenize_command(String::from("cat arg1 arg2"));
        assert_eq!("cat", tokenized_command.keyword);
        assert_eq!(vec![String::from("arg1"), String::from("arg2")], tokenized_command.arguments);
    }

    #[test]
    fn keyword_and_many_arguments() {
        let tokenized_command = tokenize_command(String::from("cat arg1 arg2 arg3 blabla sth"));
        assert_eq!("cat", tokenized_command.keyword);
        assert_eq!(vec![String::from("arg1"), String::from("arg2"), String::from("arg3"), String::from("blabla"), String::from("sth")], 
                   tokenized_command.arguments);
    }
}
