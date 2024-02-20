use std::io::{Write, stdout, stdin};
use std::fs::File;
use std::fs;
use std::path::Path;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Local};
use std::fmt;

use rmp_serde::{Deserializer, Serializer};

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Todo {
    item_id: usize,
    date_time: DateTime<Local>,
    item_content: String,
}

impl fmt::Display for Todo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "id: {}\nitem: {}created: {}", self.item_id, self.item_content, self.date_time.format("%d-%m-%Y"))
    }
}

#[derive(Serialize, Deserialize)]
struct TodoList {
    list: Vec<Todo>,
    next_id: usize,
}

impl TodoList {
    fn add_item(&mut self, input: String) -> TodoList {
        let mut words = input.split(" ");
        words.next();
        let item = words.collect::<Vec<&str>>().join(" ");
        self.list.push(Todo {item_id: self.next_id, date_time: Local::now(), item_content: item});
        self.next_id += 1;

        TodoList {list: self.list.to_vec(), next_id: self.next_id}
    }

    fn list_items(&self) -> TodoList {
        let mut iter = self.list.iter().peekable();
        while iter.peek().is_some() {
            println!("{}", iter.next().unwrap());
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

    fn write_list(&self) {
        let path = Path::new("list.mpk");
        let display = path.display();


        let mut file = match File::create(&path) {
            Err(e) => panic!("couldn't create {}: {}", display, e),
            Ok(file) => file,
        };

        let mut to_write = Vec::new();
        self.serialize(&mut Serializer::new(&mut to_write)).unwrap();

        match file.write_all(&to_write) {
            Err(e) => panic!("couldn't write to {}: {}", display, e),
            Ok(_) => println!("successfully wrote todo list to {}", display),
        }

    }
}

fn display_useage() {
    println!("simple todo list");
    println!("Useage....");
    println!("list -> displays the current todo list");
    println!("add <item> -> adds the item written to the list");
    println!("remove <id> | <id>..<id> -> removes the specified item(s) from the list");
    println!("save -> saves the current list to disk");
    println!("quit -> saves the current list to disk and then quits the application");
}

fn parse_input(input: String, mut list: TodoList) -> TodoList {
    let item_match: &str;
    if input.contains(" ") {
        let mut inputs = input.split(" ");
        item_match = inputs.next().unwrap();
        
    } else {
        item_match = input.trim();
    }

    let ret = match item_match {
        //list should display all items in date order.
        "list" => list.list_items(),
        "add" => list.add_item(input),
        "remove" => list.remove_items(input),
        "save" => {list.write_list();list},
        "quit" => {list.write_list();list}
        _ => {display_useage();list},
    };

    ret
    
}

fn init() -> TodoList {
    let path = Path::new("list.mpk");
    let display = path.display();
        
    if path.exists() {
        let mut _file = match File::open(&path) {
            Err(e) => panic!("couldn't open {}: {}", display, e),
            Ok(file) => file,
        };

        let deser: TodoList = match &fs::read(path) {
            Err(e) => panic!("couldn't read {}: {}", display, e),
            Ok(buf) => if buf.is_empty() {
                let _file = match File::create(&path) {
                    Err(e) => panic!("couldn't create {}: {}", display, e),
                    Ok(file) => file,
                };
                TodoList {list: Vec::new(), next_id: 1}
            } else {
                let mut de = Deserializer::new(&buf[..]);
                Deserialize::deserialize(&mut de).unwrap()
            },
        };

        deser
    } else {
        let _file = match File::create(&path) {
            Err(e) => panic!("couldn't create {}: {}", display, e),
            Ok(file) => file,
        };
        let ret: TodoList = TodoList { list: Vec::new(), next_id: 1};

        ret
    }

    
}


fn main() {
    let mut list: TodoList = init();
    loop {
        print!("todo> ");
        stdout().flush().unwrap();
        let mut input: String = String::new();
        match stdin().read_line(&mut input) {
            Ok(_n) => list = parse_input(input.clone(), list),
            Err(e) => println!("error: {}", e),
        }
        if input.trim() == "quit" {
            break;
        }
    }
}
