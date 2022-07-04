use serde::Deserialize;
use serde::de::value::CharDeserializer;
use std::char::ParseCharError;
use std::io::{prelude::*, BufReader};
use std::num::ParseIntError;
use std::string::ParseError;
use std::string::String;
use anyhow::{Context, Result, Error, anyhow};
use ctrlc;
use std::sync::mpsc::channel;
use crossbeam_channel::{bounded, tick, Receiver, select};
use std::time::Duration;
use std::io::{self};
use std::collections::HashMap;
use std::fs::File;
use serde_json;

#[derive(Deserialize, Debug)]
struct Character {
    name: String,
    display_char: String,
    def: u32,
    att: u32,
}

struct Coordinates {
    row: u32,
    col: u32
}

fn ctrl_channel() -> Result<Receiver<()>, ctrlc::Error>{
    let (sender, receiver) = bounded(100);
    ctrlc::set_handler(move || {
        let _ = sender.send(());
    })?;

    Ok(receiver)
}

// TODO: Label the board spaces

fn get_board(spaces_array: &[[&Character; 3]; 3]) -> Result<String, io::Error>{
    let mut board = String::new().to_owned();
    let mut i = 0;
    let board_space_height = 6;
    let board_height = (spaces_array.len() * board_space_height) + 1;
    let board_space_width = 10;
    let board_width = (spaces_array[0].len() * board_space_width) + 1;
    

    // iterate over height (rows)
    while i < board_height {
        if i % board_space_height == 0 || i == board_height - 1 {
            let mut j = 0;
            while j < board_width {
                board.push_str("#");
                j += 1;
            }
        } else {
            // iterate over width (columns)
            let mut j = 0;
            while j < board_width {
                if j % board_space_width == 0 || j == board_width - 1 {
                    board.push_str("#");
                } else if 
                    i - (i / &board_space_height) * &board_space_height == board_space_height / 2 && 
                    j - (j / &board_space_width) * &board_space_width == board_space_width / 2
                {
                    // Print actual object in center
                    board.push_str(&spaces_array[i / board_space_height][j / board_space_width].display_char);
                } else {
                    board.push_str(" ");
                }
                j += 1;
            }
        }
        board.push_str("\n");
        i += 1;
    }

    Ok(board)
}

fn parse_coordinates(space_name: &str) -> Result<Coordinates, Error> {
    let column_char = space_name[0 .. 1].parse::<char>()?;

    let column_ascii: u32 = column_char as u32;
    
    let row_index: u32 = space_name.trim()[1 .. ].parse::<u32>()? - 1;

    #[derive(Debug, Clone)]
    struct ColumnIndexError;

    let mut column_index = 0;
    // Convert column index num to array index num (i.e. A -> 65 -> 0)
    if column_ascii >= 65 && column_ascii <= 90 {
        column_index = column_ascii - 65;
    } else if column_ascii >= 97 && column_ascii <= 122 {
        column_index = column_ascii - 97;
    } else {
        println!("Column Index not in range!");
        return Err(anyhow!{"Column Index not in range!"});
    }

    println!("Column Index: {}", &column_index);
    println!("Row Index: {}", &row_index);

    Ok(Coordinates{row: row_index, col: column_index})
}

fn parse_commands(line: &str) -> Result<Vec<String>, Error> {
    let v: Vec<String> = line.split_whitespace().map(String::from).collect();

    // command structure should mimic chess i.e.
    // A1 to B1
    // B1 attack B2
    // etc.
    // syntax: <select_unit_coordinates> <action> <target_coordinates>

    if v.len() != 3 {
        return Err(anyhow!("Wrong number of commands!"));
    }

    Ok(v)

}

fn validate_move(
    coordinates: &Coordinates,
    spaces_array: &[[&Character; 3]; 3]
) -> Result<()>{
    if (coordinates.row as usize)  < spaces_array.len() {
        println!("Valid row");
    } else {
        // TODO: More descriptive error message for row/col
        return Err(anyhow!{"Row input is not valid."});
    }
    if (coordinates.col as usize) < spaces_array[0].len() {
        println!("Valid col");
    } else {
        return Err(anyhow!{"Column input is not valid."});
    }

    Ok(())
}

fn move_character<'a, 'b, 'c>(
    selected_name: &'c str, 
    space_name: &'c str, 
    spaces_array: &'b mut [[&'a Character; 3]; 3],
    empty_space: &'a Character
) -> Result<()> {    
    // parse space name
    let select_coordinates: Coordinates = parse_coordinates(selected_name)?;
    let selected_character = spaces_array[select_coordinates.row as usize][select_coordinates.col as usize];
    let move_to_coordinates = parse_coordinates(space_name)?;
    
    // TODO: Assert that move to coordinates is in range here
    // TODO: Add "move range" (or something) to character class and use manhattan distance from Unreal project
    // TODO: Highlight available spaces for move by using parentheses around character on space

    match validate_move(&move_to_coordinates, &spaces_array) {
        Ok(()) => println!("OK"),
        Err(e) => {
            println!("{}", e);
            return Err(e);
        }
    };
    
    println!("Moving Character to {}", space_name);
    spaces_array[move_to_coordinates.row as usize][move_to_coordinates.col as usize] = selected_character;
    spaces_array[select_coordinates.row as usize][select_coordinates.col as usize] = empty_space;

    Ok(())
}

fn main() -> Result<()> {
    let empty_space = Character {
        name: String::from("empty"),
        display_char: String::from(" "),
        att: 0,
        def: 0
    };
    let ctrl_c_events= ctrl_channel()?;
    let ticks = tick(Duration::from_secs(1));
    let mut spaces_array: [[&Character; 3]; 3] = [[&empty_space; 3]; 3];
    let mut spaces = HashMap::new();
    let soldier_path = "./src/character_classes/soldier.json";
    let soldier_file = File::open(soldier_path)?;
    let reader = BufReader::new(soldier_file);
    let soldier_data: Character = serde_json::from_reader(reader)?;

    spaces.insert(
        "A1".to_string(),
        &spaces_array[0][0]
    );

    spaces_array[0][0] = &soldier_data;

    loop {
        let mut line = String::new();
        let mut selected_coordinates:Coordinates;
        let mut selected_char: Character;
        std::io::stdin().read_line(&mut line).expect("Error");
        
        // Render board on enter or ctrl-c to quit
        select! {
            recv(ctrl_c_events) -> _ => {
                println!("Goodbye!");
                break;
            }
            recv(ticks) -> _ => {
                println!("{}", get_board(&spaces_array)?);
                // Do game logic here
                let commands = match parse_commands(&line) {
                    Ok(commands) => commands,
                    Err(_) => continue
                };
                
                // selected_coordinates = parse_coordinates(&line)?;
                move_character(&commands[0], &commands[2], &mut spaces_array, &empty_space);
            }
        }
    }

    Ok(())
}