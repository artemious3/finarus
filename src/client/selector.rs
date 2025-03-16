
use std::vec::Vec;
use std::io::Write;

pub fn select_from<T>(options : &Vec<T> ) ->  Option<T>
    where T : ToString + Copy
{

    if options.len() == 0{
        println!("Nothing to select from.");
        return None;
    }


    for (i,op) in options.iter().enumerate(){
        println!("{} : {}", i+1, op.to_string());
    }

    let sel : usize;

    loop {
        print!("\nOption: ");
        std::io::stdout().flush().unwrap();
        let mut inp_s = String::new();
        std::io::stdin().read_line(&mut inp_s).ok()?;
        let selected_opt = inp_s.trim().parse::<usize>();
        if let Ok(opt) = selected_opt {
            if opt > 0 && opt <= options.len() {
                    sel = opt;
                    break
            }
        }

        println!("Wrong input\n");
    }


    Some(options[sel-1])
}

pub fn select_idx<T>(options : &Vec<T> ) ->  Option<usize>
    where T : ToString 
{

    if options.len() == 0{
        println!("Nothing to select from.");
        return None;
    }

    for (i,op) in options.iter().enumerate(){
        println!("{} : {}", i+1, op.to_string());
    }

    let sel : usize;

    loop {
        print!("\nOption: ");
        std::io::stdout().flush().unwrap();
        let mut inp_s = String::new();
        std::io::stdin().read_line(&mut inp_s).ok()?;
        let selected_opt = inp_s.trim().parse::<usize>();
        if let Ok(opt) = selected_opt {
            if opt > 0 && opt <= options.len() {
                    sel = opt;
                    break
            }
        }

        println!("Wrong input\n");
    }


    Some(sel-1)
}
