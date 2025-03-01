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

    while let Ok(_) = workspace.parse_expression(get_user_input().as_str()) {}

    workspace.print_knowledge_base_from_all_expressions();
}

