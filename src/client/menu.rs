use std::collections::hash_map::{HashMap};
use std::boxed::Box;
use std::io::Write;

use crate::client::ClientContext;
use std::sync::{Arc, Mutex};

pub trait Action {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn exec(&mut self, ctx : Arc<Mutex<ClientContext>>) -> Result<(), String>;
}

pub struct Menu<'a>{
    pub name : &'static str,
    pub desc : &'static str,
    actions : HashMap<u8, Box<dyn Action + 'a> >
}

impl<'a> Menu<'a> {
    pub fn new() -> Self {
        Menu{
            name:"",
            desc:"",
            actions: HashMap::new()
        }
    }

    pub fn set_name(&mut self, name : &'static str) -> &mut Self{
        self.name = name;
        self
    }
    pub fn set_description(&mut self, desc : &'static str) -> &mut Self{
        self.desc = desc;
        self
    }

    pub fn add_action(&mut self, opt : u8, action : Box<dyn Action + 'a>) -> &mut Self{
        let _ = self.actions.insert(opt, action);
        self
    }

}

impl<'a> Action for Menu<'a> {
    fn exec(&mut self, ctx: Arc<Mutex<ClientContext>>) -> Result<(), String> {
        println!("Select an option:");
        loop {
            for (option, menu) in &self.actions {
                println!("   `{}` -- {}", *option as char, menu.name()); 
            }

            print!(">>> ");
            std::io::stdout().flush().expect("Input error");
            let mut inp = String::new();
            std::io::stdin().read_line(&mut inp).expect("Input error");
            
            // `inp` is 1 char + '\n'
            if inp.len() == 2 {
                let option = inp.as_bytes()[0];
                match self.actions.get_mut(&option){
                    Some(menu) => {
                        return menu.exec(ctx.clone()).map_err(|err : String|{
                            println!("ERROR : {}", err);
                           err 
                        });
                    }
                    None => ()
                }
            }

            println!("WRONG INPUT\n");
        }
    }

    fn name(&self) -> &'static str {
        self.name
    }
    fn description(&self) -> &'static str {
        self.desc
    }
}


// pub struct FnAction<'a>
// {
//     pub name : &'static str,
//     pub desc : &'static str,
//     func : Box<dyn FnMut()->Result<(), String> + 'a>,
// }
//
// impl<'a> FnAction<'a>
// {
//     pub fn new() -> Self{
//         FnAction{
//             name:"",
//             desc:"",
//             func: Box::new(|| Ok(()))
//         }
//     }
//     pub fn set_name(&mut self, name : &'static str) -> &mut Self{
//         self.name = name;
//         self
//     }
//     pub fn set_description(&mut self, desc : &'static str) -> &mut Self{
//         self.desc = desc;
//         self
//     }
//     pub fn set_func(&mut self,func : impl FnMut()->Result<(), String> + 'a) -> &mut Self{
//         self.func = Box::new(func);
//         self
//     }
// }
//
// impl<'a> Action for FnAction<'a>{
//     fn name(&self) -> &'static str {
//         self.name
//     }
//     fn description(&self) -> &'static str {
//         self.desc
//     }
//     fn exec(&mut self) -> Result<(), String> {
//         (self.func)()
//     }
// }
//


