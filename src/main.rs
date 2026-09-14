use std::io::{Write, stdout, stdin};
use std::fs::File;
use std::fs;
use std::path::Path;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Local};
use std::fmt;
use app_dirs2::*;

use rmp_serde::{Deserializer, Serializer};

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Todo {
    item_id: usize,
    date_time: DateTime<Local>,
    item_name: String,
    item_content: String,
}

const APP_INFO: AppInfo = AppInfo{
    name: "Rust Todo",
    author: "Eloise Nash"
};

impl fmt::Display for Todo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "id: {}\nitem: {}\ncreated: {}", self.item_id, self.item_name, self.date_time.format("%d-%m-%Y"))
    }
}

#[derive(Serialize, Deserialize)]
struct TodoList {
    list: Vec<Todo>,
    next_id: usize,
}

impl TodoList {
    fn add_item(&mut self, input: String) -> Option<TodoList> {
        let mut early_ret: bool = false;
        let mut words = input.split(" ");
        words.next();
        let name = match words.next() {
            Some(v) => String::from(v),
            None => {
                display_useage();
                early_ret = true; // Items with no name aren't allowed
                String::new()
            }
        };
        let details = words.collect::<Vec<&str>>().join(" "); // Items without details are allowed
        if !early_ret {
            self.list.push(Todo {item_id: self.next_id, date_time: Local::now(), item_name: name, item_content: details});
            self.next_id += 1;

            Some(TodoList {list: self.list.to_vec(), next_id: self.next_id})
        } else {
            None
        }
    }

    fn list_items(&self) -> TodoList {
        let mut iter = self.list.iter().peekable();
        while iter.peek().is_some() {
            let item = iter.next().unwrap();
            let name = &item.item_name;
            let id = &item.item_id;
            println!("{}: {}", id, name);
        }
        TodoList {list: self.list.to_vec(), next_id: self.next_id}
    }

    fn remove_items(&mut self, input: String) -> TodoList {
        let mut words = input.split(" ");
        words.next();
        let mut ids: Vec<usize> = Vec::new();
        for item in words {
            ids.push(item.trim().parse().unwrap());
        }

        ids.sort_by(|a, b| b.cmp(a));

        for id in ids {
            self.list.remove(id - 1);
            self.next_id -= 1;
        }

        let mut new_list: Vec<Todo> = Vec::new();
        
        let mut new_id = 1;

        for item in self.list.to_vec() {
            let mut update_item = item;
            update_item.item_id = new_id;
            new_id += 1;
            new_list.push(update_item);
        }

        TodoList {list: new_list, next_id: self.next_id}
    }

    fn write_list(&self, path: &Path) {
        let display = path.display();


        let mut file = match File::create(path) {
            Err(e) => panic!("Couldn't create {}: {}", display, e),
            Ok(file) => file,
        };

        let mut to_write = Vec::new();
        self.serialize(&mut Serializer::new(&mut to_write)).unwrap();

        match file.write_all(&to_write) {
            Err(e) => panic!("Couldn't write to {}: {}", display, e),
            Ok(_) => println!("Successfully wrote todo list to {}", display),
        }

    }

    fn display_info(&self, input: String) {
        let mut words = input.split(" ");
        words.next();
        let id =  words.next().unwrap().parse::<usize>().unwrap();

        let item = &self.list[id - 1];

        println!("id: {}", item.item_id);
        println!("item name: {}", item.item_name);
        println!("created on: {}", item.date_time);
        println!("details: {}", item.item_content);
    }
}

fn display_useage() {
    println!("simple todo list");
    println!("Useage....");
    println!("list -> displays the current todo list");
    println!("info -> displays further information about the specified item, if extra detail was given");
    println!("add <item> -> adds the item written to the list");
    println!("remove <id> | <id>..<id> -> removes the specified item(s) from the list");
    println!("save -> saves the current list to disk");
    println!("quit -> saves the current list to disk and then quits the application");
}

fn parse_input(input: String, mut list: TodoList, path: &Path) -> TodoList {
    let item_match: &str;
    if input.contains(" ") {
        let mut inputs = input.split(" ");
        item_match = inputs.next().unwrap().trim();
        
    } else {
        item_match = input.trim();
    }

    let ret = match item_match {
        //list should display all items in date order.
        "list" => list.list_items(),
        "add" => match list.add_item(input.trim().to_string()) {
            Some(v) => v,
            None => list
        },
        "remove" => list.remove_items(input.trim().to_string()),
        "save" => {list.write_list(path); list},
        "quit" => {list.write_list(path); list},
        "info" => {list.display_info(input.trim().to_string()); list},
        _ => {display_useage();list},
    };

    ret
    
}

fn init(path: &Path) -> TodoList {
    let display = path.display();

    println!("{}", display);
        
    if path.exists() {
        let mut _file = match File::open(path) {
            Err(e) => panic!("couldn't open {}: {}", display, e),
            Ok(file) => file,
        };

        let deser: TodoList = match &fs::read(path) {
            Err(e) => panic!("couldn't read {}: {}", display, e),
            Ok(buf) => if buf.is_empty() {
                let _file = match File::create(path) {
                    Err(e) => panic!("couldn't create {}: {}", display, e),
                    Ok(file) => file,
                };
                TodoList {list: Vec::new(), next_id: 1}
            } else {
                let mut de = Deserializer::new(&buf[..]);
                Deserialize::deserialize(&mut de).unwrap()
            },
        };
        println!("Current list:");
        deser.list_items();
        deser
    } else {
        let _file = match File::create(path) {
            Err(e) => panic!("couldn't create {}: {}", display, e),
            Ok(file) => file,
        };
        let ret: TodoList = TodoList { list: Vec::new(), next_id: 1};

        ret
    }

    
}


fn main() {
    let path = match app_root(AppDataType::UserData, &APP_INFO) {
        Ok(v) => v,
        Err(e) => panic!("Could not create/locate user data dir.\n({e})")
    };
    let path = path.join(Path::new("list.mpk"));
    let mut list: TodoList = init(&path);
    loop {
        print!("todo> ");
        stdout().flush().unwrap();
        let mut input: String = String::new();
        match stdin().read_line(&mut input) {
            Ok(_n) => list = parse_input(input.clone(), list, &path),
            Err(e) => println!("error: {}", e),
        }
        if input.trim() == "quit" {
            break;
        }
    }
}
