use expression::Workspace;

mod expression;

fn get_user_input()->String{
    let mut line = String::new();
    std::io::stdin().read_line(&mut line).unwrap();
    return line
}


fn main() {
    println!("Hello, world!");
    let mut workspace = Workspace::new();

    // two_are_evil();

    while let Ok(_) = workspace.parse_expression(get_user_input().as_str()) {}
    println!("Done parsing");

    workspace.print_knowledge_base_from_all_expressions();
}


fn two_are_evil(){
    let names = vec!["steph", "anna", "tim", "matthew", "fraser", "you", "josh"];
    let mut out = String::new();
    for a in names.iter(){
        for b in names.iter(){
            if a <= b {continue;}
            out.push_str(("(!".to_string()+a+"&!"+b+")|").as_str());
        }
    }
    out.remove(out.len()-1);
    println!("{}", out);
}