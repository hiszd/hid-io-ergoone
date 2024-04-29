/// Handle layer event
/// out: raw string from TerminalOut
pub fn handle_layer_event(out: &str) -> () {
    let splt = out.split(":").collect::<Vec<&str>>();
    let layer = splt[1].parse::<u8>().unwrap();
    println!("Layer: {}", layer);
}
