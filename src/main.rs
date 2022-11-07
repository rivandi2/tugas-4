use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;
use structopt::StructOpt;
use thiserror::Error;

#[derive(Debug)]
struct Food {
    id: i64,
    name: String,
    stock: i64,
    price: i64,
}

struct Foods {
    inner: HashMap<i64, Food>,
}
impl Foods {
    fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    fn edit(&mut self, id: i64, name: &str, stock: i64, price: i64){
        self.inner.insert(
            id,
            Food {
                id,
                name: name.to_string(),
                stock,
                price,
            },
        );
    }

    fn add(&mut self, food: Food){
        self.inner.insert(food.id, food);
    }

    fn remove(&mut self, id: &i64) -> Option<Food> {
        self.inner.remove(&id)
    }

    fn into_vec(mut self) -> Vec<Food> {
        let mut foods: Vec<_> = self.inner.drain().map(|e| e.1).collect();
        foods.sort_by_key(|fd| fd.id);
        foods
    }

    fn next_id(&self) -> i64 {
        let mut ids: Vec<_> = self.inner.keys().collect();
        ids.sort();
        match ids.pop() {
            Some(id) => id+1,
            None => 1,
        }
    }

    fn search(&mut self, name:&str) -> Option<i64> {
        let foods: Vec<_> = self.inner.drain().map(|e| e.1).collect();
        for food in foods {
            if food.name.to_lowercase() == name.to_lowercase(){
               return Some(food.id);
            }
        }
        None
    }

    fn search_stock(&mut self, name_buy: &str, stock_buy: &i64) -> Result<i64, ParseError> {
        let foods: Vec<_> = self.inner.drain().map(|e| e.1).collect();
        for food in foods {
            if food.name == name_buy {
                if stock_buy > &food.stock{
                    return Err(ParseError::StockFood);
                } else {
                    return Ok(food.id);
                }
            }    
        }
        Err(ParseError::FoundFood) 
    }

    fn sort_foods(&mut self, id_del: i64) {
        let foods: Vec<_> = self.inner.drain().map(|e| e.1).collect();
        for food in foods {
            if food.id >= id_del + 1 {
                self.inner.insert(
                    food.id - 1,
                    Food {
                        id: food.id - 1,
                        name: food.name,
                        stock: food.stock,
                        price: food.price,
                    },
                );
            } else{
                self.inner.insert(
                    food.id,
                    Food {
                        id: food.id,
                        name: food.name,
                        stock: food.stock,
                        price: food.price,
                    },
                );
            }
        }
    }

    fn buy(&mut self, id: i64, name: &str, stock_buy: i64) {
        let foods: Vec<_> = self.inner.drain().map(|e| e.1).collect();
        for food in foods {
            if food.name.to_lowercase() == name.to_lowercase(){
                println!("{} {} porsi telah terjual, total transaksi adalah Rp {},00, sisa stock {} adalah {} porsi", 
                &food.name, &stock_buy, &food.price*&stock_buy, &food.name, &food.stock-&stock_buy);     
                self.inner.insert(
                    id,
                    Food {
                        id: id,
                        name: food.name,
                        stock: food.stock - stock_buy,
                        price: food.price,
                    },
                );   
            } else{
                self.inner.insert(
                    food.id,
                    Food {
                        id: food.id,
                        name: food.name,
                        stock: food.stock,
                        price: food.price,
                    },
                );
            }
        }
    }
}


#[derive(Error, Debug)]
enum ParseError {
    #[error("id must be a number: {0}")] InvalidId(#[from] std::num::ParseIntError),
    #[error("Maaf makanan tidak ditemukan")] FoundFood,
    #[error("Maaf stock kurang atau habis.....")] StockFood,
    #[error("Missing: {0}")] MissingField(String),
    #[error("Tidak ada makanan dengan nama {0} di data")] FoodNotFound(String),
}

fn parse_food(food: &str) -> Result<Food, ParseError> {
    let fields: Vec<&str> = food.split(',').collect();
    let id = match fields.get(0) {
        Some(id) => i64::from_str_radix(id, 10)?,
        None => return Err(ParseError::FoundFood),
    };

    let name = match fields.get(1).filter(|name| **name != "") {
        Some(name) => name.to_string(),
        None => return Err(ParseError::MissingField("name".to_owned())),
    };

    let stock = match fields.get(2) {
        Some(stock) => i64::from_str_radix(stock, 10)?,
        None => return Err(ParseError::FoundFood),
    };

    let price = match fields.get(3) {
        Some(price) => i64::from_str_radix(price, 10)?,
        None => return Err(ParseError::FoundFood),
    };

    Ok(Food {id,name,stock,price})
}

fn parse_foods(foods: String, verbose: bool) -> Foods {
    let mut fods = Foods::new();
    for (num, food) in foods.split('\n').enumerate(){
        if food != ""{
            match parse_food(food){
                Ok(fod) => fods.add(fod),
                Err(e) =>{
                    if verbose {
                        println!(
                            "error on line number {}: {}\n > \"{}\"\n",
                            num + 1,
                            e,
                            food
                        );
                    }
                }
            }
        }
    }
    fods
}

fn load_foods(file_name: PathBuf, verbose: bool) -> std::io::Result<Foods> {
    let mut file = File::open(file_name)?;

    let mut buffer = String::new();
    file.read_to_string(&mut buffer)?;

    Ok(parse_foods(buffer, verbose))
}

fn save_foods(file_name: PathBuf, foods: Foods) -> std::io::Result<()>{
    let mut file = OpenOptions::new()
    .write(true)
    .truncate(true)
    .open(file_name)?;

    file.write(b"id,name,stock,price\n")?;

    for food in foods.into_vec().into_iter(){
        let line = format!("{},{},{},{}\n", food.id, food.name, food.stock, food.price);
        file.write(line.as_bytes())?;
    }

    file.flush()?;
    Ok(())
}

#[derive(StructOpt, Debug)]
#[structopt(about = "Restaurant Application")]
struct Opt{
    #[structopt(short, parse(from_os_str), default_value = "food.csv")] data_file: PathBuf,
    #[structopt(subcommand)] cmd: Command,
    #[structopt(short, help = "verbose")] verbose: bool,
}

#[derive(StructOpt, Debug)]
enum Command {
    Add {
        name: String,
        stock: i64,
        price: i64,
    },
    Buy {
        name: String,
        stock: i64,
    },
    List {},
    Delete {
        name: String,
    },
}

fn run(opt: Opt) -> Result<(), std::io::Error> {
    match opt.cmd {
        Command::Add { name, stock, price} => { 
            let mut fods = load_foods(opt.data_file.clone(), opt.verbose)?;
            let results = fods.search(&name);
            match results {
                Some(p) => {
                    let mut fods = load_foods(opt.data_file.clone(), opt.verbose)?;
                    println!("Berhasil mengubah makanan, {}, dengan stok {}, dan harga Rp {},00", &name, &stock, &price);
                    fods.edit(p, &name, stock, price);
                    save_foods(opt.data_file, fods)?;
                },
                None => {
                    let mut fods = load_foods(opt.data_file.clone(), opt.verbose)?;
                    let next_id = fods.next_id();
                    println!("Berhasil menambahkan makanan baru, {}, dengan stok {}, dan harga Rp {},00", &name, &stock, &price);
                    fods.add(Food {
                        id: next_id,
                        name,
                        stock,
                        price,
                    });
                    save_foods(opt.data_file, fods)?;
                },
            }
        }
        Command::List { .. } => {
            let fods = load_foods(opt.data_file, opt.verbose)?;
            println!{"id,name,stock,price"};
            for food in fods.into_vec(){
                println!("{},{},{},{}", food.id, food.name, food.stock, food.price);
            }
        }
        Command::Delete {name} => {
            let mut fods = load_foods(opt.data_file.clone(), opt.verbose)?;
            let results = fods.search(&name);
            match results{
                Some(p) => {
                    let mut fods = load_foods(opt.data_file.clone(), opt.verbose)?;
                    fods.remove(&p);
                    fods.sort_foods(p);
                    save_foods(opt.data_file, fods)?;
                    println!("Berhasil menghapus {} dari daftar", &name);
                }
                None => {
                    println!("{}", ParseError::FoodNotFound(name));
                }
            }
        }
        Command::Buy {name,stock} => {
            let mut fods = load_foods(opt.data_file.clone(), opt.verbose)?;
            let results = fods.search_stock(&name, &stock);
            match results{
                Ok(p) => {
                    let mut fods = load_foods(opt.data_file.clone(), opt.verbose)?;
                    fods.buy(p, &name, stock);
                    save_foods(opt.data_file, fods)?;
                }
                Err(er) => {
                    println!("{}", er);
                }
            }
        }
    }
    Ok(())
}


fn main() {
    let opt = Opt::from_args();
    if let Err(e) = run(opt) {
        println!("an error occured: {}", e);
    } 
}
